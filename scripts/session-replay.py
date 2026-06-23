#!/usr/bin/env python3
"""
Swarm-OS Session Replay — dumps a full session timeline to stdout.

Shows every user query, AI reasoning step, tool call, and tool output
in chronological order for a given session (or the most recent session
if no session_id is provided).

Usage:
    python3 scripts/session-replay.py                  # most recent session
    python3 scripts/session-replay.py <session_id>     # specific session
    python3 scripts/session-replay.py --list           # list all sessions
    python3 scripts/session-replay.py --stats          # aggregate stats
"""

from __future__ import annotations

import json
import sqlite3
import sys
from pathlib import Path

DB_PATH = Path(__file__).resolve().parent.parent / ".swarm-os" / "telemetry.db"


def _connect() -> sqlite3.Connection:
    if not DB_PATH.exists():
        print(f"Telemetry DB not found at {DB_PATH}", file=sys.stderr)
        print("Run a session first to generate telemetry data.", file=sys.stderr)
        sys.exit(1)
    conn = sqlite3.connect(str(DB_PATH))
    conn.row_factory = sqlite3.Row
    return conn


def _truncate(text: str, max_len: int = 2000) -> str:
    if not text:
        return ""
    if len(text) <= max_len:
        return text
    return text[:max_len] + f"\n...[+{len(text) - max_len} chars truncated]"


def list_sessions(conn: sqlite3.Connection) -> None:
    rows = conn.execute(
        """SELECT id, started_at, ended_at, agent_type, agent_version,
                  git_branch, query_count, tool_call_count, duration_seconds,
                  exit_reason
           FROM sessions ORDER BY started_at DESC LIMIT 50"""
    ).fetchall()
    if not rows:
        print("No sessions recorded yet.")
        return
    print(f"{'ID':<38} {'started':<21} {'agent':<12} {'branch':<10} {'q':>4} {'tools':>6} {'dur':>6} {'exit':<10}")
    print("─" * 120)
    for r in rows:
        dur = f"{r['duration_seconds']}s" if r['duration_seconds'] else "?"
        print(
            f"{r['id']:<38} {r['started_at']:<21} {r['agent_type']:<12} "
            f"{(r['git_branch'] or '?'):<10} {r['query_count']:>4} "
            f"{r['tool_call_count']:>6} {dur:>6} {r['exit_reason'] or '?':<10}"
        )


def show_stats(conn: sqlite3.Connection) -> None:
    print("═" * 70)
    print("  Swarm-OS Telemetry Aggregate Stats")
    print("═" * 70)

    # Session stats
    row = conn.execute(
        """SELECT COUNT(*) AS total,
                  COUNT(DISTINCT agent_type) AS agent_types,
                  SUM(query_count) AS total_queries,
                  SUM(tool_call_count) AS total_tools,
                  AVG(duration_seconds) AS avg_duration
           FROM sessions"""
    ).fetchone()
    print(f"  Sessions:           {row['total']}")
    print(f"  Agent types:        {row['agent_types']}")
    print(f"  Total queries:      {row['total_queries']}")
    print(f"  Total tool calls:   {row['total_tools']}")
    print(f"  Avg session length: {row['avg_duration']:.0f}s" if row['avg_duration'] else "  Avg session length: ?")

    # Query stats
    row = conn.execute(
        """SELECT COUNT(*) AS total,
                  AVG(prompt_tokens_estimate) AS avg_prompt_tokens,
                  AVG(response_duration_ms) AS avg_response_ms
           FROM user_queries"""
    ).fetchone()
    print(f"  Queries logged:     {row['total']}")
    if row['avg_prompt_tokens']:
        print(f"  Avg prompt tokens:  {row['avg_prompt_tokens']:.0f}")
    if row['avg_response_ms']:
        print(f"  Avg response time:  {row['avg_response_ms']:.0f}ms")

    # Tool call stats
    print()
    print("  Tool call breakdown:")
    rows = conn.execute(
        """SELECT tool_name, COUNT(*) AS count,
                  SUM(CASE WHEN success=1 THEN 1 ELSE 0 END) AS success,
                  SUM(CASE WHEN success=0 THEN 1 ELSE 0 END) AS failed,
                  AVG(duration_ms) AS avg_ms
           FROM tool_calls GROUP BY tool_name ORDER BY count DESC"""
    ).fetchall()
    if rows:
        print(f"    {'tool':<15} {'count':>6} {'ok':>6} {'fail':>6} {'avg_ms':>8}")
        print("    " + "─" * 50)
        for r in rows:
            avg = f"{r['avg_ms']:.0f}" if r['avg_ms'] else "?"
            print(f"    {r['tool_name']:<15} {r['count']:>6} {r['success']:>6} {r['failed']:>6} {avg:>8}")
    else:
        print("    (no tool calls recorded)")

    # Reasoning stats
    print()
    print("  AI reasoning breakdown:")
    rows = conn.execute(
        """SELECT reasoning_type, COUNT(*) AS count
           FROM ai_reasoning GROUP BY reasoning_type ORDER BY count DESC"""
    ).fetchall()
    if rows:
        for r in rows:
            print(f"    {r['reasoning_type']:<20} {r['count']:>6}")
    else:
        print("    (no reasoning steps recorded)")

    # Build metrics
    print()
    print("  Build metrics (last 10):")
    rows = conn.execute(
        """SELECT timestamp, stack, test_total, test_passed, test_failed,
                  compile_time_ms, lint_errors
           FROM build_metrics ORDER BY timestamp DESC LIMIT 10"""
    ).fetchall()
    if rows:
        print(f"    {'timestamp':<21} {'stack':<8} {'total':>6} {'pass':>6} {'fail':>6} {'compile_ms':>11} {'lint_err':>9}")
        print("    " + "─" * 80)
        for r in rows:
            print(f"    {r['timestamp']:<21} {r['stack']:<8} {r['test_total']:>6} {r['test_passed']:>6} {r['test_failed']:>6} {r['compile_time_ms'] or 0:>11} {r['lint_errors'] or 0:>9}")
    else:
        print("    (no build metrics recorded)")

    print("═" * 70)


def replay_session(conn: sqlite3.Connection, session_id: str) -> None:
    session = conn.execute(
        "SELECT * FROM sessions WHERE id = ?", (session_id,)
    ).fetchone()
    if not session:
        print(f"Session {session_id} not found.", file=sys.stderr)
        sys.exit(1)

    print("═" * 80)
    print(f"  Session Replay: {session_id}")
    print("═" * 80)
    print(f"  Agent:       {session['agent_type']} {session['agent_version'] or ''}")
    print(f"  Started:     {session['started_at']}")
    print(f"  Ended:       {session['ended_at'] or '(still running)'}")
    print(f"  Duration:    {session['duration_seconds'] or '?'}s")
    print(f"  Branch:      {session['git_branch']}")
    print(f"  HEAD at start: {session['git_head_sha']}")
    print(f"  Queries:     {session['query_count']}")
    print(f"  Tool calls:  {session['tool_call_count']}")
    print(f"  Files changed: {session['files_changed']}  (+{session['lines_added']}/-{session['lines_removed']})")
    print(f"  Exit reason: {session['exit_reason']}")
    print("═" * 80)
    print()

    # Get all queries for this session
    queries = conn.execute(
        "SELECT * FROM user_queries WHERE session_id = ? ORDER BY query_number",
        (session_id,),
    ).fetchall()

    for q in queries:
        print(f"┌─ Query #{q['query_number']} @ {q['timestamp']} ─────────────────────────────")
        print(f"│ Prompt ({q['prompt_tokens_estimate']} tokens):")
        for line in _truncate(q['prompt'], 5000).splitlines():
            print(f"│   {line}")
        print("│")
        print(f"│ Response: {q['response_duration_ms'] or '?'}ms, ~{q['response_tokens_estimate'] or '?'} tokens, {q['tool_calls_invoked']} tool calls, {q['files_modified']} files modified")

        # Reasoning steps
        reasoning = conn.execute(
            """SELECT * FROM ai_reasoning
               WHERE session_id = ? AND query_id = ?
               ORDER BY step_number""",
            (session_id, q['id']),
        ).fetchall()
        if reasoning:
            print("│")
            print(f"│ AI reasoning ({len(reasoning)} steps):")
            for r in reasoning:
                print(f"│   [{r['step_number']}] {r['reasoning_type']} ({r['duration_ms'] or '?'}ms):")
                for line in _truncate(r['content'], 800).splitlines():
                    print(f"│       {line}")

        # Tool calls
        tools = conn.execute(
            """SELECT tc.*, (
                   SELECT GROUP_CONCAT(
                     output_type || ': ' || substr(content,1,500),
                     char(10)
                   )
                   FROM tool_outputs WHERE tool_call_id = tc.id
               ) AS outputs
               FROM tool_calls tc
               WHERE tc.session_id = ? AND tc.query_id = ?
               ORDER BY call_number""",
            (session_id, q['id']),
        ).fetchall()
        if tools:
            print("│")
            print(f"│ Tool calls ({len(tools)}):")
            for t in tools:
                status = "OK" if t['success'] else f"FAIL: {t['error_message']}"
                print(f"│   [{t['call_number']}] {t['tool_name']} ({t['duration_ms'] or '?'}ms) — {status}")
                if t['tool_input']:
                    try:
                        inp = json.loads(t['tool_input'])
                        for k, v in inp.items():
                            v_str = str(v)
                            if len(v_str) > 200:
                                v_str = v_str[:200] + "..."
                            print(f"│       input.{k}: {v_str}")
                    except Exception:
                        print(f"│       input: {t['tool_input'][:200]}")
                if t['outputs']:
                    print("│       output:")
                    for line in _truncate(t['outputs'], 1500).splitlines():
                        print(f"│         {line}")
        print(f"└{'─' * 78}")
        print()

    # Session summary
    summary = conn.execute(
        "SELECT * FROM session_summaries WHERE session_id = ?", (session_id,)
    ).fetchone()
    if summary:
        print("═" * 80)
        print("  Session Summary")
        print("═" * 80)
        print(f"  {summary['summary']}")
        print(f"  Tasks: {summary['tasks_completed']} completed, {summary['tasks_failed']} failed")
        print(f"  Tests: {summary['tests_passed']}/{summary['tests_run']} passed")
        print(f"  Commits: {summary['commits_made']}")
        print("═" * 80)


def main() -> int:
    if len(sys.argv) < 2:
        # Default: replay most recent session
        conn = _connect()
        row = conn.execute(
            "SELECT id FROM sessions ORDER BY started_at DESC LIMIT 1"
        ).fetchone()
        if not row:
            print("No sessions recorded yet.")
            return 1
        replay_session(conn, row['id'])
        return 0

    arg = sys.argv[1]
    conn = _connect()

    if arg == "--list":
        list_sessions(conn)
        return 0

    if arg == "--stats":
        show_stats(conn)
        return 0

    # Otherwise treat as session_id
    replay_session(conn, arg)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
