#!/usr/bin/env bash
# Swarm-OS Telemetry Pipeline v2
# Captures build metrics, test results, and (via hooks) full session telemetry.
# Usage: bash scripts/telemetry.sh [rust|react|python|all]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

STACK="${1:-all}"

# Ensure DB + schema exist
python3 scripts/telemetry_collector.py > /dev/null 2>&1 || true

# ── Run tests per stack and capture metrics ──────────────────────────────────

test_total=0
test_passed=0
test_failed=0
lint_errors=0
compile_time_ms=0

if [ "$STACK" = "rust" ] || [ "$STACK" = "all" ]; then
  echo "── Rust: cargo test ──"
  start_ms=$(date +%s%3N 2>/dev/null || date +%s)
  test_output=$(cargo test --workspace 2>&1 || true)
  if echo "$test_output" | grep -q "test result:"; then
    test_result=$(echo "$test_output" | grep "test result:" | tail -1)
    test_passed=$(echo "$test_result" | grep -oP '\d+ passed' | grep -oP '\d+' || echo 0)
    test_failed=$(echo "$test_result" | grep -oP '\d+ failed' | grep -oP '\d+' || echo 0)
    test_total=$((test_passed + test_failed))
  fi
  end_ms=$(date +%s%3N 2>/dev/null || date +%s)
  compile_time_ms=$((end_ms - start_ms))

  # Lint errors via clippy (non-blocking — may not be installable)
  lint_errors=0
  if cargo clippy --workspace -- -D warnings 2>/dev/null; then
    lint_errors=0
  else
    lint_errors=$((lint_errors + 1))
  fi

  # Record via Python collector
  python3 -c "
import sys; sys.path.insert(0, 'scripts')
from telemetry_collector import TelemetryCollector
tc = TelemetryCollector()
tc.log_build_metrics(
    stack='rust',
    compile_time_ms=$compile_time_ms,
    test_total=$test_total,
    test_passed=$test_passed,
    test_failed=$test_failed,
    lint_errors=$lint_errors,
)
print(f'  rust: $test_passed/$test_total tests, ${compile_time_ms}ms, $lint_errors lint errors')
"
fi

if [ "$STACK" = "react" ] || [ "$STACK" = "all" ]; then
  echo "── React: pnpm test ──"
  start_ms=$(date +%s%3N 2>/dev/null || date +%s)
  test_output=$(pnpm test 2>&1 || true)
  if echo "$test_output" | grep -q "Tests"; then
    test_passed=$(echo "$test_output" | grep -oP '\d+ passed' | grep -oP '\d+' | tail -1 || echo 0)
    test_failed=$(echo "$test_output" | grep -oP '\d+ failed' | grep -oP '\d+' | tail -1 || echo 0)
    test_total=$((test_passed + test_failed))
  fi
  end_ms=$(date +%s%3N 2>/dev/null || date +%s)
  compile_time_ms=$((end_ms - start_ms))

  lint_errors=0
  pnpm lint > /dev/null 2>&1 || lint_errors=$((lint_errors + 1))

  python3 -c "
import sys; sys.path.insert(0, 'scripts')
from telemetry_collector import TelemetryCollector
tc = TelemetryCollector()
tc.log_build_metrics(
    stack='react',
    compile_time_ms=$compile_time_ms,
    test_total=$test_total,
    test_passed=$test_passed,
    test_failed=$test_failed,
    lint_errors=$lint_errors,
)
print(f'  react: $test_passed/$test_total tests, ${compile_time_ms}ms, $lint_errors lint errors')
"
fi

if [ "$STACK" = "python" ] || [ "$STACK" = "all" ]; then
  echo "── Python: pytest ──"
  start_ms=$(date +%s%3N 2>/dev/null || date +%s)
  test_output=$(cd litellm-proxy && python3 -m pytest tests/ -v 2>&1 || true)
  if echo "$test_output" | grep -q "passed"; then
    test_passed=$(echo "$test_output" | grep -oP '\d+ passed' | grep -oP '\d+' | tail -1 || echo 0)
    test_failed=$(echo "$test_output" | grep -oP '\d+ failed' | grep -oP '\d+' | tail -1 || echo 0)
    test_total=$((test_passed + test_failed))
  fi
  end_ms=$(date +%s%3N 2>/dev/null || date +%s)
  compile_time_ms=$((end_ms - start_ms))

  lint_errors=0
  (cd litellm-proxy && ruff check . > /dev/null 2>&1) || lint_errors=$((lint_errors + 1))

  python3 -c "
import sys; sys.path.insert(0, 'scripts')
from telemetry_collector import TelemetryCollector
tc = TelemetryCollector()
tc.log_build_metrics(
    stack='python',
    compile_time_ms=$compile_time_ms,
    test_total=$test_total,
    test_passed=$test_passed,
    test_failed=$test_failed,
    lint_errors=$lint_errors,
)
print(f'  python: $test_passed/$test_total tests, ${compile_time_ms}ms, $lint_errors lint errors')
"
fi

# Binary size (if built)
binary_size=0
if [ -f "$PROJECT_DIR/target/release/swarm-os" ]; then
  binary_size=$(wc -c < "$PROJECT_DIR/target/release/swarm-os" 2>/dev/null | tr -d ' ' || echo 0)
fi

echo ""
echo "Telemetry recorded. Run: python3 scripts/session-replay.py --stats"
