#!/usr/bin/env python3
"""
Swarm-OS Live Dashboard — real-time telemetry view.

Shows:
- Current session (if active) with live query/tool counts
- Cumulative stats across all sessions
- Tool call breakdown
- Recent activity feed (last 20 events)

Usage:
    python3 scripts/dashboard.py              # one-shot snapshot
    python3 scripts/dashboard.py --watch      # refresh every 2s
    python3 scripts/dashboard.py --watch 5    # refresh every 5s
"""

from __future__ import annotations

import os
import sqlite3
import sys
import time
from pathlib import Path

DB_PATH = Path(__file__).resolve().parent.parent / ".swarm-os" / "telemetry.db"
SESSION_FILE = DB_PATH.parent / "current-session"


def _connect() -> sqlite3.Connection:
    if not DB_PATH.exists():
        print("Telemetry DB not found. Run a session first.")
        sys.exit(1)
    conn = sqlite3.connect(str(DB_PATH))
    conn.row_factory = sqlite3.Row
    return conn


def _clear() -> None:
    os.system("clear" if os.name == "posix" else "cls")


def _box(title: str, lines: list, width: int = 70) -> list:
    top = "+" + "-" * (width - 2) + "+"
    bot = "+" + "-" * (width - 2) + "+"
    mid = "+" + "-" * (width - 2) + "+"
    out = [top]
    out.append(f"| {title:<{width - 4}} |")
    out.append(mid)
    for line in lines:
        out.append(f"| {line:<{width - 4}} |")
    out.append(bot)
    return out


def render(conn: sqlite3.Connection) -> None:
    _clear()
    width = 70

    print("=" * width)
    print("  Swarm-OS Telemetry Dashboard".center(width))
    print(
        "  {}".format(
            time.strftime("%Y-%m-%d %H:%M:%S UTC", time.gmtime())
        ).center(width)
    )
    print("=" * width)
    print()

    # Current session
    current_session_id = None
    if SESSION_FILE.exists():
        current_session_id = SESSION_FILE.read_text().strip()

    if current_session_id:
        row = conn.execute(
            "SELECT * FROM sessions WHERE id = ?", (current_session_id,)
        ).fetchone()
        if row:
            lines = [
                "Session ID:  {}".format(row["id"][:36]),
                "Started:     {}".format(row["started_at"]),
                "Agent:       {} {}".format(row["agent_type"], row["agent_version"] or ""),
                "Branch:      {}".format(row["git_branch"]),
                "Queries:     {}".format(row["query_count"]),
                "Tool calls:  {}".format(row["tool_call_count"]),
                "HEAD at start: {}".format((row["git_head_sha"] or "")[:12]),
            ]
            print("\n".join(_box("ACTIVE SESSION", lines, width)))
            print()
        else:
            print("  (active session file exists but session not found in DB)")
            print()
    else:
        print("  No active session.")
        print()

    # Cumulative stats
    row = conn.execute(
        """SELECT COUNT(*) AS total_sessions,
                  COALESCE(SUM(query_count), 0) AS total_queries,
                  COALESCE(SUM(tool_call_count), 0) AS total_tools,
                  COALESCE(AVG(duration_seconds), 0) AS avg_duration
           FROM sessions"""
    ).fetchone()
    avg_dur = "{:.0f}s".format(row["avg_duration"]) if row["avg_duration"] else "?"
    lines = [
        "Total sessions:    {}".format(row["total_sessions"]),
        "Total queries:     {}".format(row["total_queries"]),
        "Total tool calls:  {}".format(row["total_tools"]),
        "Avg session length: {}".format(avg_dur),
    ]
    print("\n".join(_box("CUMULATIVE STATS", lines, width)))
    print()

    # Tool call breakdown
    rows = conn.execute(
        """SELECT tool_name, COUNT(*) AS count,
                  SUM(CASE WHEN success=1 THEN 1 ELSE 0 END) AS ok,
                  SUM(CASE WHEN success=0 THEN 1 ELSE 0 END) AS fail,
                  COALESCE(AVG(duration_ms), 0) AS avg_ms
           FROM tool_calls GROUP BY tool_name ORDER BY count DESC LIMIT 10"""
    ).fetchall()
    if rows:
        lines = ["{:<14} {:>6} {:>6} {:>6} {:>8}".format("tool", "count", "ok", "fail", "avg_ms")]
        lines.append("-" * 50)
        for r in rows:
            avg = "{:.0f}".format(r["avg_ms"]) if r["avg_ms"] else "?"
            lines.append(
                "{:<14} {:>6} {:>6} {:>6} {:>8}".format(
                    r["tool_name"], r["count"], r["ok"], r["fail"], avg
                )
            )
        print("\n".join(_box("TOOL CALL BREAKDOWN", lines, width)))
        print()

    # Reasoning breakdown
    rows = conn.execute(
        """SELECT reasoning_type, COUNT(*) AS count
           FROM ai_reasoning GROUP BY reasoning_type ORDER BY count DESC"""
    ).fetchall()
    if rows:
        lines = ["{:<20} {:>6}".format(r["reasoning_type"], r["count"]) for r in rows]
        print("\n".join(_box("AI REASONING BREAKDOWN", lines, width)))
        print()

    # Recent activity feed
    print("+" + "-" * (width - 2) + "+")
    print("| {:<{w}} |".format("RECENT ACTIVITY (last 20 events)", w=width - 4))
    print("+" + "-" * (width - 2) + "+")
    events = []
    for r in conn.execute(
        """SELECT timestamp, 'query' AS type, prompt AS detail
           FROM user_queries
           UNION ALL
           SELECT timestamp, 'tool:' || tool_name AS type,
                  substr(tool_input, 1, 60) AS detail
           FROM tool_calls
           ORDER BY timestamp DESC LIMIT 20"""
    ).fetchall():
        events.append(r)
    if not events:
        print("| {:<{w}} |".format("(no events yet)", w=width - 4))
    else:
        for r in events:
            ts = r["timestamp"][:19] if r["timestamp"] else "?"
            detail = (r["detail"] or "")[:48]
            line = "{}  [{:<12}]  {}".format(ts, r["type"], detail)
            print("| {:<{w}} |".format(line, w=width - 4))
    print("+" + "-" * (width - 2) + "+")

    print()
    if "--watch" in sys.argv:
        print("Press Ctrl+C to exit")


def main() -> int:
    watch = "--watch" in sys.argv
    interval = 2
    if watch:
        try:
            idx = sys.argv.index("--watch")
            if idx + 1 < len(sys.argv):
                interval = int(sys.argv[idx + 1])
        except Exception:
            pass

    try:
        while True:
            conn = _connect()
            render(conn)
            conn.close()
            if not watch:
                break
            time.sleep(interval)
    except KeyboardInterrupt:
        print("\nDashboard stopped.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
