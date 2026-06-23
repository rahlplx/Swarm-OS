#!/usr/bin/env python3
"""
Swarm-OS Telemetry Export — dump full session(s) as JSON for external analysis.

Usage:
    python3 scripts/session-export.py <session_id>           # one session
    python3 scripts/session-export.py --all                  # all sessions
    python3 scripts/session-export.py --recent <N>           # last N sessions
    python3 scripts/session-export.py <session_id> -o out.json   # write to file
"""

from __future__ import annotations

import json
import sqlite3
import sys
from datetime import datetime, timezone
from pathlib import Path


def _utc_now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

DB_PATH = Path(__file__).resolve().parent.parent / ".swarm-os" / "telemetry.db"


def _connect() -> sqlite3.Connection:
    if not DB_PATH.exists():
        print(f"Telemetry DB not found at {DB_PATH}", file=sys.stderr)
        sys.exit(1)
    conn = sqlite3.connect(str(DB_PATH))
    conn.row_factory = sqlite3.Row
    return conn


def _row_to_dict(row):
    if row is None:
        return None
    return {k: row[k] for k in row.keys()}


def _export_session(conn: sqlite3.Connection, session_id: str) -> dict:
    session = _row_to_dict(
        conn.execute("SELECT * FROM sessions WHERE id = ?", (session_id,)).fetchone()
    )
    if not session:
        print(f"Session {session_id} not found.", file=sys.stderr)
        sys.exit(1)

    queries = []
    for q in conn.execute(
        "SELECT * FROM user_queries WHERE session_id = ? ORDER BY query_number",
        (session_id,),
    ).fetchall():
        q_dict = _row_to_dict(q)
        q_dict["reasoning"] = [
            _row_to_dict(r)
            for r in conn.execute(
                "SELECT * FROM ai_reasoning WHERE query_id = ? ORDER BY step_number",
                (q["id"],),
            ).fetchall()
        ]
        q_dict["tool_calls"] = []
        for tc in conn.execute(
            "SELECT * FROM tool_calls WHERE query_id = ? ORDER BY call_number",
            (q["id"],),
        ).fetchall():
            tc_dict = _row_to_dict(tc)
            tc_dict["outputs"] = [
                _row_to_dict(o)
                for o in conn.execute(
                    "SELECT * FROM tool_outputs WHERE tool_call_id = ?",
                    (tc["id"],),
                ).fetchall()
            ]
            if tc_dict.get("tool_input"):
                try:
                    tc_dict["tool_input"] = json.loads(tc_dict["tool_input"])
                except Exception:
                    pass
            q_dict["tool_calls"].append(tc_dict)
        queries.append(q_dict)

    summary = _row_to_dict(
        conn.execute(
            "SELECT * FROM session_summaries WHERE session_id = ?",
            (session_id,),
        ).fetchone()
    )

    return {
        "session": session,
        "queries": queries,
        "summary": summary,
        "exported_at": _utc_now_iso(),
        "schema_version": "2",
    }


def main() -> int:
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        return 1

    conn = _connect()
    output_file = None
    if "-o" in args:
        idx = args.index("-o")
        output_file = args[idx + 1]
        args = args[:idx] + args[idx + 2:]

    if args[0] == "--all":
        session_ids = [
            r["id"]
            for r in conn.execute(
                "SELECT id FROM sessions ORDER BY started_at"
            ).fetchall()
        ]
        export = {
            "sessions": [_export_session(conn, sid) for sid in session_ids],
            "exported_at": _utc_now_iso(),
            "schema_version": "2",
        }
    elif args[0] == "--recent":
        n = int(args[1]) if len(args) > 1 else 5
        session_ids = [
            r["id"]
            for r in conn.execute(
                "SELECT id FROM sessions ORDER BY started_at DESC LIMIT ?", (n,)
            ).fetchall()
        ]
        export = {
            "sessions": [_export_session(conn, sid) for sid in session_ids],
            "exported_at": _utc_now_iso(),
            "schema_version": "2",
        }
    else:
        export = _export_session(conn, args[0])

    json_str = json.dumps(export, indent=2, default=str)

    if output_file:
        Path(output_file).write_text(json_str)
        print(f"Exported to {output_file} ({len(json_str)} bytes)", file=sys.stderr)
    else:
        print(json_str)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
