#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"
cd "$PROJECT_DIR"

echo "Session end: running full test suite..."

echo "  Rust tests..."
cargo test --workspace 2>&1 | tail -3

if [ -f vitest.config.ts ]; then
  echo "  React tests..."
  pnpm test 2>&1 | tail -3
fi

if [ -d litellm-proxy ]; then
  echo "  Python tests..."
  cd litellm-proxy
  python3 -m pytest tests/ -q 2>&1 | tail -3
  cd ..
fi

echo "  Collecting telemetry..."
bash scripts/telemetry.sh all 2>/dev/null || true

echo "  Telemetry report:"
bash scripts/telemetry-report.sh 2>/dev/null || true
