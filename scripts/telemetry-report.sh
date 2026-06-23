#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DB="$PROJECT_DIR/.swarm-os/telemetry.db"

if [ ! -f "$DB" ]; then
  echo "No telemetry data yet. Run scripts/telemetry.sh first."
  exit 0
fi

echo "╔══════════════════════════════════════╗"
echo "║     Swarm-OS Telemetry Report        ║"
echo "╠══════════════════════════════════════╣"

days_completed=$(sqlite3 "$DB" "SELECT COUNT(*) FROM phase_progress WHERE status = 'completed';")
days_total=$(sqlite3 "$DB" "SELECT COUNT(*) FROM phase_progress;")
echo "║ Phase 0: Day $days_completed/$days_total completed"

latest=$(sqlite3 "$DB" "SELECT test_passed, test_failed, test_skipped, compile_time_ms, binary_size_bytes FROM build_metrics ORDER BY id DESC LIMIT 1;" 2>/dev/null || echo "0|0|0|0|0")
IFS='|' read -r passed failed skipped build_ms binary_size <<< "$latest"

echo "║ Tests: $passed passing, $failed failing, $skipped skipped"
if [ "$binary_size" -gt 0 ] 2>/dev/null; then
  binary_mib=$(awk "BEGIN {printf \"%.1f\", $binary_size / 1048576}" 2>/dev/null || echo "?")
  echo "║ Binary: ${binary_mib} MiB"
fi
echo "║ Last build: ${build_ms}ms"

regressions=$(sqlite3 "$DB" "SELECT COALESCE(SUM(regression_count), 0) FROM quality_metrics;" 2>/dev/null || echo 0)
echo "║ Regressions: $regressions"

echo "╚══════════════════════════════════════╝"
