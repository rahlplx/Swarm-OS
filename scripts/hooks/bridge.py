#!/usr/bin/env python3
"""
Swarm-OS Hook Bridge — feeds Claude Code hook events into the telemetry pipeline.

Invoked by .claude/settings.json hooks:
- SessionStart  → bridge.py session-start
- UserPromptSubmit → bridge.py user-query    (reads prompt from stdin JSON)
- PostToolUse   → bridge.py tool-call        (reads tool call from stdin JSON)
- Stop          → bridge.py session-end      (reads transcript path from stdin JSON)

Claude Code passes a JSON object on stdin for each hook event. The schema varies
by event type but typically includes: session_id, transcript_path, cwd, hook_event_name,
and (for UserPromptSubmit) the prompt string.

We persist a stable session_id by writing it to .swarm-os/current-session on
SessionStart, and reading it on subsequent events.
"""

from __future__ import annotations

import json
import os
import sys
import time
from pathlib import Path
from typing import Any

# Add scripts/ to path so we can import telemetry_collector
# bridge.py lives in scripts/hooks/, so parent.parent = scripts/
SCRIPTS_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(SCRIPTS_DIR))

from telemetry_collector import TelemetryCollector  # noqa: E402

PROJECT_ROOT = SCRIPTS_DIR.parent
SESSION_FILE = PROJECT_ROOT / ".swarm-os" / "current-session"
QUERY_START_FILE = PROJECT_ROOT / ".swarm-os" / "current-query-start"


def _read_stdin_json() -> dict[str, Any]:
    """Read JSON from stdin. Returns empty dict on parse failure."""
    try:
        raw = sys.stdin.read()
        if not raw.strip():
            return {}
        return json.loads(raw)
    except Exception:
        return {}


def _load_session_id() -> str | None:
    """Load the current session ID from disk."""
    try:
        if SESSION_FILE.exists():
            return SESSION_FILE.read_text().strip()
    except Exception:
        pass
    return None


def _save_session_id(session_id: str) -> None:
    SESSION_FILE.parent.mkdir(parents=True, exist_ok=True)
    SESSION_FILE.write_text(session_id)


def _clear_session_id() -> None:
    try:
        SESSION_FILE.unlink()
    except FileNotFoundError:
        pass


def _detect_agent_type(payload: dict) -> tuple[str, str | None]:
    """Detect agent type and version from hook payload / env."""
    cwd = payload.get("cwd", "")
    # Claude Code sets CLAUDE_SESSION_ID in env
    if os.environ.get("CLAUDE_SESSION_ID") or payload.get("session_id"):
        return "claude-code", os.environ.get("CLAUDE_VERSION")
    # Super Z sandbox
    if "/home/z/my-project" in cwd:
        return "super-z", None
    return "unknown", None


# ── Event handlers ───────────────────────────────────────────────────────────


def cmd_session_start(payload: dict) -> int:
    """Handle SessionStart event."""
    tc = TelemetryCollector()
    agent_type, agent_version = _detect_agent_type(payload)
    session_id = tc.start_session(
        agent_type=agent_type,
        agent_version=agent_version,
        cwd=payload.get("cwd"),
    )
    _save_session_id(session_id)
    # Emit the bootstrap banner (same as on-session-start.sh)
    print(
        "────────────────────────────────────────────────────────────────────────",
        file=sys.stderr,
    )
    print(
        f"Swarm-OS session started. session_id={session_id} agent={agent_type}",
        file=sys.stderr,
    )
    print(
        "Telemetry pipeline ACTIVE: queries, reasoning, tool calls, outputs.",
        file=sys.stderr,
    )
    print(
        "────────────────────────────────────────────────────────────────────────",
        file=sys.stderr,
    )
    return 0


def cmd_user_query(payload: dict) -> int:
    """Handle UserPromptSubmit event."""
    session_id = _load_session_id()
    if not session_id:
        # Session wasn't started (e.g. hook fired before SessionStart) — start one
        tc = TelemetryCollector()
        agent_type, agent_version = _detect_agent_type(payload)
        session_id = tc.start_session(
            agent_type=agent_type,
            agent_version=agent_version,
            cwd=payload.get("cwd"),
        )
        _save_session_id(session_id)

    tc = TelemetryCollector()
    prompt = payload.get("prompt", "")
    query_id = tc.log_query(session_id, prompt=prompt)
    # Record query start time so PostToolUse/Stop can compute response_duration
    QUERY_START_FILE.parent.mkdir(parents=True, exist_ok=True)
    QUERY_START_FILE.write_text(
        json.dumps({"query_id": query_id, "started_at_ms": int(time.time() * 1000)})
    )
    return 0


def cmd_tool_call(payload: dict) -> int:
    """Handle PostToolUse event."""
    session_id = _load_session_id()
    if not session_id:
        return 0  # No active session — nothing to log

    # Load current query_id
    query_id = None
    if QUERY_START_FILE.exists():
        try:
            data = json.loads(QUERY_START_FILE.read_text())
            query_id = data.get("query_id")
        except Exception:
            pass

    tc = TelemetryCollector()

    # Claude Code PostToolUse payload shape:
    # {tool_name, tool_input, tool_response, ...}
    tool_name = payload.get("tool_name", "Unknown")
    tool_input = payload.get("tool_input", {})
    tool_response = payload.get("tool_response", {})
    success = True
    error_message = None
    output = None
    output_type = "result"

    if isinstance(tool_response, dict):
        if tool_response.get("error"):
            success = False
            error_message = str(tool_response["error"])
            output = error_message
            output_type = "error"
        elif tool_response.get("stdout"):
            output = tool_response["stdout"]
            output_type = "stdout"
            if tool_response.get("stderr"):
                output += "\n--- stderr ---\n" + tool_response["stderr"]
        elif tool_response.get("content"):
            # Read/Edit/Grep return {content: [...]}
            content = tool_response["content"]
            if isinstance(content, list):
                output = "\n".join(
                    item.get("text", str(item)) if isinstance(item, dict) else str(item)
                    for item in content
                )
            else:
                output = str(content)
            output_type = "result"
        else:
            output = json.dumps(tool_response, default=str)
    elif isinstance(tool_response, str):
        output = tool_response
    elif tool_response is not None:
        output = str(tool_response)

    tc.log_tool_call(
        session_id=session_id,
        query_id=query_id or 0,  # 0 = unknown query
        tool_name=tool_name,
        tool_input=tool_input if isinstance(tool_input, dict) else {"raw": str(tool_input)},
        output=output,
        output_type=output_type,
        success=success,
        error_message=error_message,
        duration_ms=payload.get("duration_ms"),
    )
    return 0


def cmd_session_end(payload: dict) -> int:
    """Handle Stop event."""
    session_id = _load_session_id()
    if not session_id:
        return 0

    tc = TelemetryCollector()

    # Compute response_duration for the last query (rough)
    if QUERY_START_FILE.exists():
        try:
            data = json.loads(QUERY_START_FILE.read_text())
            started_at_ms = data.get("started_at_ms", 0)
            duration_ms = int(time.time() * 1000) - started_at_ms
            query_id = data.get("query_id")
            if query_id:
                tc.update_query_response(query_id, response_duration_ms=duration_ms)
        except Exception:
            pass
        QUERY_START_FILE.unlink(missing_ok=True)

    # Generate summary from git diff + session stats
    import subprocess

    summary_parts = []
    try:
        result = subprocess.run(
            ["git", "diff", "--stat", "HEAD"],
            capture_output=True,
            text=True,
            cwd=PROJECT_ROOT,
            timeout=5,
        )
        if result.stdout.strip():
            summary_parts.append(f"Changes: {result.stdout.strip().splitlines()[-1]}")
    except Exception:
        pass

    # Count commits made this session
    # We can't easily know which commits were "this session" without a start marker
    # so we just report total. A more precise approach would save HEAD at SessionStart.
    try:
        subprocess.run(
            ["git", "rev-list", "--count", "HEAD"],
            capture_output=True,
            text=True,
            cwd=PROJECT_ROOT,
            timeout=5,
        )
    except Exception:
        pass

    summary = " | ".join(summary_parts) if summary_parts else "Session ended."

    tc.end_session(
        session_id=session_id,
        summary=summary,
        exit_reason="normal",
    )
    _clear_session_id()
    return 0


# ── CLI ──────────────────────────────────────────────────────────────────────


def main() -> int:
    if len(sys.argv) < 2:
        print("Usage: bridge.py <session-start|user-query|tool-call|session-end>", file=sys.stderr)
        return 1

    command = sys.argv[1]
    payload = _read_stdin_json()

    handlers = {
        "session-start": cmd_session_start,
        "user-query": cmd_user_query,
        "tool-call": cmd_tool_call,
        "session-end": cmd_session_end,
    }

    handler = handlers.get(command)
    if not handler:
        print(f"Unknown command: {command}", file=sys.stderr)
        return 1

    try:
        return handler(payload)
    except Exception as exc:
        print(f"bridge.py {command} failed: {exc}", file=sys.stderr)
        return 0  # Don't block the agent on telemetry failures


if __name__ == "__main__":
    raise SystemExit(main())
