#!/usr/bin/env python3
"""
Swarm-OS Telemetry Collector — v2

Captures full agent harness telemetry:
- Session lifecycle (start, end, duration)
- User queries (every prompt + response metadata)
- AI reasoning (thinking, planning, reflection, tool-selection steps)
- Tool calls (name, input, duration, success/error)
- Tool outputs (stdout, stderr, file content, errors)

Used by Claude Code hooks (SessionStart, UserPromptSubmit, PostToolUse, Stop)
and can be invoked manually for other agent types.

Usage:
    from telemetry_collector import TelemetryCollector
    tc = TelemetryCollector()
    session_id = tc.start_session(agent_type='claude-code', agent_version='4.6')
    query_id = tc.log_query(session_id, prompt='fix the bug')
    tc.log_tool_call(session_id, query_id, tool_name='Bash', tool_input={'cmd':'ls'},
                     output='file1.txt\nfile2.txt', success=True, duration_ms=120)
    tc.end_session(session_id, summary='Fixed the bug')
"""

from __future__ import annotations

import json
import sqlite3
import subprocess
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

# ── Constants ────────────────────────────────────────────────────────────────

PROJECT_ROOT = Path(__file__).resolve().parent.parent
DB_PATH = PROJECT_ROOT / ".swarm-os" / "telemetry.db"
SCHEMA_PATH = PROJECT_ROOT / "scripts" / "schema.sql"

# Truncate tool outputs beyond this to avoid DB bloat (1 MB)
MAX_OUTPUT_LENGTH = 1_000_000
# Truncate reasoning content beyond this (256 KB)
MAX_REASONING_LENGTH = 262_144
# Truncate user prompts beyond this (64 KB — prompts are usually short)
MAX_PROMPT_LENGTH = 65_536


# ── Helpers ──────────────────────────────────────────────────────────────────


def _utc_now_iso() -> str:
    """ISO-8601 UTC timestamp with milliseconds + 'Z' suffix."""
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ")


def _utc_now_sqlite() -> str:
    """SQLite-compatible UTC timestamp: 'YYYY-MM-DD HH:MM:SS' (matches datetime('now'))."""
    return datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S")


def _parse_timestamp(ts: str) -> Optional[datetime]:
    """Parse a timestamp in either ISO-8601 or SQLite format. Returns timezone-aware datetime."""
    if not ts:
        return None
    try:
        cleaned = ts.rstrip("Z")
        dt = datetime.fromisoformat(cleaned)
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        return dt
    except Exception:
        pass
    for fmt in ("%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S.%f"):
        try:
            dt = datetime.strptime(ts, fmt)
            return dt.replace(tzinfo=timezone.utc)
        except Exception:
            continue
    return None


def _estimate_tokens(text: str) -> int:
    """Rough token estimate: ~4 chars per token for English."""
    if not text:
        return 0
    return max(1, len(text) // 4)


def _truncate(text: str, max_length: int) -> tuple[str, bool]:
    """Truncate text to max_length. Returns (text, was_truncated)."""
    if text is None:
        return "", False
    if len(text) <= max_length:
        return text, False
    return text[:max_length] + "\n...[truncated]", True


def _git_branch() -> Optional[str]:
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            capture_output=True,
            text=True,
            cwd=PROJECT_ROOT,
            timeout=5,
        )
        return result.stdout.strip() if result.returncode == 0 else None
    except Exception:
        return None


def _git_head_sha() -> Optional[str]:
    try:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            cwd=PROJECT_ROOT,
            timeout=5,
        )
        return result.stdout.strip() if result.returncode == 0 else None
    except Exception:
        return None


def _git_diff_stats() -> tuple[int, int, int]:
    """Returns (files_changed, lines_added, lines_removed) since last commit."""
    try:
        result = subprocess.run(
            ["git", "diff", "--stat", "HEAD"],
            capture_output=True,
            text=True,
            cwd=PROJECT_ROOT,
            timeout=5,
        )
        if result.returncode != 0:
            return 0, 0, 0
        # Parse last line: " 5 files changed, 120 insertions(+), 30 deletions(-)"
        lines = result.stdout.strip().split("\n")
        if not lines or not lines[-1]:
            return 0, 0, 0
        last = lines[-1]
        files = 0
        added = 0
        removed = 0
        if "file" in last:
            parts = last.split(",")
            for part in parts:
                part = part.strip()
                if "file" in part:
                    files = int(part.split()[0])
                elif "insertion" in part:
                    added = int(part.split()[0])
                elif "deletion" in part:
                    removed = int(part.split()[0])
        return files, added, removed
    except Exception:
        return 0, 0, 0


# ── Collector ────────────────────────────────────────────────────────────────


class TelemetryCollector:
    """Thread-safe SQLite-backed telemetry collector."""

    def __init__(self, db_path: Path = DB_PATH) -> None:
        self.db_path = db_path
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self._init_schema()

    def _conn(self) -> sqlite3.Connection:
        conn = sqlite3.connect(str(self.db_path), timeout=30.0)
        conn.execute("PRAGMA journal_mode=WAL")
        conn.execute("PRAGMA foreign_keys=ON")
        return conn

    def _init_schema(self) -> None:
        with self._conn() as conn:
            conn.executescript(SCHEMA_PATH.read_text())
            conn.commit()

    # ── Sessions ─────────────────────────────────────────────────────────────

    def start_session(
        self,
        agent_type: str = "unknown",
        agent_version: Optional[str] = None,
        cwd: Optional[str] = None,
    ) -> str:
        """Create a new session row. Returns the session_id (UUID v4)."""
        session_id = str(uuid.uuid4())
        branch = _git_branch()
        head_sha = _git_head_sha()
        started_at = _utc_now_sqlite()
        with self._conn() as conn:
            conn.execute(
                """INSERT INTO sessions
                   (id, started_at, agent_type, agent_version, cwd, git_branch, git_head_sha)
                   VALUES (?, ?, ?, ?, ?, ?, ?)""",
                (
                    session_id,
                    started_at,
                    agent_type,
                    agent_version,
                    cwd or str(PROJECT_ROOT),
                    branch,
                    head_sha,
                ),
            )
            conn.commit()
        # Persist head_sha so end_session can count new commits
        head_sha_file = self.db_path.parent / "session-start-sha"
        head_sha_file.parent.mkdir(parents=True, exist_ok=True)
        head_sha_file.write_text(head_sha or "")
        return session_id

    def end_session(
        self,
        session_id: str,
        summary: Optional[str] = None,
        tasks_completed: int = 0,
        tasks_failed: int = 0,
        tests_run: int = 0,
        tests_passed: int = 0,
        commits_made: Optional[int] = None,
        exit_reason: str = "normal",
    ) -> None:
        """Mark a session as ended and record summary stats."""
        files_changed, lines_added, lines_removed = _git_diff_stats()
        ended_at = _utc_now_sqlite()

        if commits_made is None:
            commits_made = self._count_commits_since_session_start()

        with self._conn() as conn:
            row = conn.execute(
                "SELECT started_at FROM sessions WHERE id = ?",
                (session_id,),
            ).fetchone()
            duration_seconds = None
            if row:
                started = _parse_timestamp(row[0])
                if started:
                    duration_seconds = int(
                        (datetime.now(timezone.utc) - started).total_seconds()
                    )

            conn.execute(
                """UPDATE sessions
                   SET ended_at = ?, duration_seconds = ?,
                       files_changed = ?, lines_added = ?, lines_removed = ?,
                       exit_reason = ?
                   WHERE id = ?""",
                (
                    ended_at,
                    duration_seconds,
                    files_changed,
                    lines_added,
                    lines_removed,
                    exit_reason,
                    session_id,
                ),
            )

            if summary:
                conn.execute(
                    """INSERT OR REPLACE INTO session_summaries
                       (session_id, summary, tasks_completed, tasks_failed,
                        tests_run, tests_passed, commits_made)
                       VALUES (?, ?, ?, ?, ?, ?, ?)""",
                    (
                        session_id,
                        summary,
                        tasks_completed,
                        tasks_failed,
                        tests_run,
                        tests_passed,
                        commits_made,
                    ),
                )
            conn.commit()

        head_sha_file = self.db_path.parent / "session-start-sha"
        try:
            head_sha_file.unlink()
        except FileNotFoundError:
            pass

    def _count_commits_since_session_start(self) -> int:
        """Count commits made between session-start HEAD and current HEAD."""
        head_sha_file = self.db_path.parent / "session-start-sha"
        if not head_sha_file.exists():
            return 0
        start_sha = head_sha_file.read_text().strip()
        if not start_sha:
            return 0
        try:
            result = subprocess.run(
                ["git", "rev-list", "--count", f"{start_sha}..HEAD"],
                capture_output=True,
                text=True,
                cwd=PROJECT_ROOT,
                timeout=5,
            )
            if result.returncode == 0:
                return int(result.stdout.strip())
        except Exception:
            pass
        return 0

    # ── User queries ─────────────────────────────────────────────────────────

    def log_query(
        self,
        session_id: str,
        prompt: str,
        response_duration_ms: Optional[int] = None,
        response_tokens_estimate: Optional[int] = None,
    ) -> int:
        """Log a user query. Returns the query_id (auto-increment)."""
        prompt_truncated, _ = _truncate(prompt, MAX_PROMPT_LENGTH)
        # Get next query_number within this session
        with self._conn() as conn:
            row = conn.execute(
                "SELECT COALESCE(MAX(query_number), 0) + 1 FROM user_queries WHERE session_id = ?",
                (session_id,),
            ).fetchone()
            query_number = row[0] if row else 1

            cur = conn.execute(
                """INSERT INTO user_queries
                   (session_id, query_number, prompt, prompt_tokens_estimate,
                    response_duration_ms, response_tokens_estimate)
                   VALUES (?, ?, ?, ?, ?, ?)""",
                (
                    session_id,
                    query_number,
                    prompt_truncated,
                    _estimate_tokens(prompt),
                    response_duration_ms,
                    response_tokens_estimate,
                ),
            )
            query_id = cur.lastrowid

            # Increment session query_count
            conn.execute(
                "UPDATE sessions SET query_count = query_count + 1 WHERE id = ?",
                (session_id,),
            )
            conn.commit()
        return query_id

    def update_query_response(
        self,
        query_id: int,
        response_duration_ms: int,
        response_tokens_estimate: Optional[int] = None,
        tool_calls_invoked: Optional[int] = None,
        files_modified: Optional[int] = None,
    ) -> None:
        """Update a query row with response metadata after the AI finishes.

        Only updates fields that are not None — preserves existing values
        for fields that were already set (e.g. tool_calls_invoked incremented
        by start_tool_call).
        """
        with self._conn() as conn:
            sets = ["response_duration_ms = ?"]
            params = [response_duration_ms]
            if response_tokens_estimate is not None:
                sets.append("response_tokens_estimate = ?")
                params.append(response_tokens_estimate)
            if tool_calls_invoked is not None:
                sets.append("tool_calls_invoked = ?")
                params.append(tool_calls_invoked)
            if files_modified is not None:
                sets.append("files_modified = ?")
                params.append(files_modified)
            params.append(query_id)
            conn.execute(
                f"UPDATE user_queries SET {', '.join(sets)} WHERE id = ?",
                params,
            )
            conn.commit()

    # ── AI reasoning ─────────────────────────────────────────────────────────

    def log_reasoning(
        self,
        session_id: str,
        query_id: int,
        reasoning_type: str,
        content: str,
        duration_ms: Optional[int] = None,
    ) -> int:
        """Log an AI reasoning/thinking step. Returns the reasoning row id.

        reasoning_type: 'thinking', 'planning', 'reflection', 'decision', 'tool_selection'
        """
        assert reasoning_type in (
            "thinking",
            "planning",
            "reflection",
            "decision",
            "tool_selection",
        ), f"Invalid reasoning_type: {reasoning_type}"

        content_truncated, _ = _truncate(content, MAX_REASONING_LENGTH)
        with self._conn() as conn:
            row = conn.execute(
                """SELECT COALESCE(MAX(step_number), 0) + 1
                   FROM ai_reasoning WHERE session_id = ? AND query_id = ?""",
                (session_id, query_id),
            ).fetchone()
            step_number = row[0] if row else 1

            cur = conn.execute(
                """INSERT INTO ai_reasoning
                   (session_id, query_id, step_number, reasoning_type, content, duration_ms)
                   VALUES (?, ?, ?, ?, ?, ?)""",
                (
                    session_id,
                    query_id,
                    step_number,
                    reasoning_type,
                    content_truncated,
                    duration_ms,
                ),
            )
            reasoning_id = cur.lastrowid
            conn.commit()
        return reasoning_id

    # ── Tool calls ───────────────────────────────────────────────────────────

    def start_tool_call(
        self,
        session_id: str,
        query_id: int,
        tool_name: str,
        tool_input: Optional[dict] = None,
    ) -> int:
        """Record the start of a tool call (called from PreToolUse hook).
        Returns the tool_call_id. Call end_tool_call() with the same id
        once the tool completes to populate duration_ms + output + success."""
        with self._conn() as conn:
            row = conn.execute(
                """SELECT COALESCE(MAX(call_number), 0) + 1
                   FROM tool_calls WHERE session_id = ?""",
                (session_id,),
            ).fetchone()
            call_number = row[0] if row else 1

            tool_input_json = (
                json.dumps(tool_input, default=str) if tool_input else None
            )

            cur = conn.execute(
                """INSERT INTO tool_calls
                   (session_id, query_id, call_number, tool_name, tool_input,
                    timestamp, success)
                   VALUES (?, ?, ?, ?, ?, ?, 1)""",
                (
                    session_id,
                    query_id,
                    call_number,
                    tool_name,
                    tool_input_json,
                    _utc_now_sqlite(),
                ),
            )
            tool_call_id = cur.lastrowid

            conn.execute(
                "UPDATE sessions SET tool_call_count = tool_call_count + 1 WHERE id = ?",
                (session_id,),
            )
            if query_id:
                conn.execute(
                    "UPDATE user_queries SET tool_calls_invoked = tool_calls_invoked + 1 WHERE id = ?",
                    (query_id,),
                )
            conn.commit()
        return tool_call_id

    def end_tool_call(
        self,
        tool_call_id: int,
        output: Optional[str] = None,
        output_type: str = "stdout",
        success: bool = True,
        error_message: Optional[str] = None,
    ) -> None:
        """Record the completion of a tool call (called from PostToolUse hook).
        Computes duration_ms from the tool_calls.timestamp column."""
        with self._conn() as conn:
            row = conn.execute(
                "SELECT timestamp FROM tool_calls WHERE id = ?",
                (tool_call_id,),
            ).fetchone()
            duration_ms = None
            if row:
                started = _parse_timestamp(row[0])
                if started:
                    duration_ms = int(
                        (datetime.now(timezone.utc) - started).total_seconds() * 1000
                    )

            conn.execute(
                """UPDATE tool_calls
                   SET duration_ms = ?, success = ?, error_message = ?
                   WHERE id = ?""",
                (duration_ms, 1 if success else 0, error_message, tool_call_id),
            )

            if output is not None:
                output_truncated, was_truncated = _truncate(output, MAX_OUTPUT_LENGTH)
                conn.execute(
                    """INSERT INTO tool_outputs
                       (tool_call_id, output_type, content, content_length, truncated)
                       VALUES (?, ?, ?, ?, ?)""",
                    (tool_call_id, output_type, output_truncated, len(output),
                     1 if was_truncated else 0),
                )
            conn.commit()

    def log_tool_call(
        self,
        session_id: str,
        query_id: int,
        tool_name: str,
        tool_input: Optional[dict] = None,
        output: Optional[str] = None,
        output_type: str = "stdout",
        success: bool = True,
        error_message: Optional[str] = None,
        duration_ms: Optional[int] = None,
    ) -> int:
        """Log a complete tool call in one shot (for manual/retroactive logging).
        For real-time capture, prefer start_tool_call + end_tool_call."""
        with self._conn() as conn:
            row = conn.execute(
                """SELECT COALESCE(MAX(call_number), 0) + 1
                   FROM tool_calls WHERE session_id = ?""",
                (session_id,),
            ).fetchone()
            call_number = row[0] if row else 1

            tool_input_json = (
                json.dumps(tool_input, default=str) if tool_input else None
            )

            cur = conn.execute(
                """INSERT INTO tool_calls
                   (session_id, query_id, call_number, tool_name, tool_input,
                    timestamp, duration_ms, success, error_message)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    session_id, query_id, call_number, tool_name, tool_input_json,
                    _utc_now_sqlite(), duration_ms, 1 if success else 0, error_message,
                ),
            )
            tool_call_id = cur.lastrowid

            if output is not None:
                output_truncated, was_truncated = _truncate(output, MAX_OUTPUT_LENGTH)
                conn.execute(
                    """INSERT INTO tool_outputs
                       (tool_call_id, output_type, content, content_length, truncated)
                       VALUES (?, ?, ?, ?, ?)""",
                    (tool_call_id, output_type, output_truncated, len(output),
                     1 if was_truncated else 0),
                )

            conn.execute(
                "UPDATE sessions SET tool_call_count = tool_call_count + 1 WHERE id = ?",
                (session_id,),
            )
            if query_id:
                conn.execute(
                    "UPDATE user_queries SET tool_calls_invoked = tool_calls_invoked + 1 WHERE id = ?",
                    (query_id,),
                )
            conn.commit()
        return tool_call_id

    # ── Build metrics (v1 compatibility) ─────────────────────────────────────

    def log_build_metrics(
        self,
        stack: str = "all",
        compile_time_ms: int = 0,
        test_total: int = 0,
        test_passed: int = 0,
        test_failed: int = 0,
        test_skipped: int = 0,
        binary_size_bytes: int = 0,
        lint_errors: int = 0,
    ) -> None:
        with self._conn() as conn:
            conn.execute(
                """INSERT INTO build_metrics
                   (compile_time_ms, test_total, test_passed, test_failed,
                    test_skipped, binary_size_bytes, lint_errors, stack)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    compile_time_ms,
                    test_total,
                    test_passed,
                    test_failed,
                    test_skipped,
                    binary_size_bytes,
                    lint_errors,
                    stack,
                ),
            )
            conn.commit()


# ── CLI entrypoint ───────────────────────────────────────────────────────────


def main() -> int:
    """Smoke test: create a session, log a query + tool call, end session."""
    tc = TelemetryCollector()
    session_id = tc.start_session(
        agent_type="super-z",
        agent_version="test",
        cwd=str(PROJECT_ROOT),
    )
    print(f"Started session: {session_id}")

    query_id = tc.log_query(
        session_id, prompt="Smoke test query from telemetry_collector"
    )
    print(f"Logged query: id={query_id}")

    tc.log_reasoning(
        session_id,
        query_id,
        reasoning_type="planning",
        content="Planning to run a smoke test of the telemetry pipeline.",
        duration_ms=12,
    )
    tc.log_reasoning(
        session_id,
        query_id,
        reasoning_type="decision",
        content="Decided to invoke Bash to verify the DB exists.",
        duration_ms=5,
    )

    tc.log_tool_call(
        session_id,
        query_id,
        tool_name="Bash",
        tool_input={"command": "ls -la .swarm-os/telemetry.db"},
        output="-rw-r--r-- 1 z z 6144 .swarm-os/telemetry.db",
        output_type="stdout",
        success=True,
        duration_ms=42,
    )

    tc.end_session(
        session_id,
        summary="Smoke test session — telemetry pipeline verified.",
        tasks_completed=1,
        tests_run=0,
        commits_made=0,
    )
    print(f"Ended session: {session_id}")
    print(f"DB path: {DB_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
