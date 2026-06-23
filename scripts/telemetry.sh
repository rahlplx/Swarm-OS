#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DB="$PROJECT_DIR/.swarm-os/telemetry.db"
STACK="${1:-all}"

mkdir -p "$PROJECT_DIR/.swarm-os"

if [ ! -f "$DB" ]; then
  sqlite3 "$DB" < "$SCRIPT_DIR/schema.sql"
fi

test_total=0
test_passed=0
test_failed=0
lint_errors=0
compile_time_ms=0

if [ "$STACK" = "rust" ] || [ "$STACK" = "all" ]; then
  start_ms=$(date +%s%3N 2>/dev/null || date +%s)
  if cargo test --workspace --quiet 2>/dev/null; then
    test_result=$(cargo test --workspace 2>&1 | tail -1)
    test_passed=$(echo "$test_result" | grep -oP '\d+ passed' | grep -oP '\d+' || echo 0)
    test_failed=$(echo "$test_result" | grep -oP '\d+ failed' | grep -oP '\d+' || echo 0)
    test_total=$((test_passed + test_failed))
  fi
  end_ms=$(date +%s%3N 2>/dev/null || date +%s)
  compile_time_ms=$((end_ms - start_ms))
fi

binary_size=0
if [ -f "$PROJECT_DIR/target/release/swarm-os" ]; then
  binary_size=$(stat -c%s "$PROJECT_DIR/target/release/swarm-os" 2>/dev/null || echo 0)
fi

sqlite3 "$DB" "INSERT INTO build_metrics (compile_time_ms, test_total, test_passed, test_failed, binary_size_bytes, lint_errors, stack) VALUES ($compile_time_ms, $test_total, $test_passed, $test_failed, $binary_size, $lint_errors, '$STACK');"

echo "Telemetry recorded: $test_passed/$test_total tests passed, ${compile_time_ms}ms"
