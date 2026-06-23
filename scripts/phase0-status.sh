#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DB="$PROJECT_DIR/.swarm-os/telemetry.db"

if [ ! -f "$DB" ]; then
  echo "Phase 0: Day 1/20 (not started)"
  exit 0
fi

current_day=$(sqlite3 "$DB" "SELECT COALESCE(MAX(day_number), 0) FROM phase_progress WHERE status IN ('in_progress', 'completed');")
if [ "$current_day" -eq 0 ]; then
  current_day=1
fi

completed=$(sqlite3 "$DB" "SELECT COUNT(*) FROM phase_progress WHERE status = 'completed';")
pct=$((completed * 100 / 20))

echo "Phase 0 Progress: Day $current_day/20 ($pct%)"

echo ""
echo "Day status:"
sqlite3 "$DB" -header -column "SELECT day_number as Day, status as Status, acceptance_tests_passing || '/' || acceptance_tests_total as 'Tests' FROM phase_progress WHERE day_number <= $((current_day + 2)) ORDER BY day_number;"
