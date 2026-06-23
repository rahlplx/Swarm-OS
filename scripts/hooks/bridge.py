#!/usr/bin/env python3
"""
Swarm-OS Hook Bridge v2 — feeds Claude Code hook events into the telemetry pipeline.

Hooks:
- SessionStart    → bridge.py session-start
- UserPromptSubmit → bridge.py user-query
- PreToolUse      → bridge.py tool-call-start  (records start time)
- PostToolUse     → bridge.py tool-call-end    (records output + computes duration)
- Stop            → bridge.py session-end
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(SCRIPTS_DIR))

from telemetry_collector import TelemetryCollector  # noqa: E402

PROJECT_ROOT = SCRIPTS_DIR.parent
STATE_DIR = PROJECT_ROOT / ".swarm-os"
SESSION_FILE = STATE_DIR / "current-session"
QUERY_START_FILE = STATE_DIR / "current-query-start"
TOOL_CALL_FILE = STATE_DIR / "current-tool-call"


def _read_stdin_json() -> dict:
    try:
        raw = sys.stdin.read()
        return json.loads(raw) if raw.strip() else {}
    except Exception:
        return {}


def _load_session_id():
    try:
        return SESSION_FILE.read_text().strip() if SESSION_FILE.exists() else None
    except Exception:
        return None


def _save_session_id(sid: str) -> None:
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    SESSION_FILE.write_text(sid)


def _clear_session_id() -> None:
    try:
        SESSION_FILE.unlink()
    except FileNotFoundError:
        pass


def _load_query_state():
    try:
        return json.loads(QUERY_START_FILE.read_text()) if QUERY_START_FILE.exists() else None
    except Exception:
        return None


def _save_tool_call_id(tc_id: int) -> None:
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    TOOL_CALL_FILE.write_text(str(tc_id))


def _load_tool_call_id():
    try:
        return int(TOOL_CALL_FILE.read_text().strip()) if TOOL_CALL_FILE.exists() else None
    except Exception:
        return None


def _clear_tool_call_id() -> None:
    try:
        TOOL_CALL_FILE.unlink()
    except FileNotFoundError:
        pass


def _detect_agent_type(payload: dict):
    cwd = payload.get("cwd", "")
    if os.environ.get("CLAUDE_SESSION_ID") or payload.get("session_id"):
        return "claude-code", os.environ.get("CLAUDE_VERSION")
    if "/home/z/my-project" in cwd:
        return "super-z", None
    return "unknown", None


def _extract_output(tool_response):
    if tool_response is None:
        return None
    if isinstance(tool_response, str):
        return tool_response
    if isinstance(tool_response, dict):
        if tool_response.get("error"):
            return str(tool_response["error"])
        if tool_response.get("stdout"):
            out = tool_response["stdout"]
            if tool_response.get("stderr"):
                out += "\n--- stderr ---\n" + tool_response["stderr"]
            return out
        if tool_response.get("content"):
            content = tool_response["content"]
            if isinstance(content, list):
                parts = []
                for item in content:
                    if isinstance(item, dict):
                        parts.append(item.get("text", str(item)))
                    else:
                        parts.append(str(item))
                return "\n".join(parts)
            return str(content)
        return json.dumps(tool_response, default=str)
    return str(tool_response)


def cmd_session_start(payload: dict) -> int:
    tc = TelemetryCollector()
    agent_type, agent_version = _detect_agent_type(payload)
    session_id = tc.start_session(agent_type=agent_type, agent_version=agent_version, cwd=payload.get("cwd"))
    _save_session_id(session_id)
    print("─" * 72, file=sys.stderr)
    print(f"Swarm-OS session started. session_id={session_id} agent={agent_type}", file=sys.stderr)
    print("Telemetry v2 ACTIVE: queries, reasoning, tool calls (with duration),", file=sys.stderr)
    print("tool outputs, commits, session summary.", file=sys.stderr)
    print("─" * 72, file=sys.stderr)
    return 0


def cmd_user_query(payload: dict) -> int:
    session_id = _load_session_id()
    if not session_id:
        tc = TelemetryCollector()
        agent_type, agent_version = _detect_agent_type(payload)
        session_id = tc.start_session(agent_type=agent_type, agent_version=agent_version, cwd=payload.get("cwd"))
        _save_session_id(session_id)
    tc = TelemetryCollector()
    query_id = tc.log_query(session_id, prompt=payload.get("prompt", ""))
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    QUERY_START_FILE.write_text(json.dumps({"query_id": query_id, "started_at_ms": int(time.time() * 1000)}))
    return 0


def cmd_tool_call_start(payload: dict) -> int:
    session_id = _load_session_id()
    if not session_id:
        return 0
    query_state = _load_query_state()
    query_id = query_state.get("query_id") if query_state else None
    tc = TelemetryCollector()
    tool_input = payload.get("tool_input", {})
    tool_call_id = tc.start_tool_call(
        session_id=session_id,
        query_id=query_id or 0,
        tool_name=payload.get("tool_name", "Unknown"),
        tool_input=tool_input if isinstance(tool_input, dict) else {"raw": str(tool_input)},
    )
    _save_tool_call_id(tool_call_id)
    return 0


def cmd_tool_call_end(payload: dict) -> int:
    session_id = _load_session_id()
    if not session_id:
        return 0
    tool_call_id = _load_tool_call_id()
    if not tool_call_id:
        # Fallback: PreToolUse didn't fire — log in one shot
        query_state = _load_query_state()
        query_id = query_state.get("query_id") if query_state else None
        tc = TelemetryCollector()
        tool_response = payload.get("tool_response")
        output = _extract_output(tool_response)
        success = not (isinstance(tool_response, dict) and tool_response.get("error"))
        error_message = str(tool_response["error"]) if isinstance(tool_response, dict) and tool_response.get("error") else None
        tool_input = payload.get("tool_input", {})
        tc.log_tool_call(
            session_id=session_id, query_id=query_id or 0,
            tool_name=payload.get("tool_name", "Unknown"),
            tool_input=tool_input if isinstance(tool_input, dict) else {"raw": str(tool_input)},
            output=output, output_type="result", success=success, error_message=error_message,
        )
        return 0
    tc = TelemetryCollector()
    tool_response = payload.get("tool_response")
    output = _extract_output(tool_response)
    success = True
    error_message = None
    output_type = "result"
    if isinstance(tool_response, dict) and tool_response.get("error"):
        success = False
        error_message = str(tool_response["error"])
        output_type = "error"
    elif isinstance(tool_response, dict) and tool_response.get("stdout"):
        output_type = "stdout"
    tc.end_tool_call(
        tool_call_id=tool_call_id, output=output, output_type=output_type,
        success=success, error_message=error_message,
    )
    _clear_tool_call_id()
    return 0


def cmd_session_end(payload: dict) -> int:
    session_id = _load_session_id()
    if not session_id:
        return 0
    tc = TelemetryCollector()
    # Update last query with response metadata
    if QUERY_START_FILE.exists():
        try:
            data = json.loads(QUERY_START_FILE.read_text())
            started_at_ms = data.get("started_at_ms", 0)
            query_id = data.get("query_id")
            duration_ms = int(time.time() * 1000) - started_at_ms
            if query_id:
                import sqlite3
                conn = sqlite3.connect(str(tc.db_path))
                total_output_bytes = conn.execute(
                    """SELECT COALESCE(SUM(LENGTH(to2.content)), 0)
                       FROM tool_outputs to2
                       JOIN tool_calls tc2 ON to2.tool_call_id = tc2.id
                       WHERE tc2.query_id = ?""",
                    (query_id,),
                ).fetchone()[0]
                files_modified = conn.execute(
                    """SELECT COUNT(DISTINCT json_extract(tool_input, '$.filepath'))
                       FROM tool_calls
                       WHERE query_id = ?
                         AND tool_name IN ('Edit', 'Write', 'MultiEdit', 'NotebookEdit')""",
                    (query_id,),
                ).fetchone()[0]
                conn.close()
                response_tokens = max(1, total_output_bytes // 4) if total_output_bytes else None
                tc.update_query_response(
                    query_id, response_duration_ms=duration_ms,
                    response_tokens_estimate=response_tokens, files_modified=files_modified,
                )
        except Exception:
            pass
        try:
            QUERY_START_FILE.unlink()
        except FileNotFoundError:
            pass
    # Summary
    summary_parts = []
    try:
        result = subprocess.run(["git", "diff", "--stat", "HEAD"], capture_output=True, text=True, cwd=PROJECT_ROOT, timeout=5)
        if result.stdout.strip():
            lines = result.stdout.strip().splitlines()
            if lines:
                summary_parts.append(f"Changes: {lines[-1]}")
    except Exception:
        pass
    summary = " | ".join(summary_parts) if summary_parts else "Session ended."
    tc.end_session(session_id=session_id, summary=summary, exit_reason="normal")
    _clear_session_id()
    _clear_tool_call_id()
    return 0


def main() -> int:
    if len(sys.argv) < 2:
        print("Usage: bridge.py <session-start|user-query|tool-call-start|tool-call-end|session-end>", file=sys.stderr)
        return 1
    command = sys.argv[1]
    payload = _read_stdin_json()
    handlers = {
        "session-start": cmd_session_start,
        "user-query": cmd_user_query,
        "tool-call-start": cmd_tool_call_start,
        "tool-call-end": cmd_tool_call_end,
        "tool-call": cmd_tool_call_end,
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
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
