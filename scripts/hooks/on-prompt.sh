#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"
COUNTER_FILE="$PROJECT_DIR/.swarm-os/query-count"

mkdir -p "$PROJECT_DIR/.swarm-os"

count=0
if [ -f "$COUNTER_FILE" ]; then
  count=$(cat "$COUNTER_FILE")
fi
count=$((count + 1))
echo "$count" > "$COUNTER_FILE"

if [ $((count % 5)) -eq 0 ]; then
  echo "[$count queries] Running checkpoint..."
  cd "$PROJECT_DIR"
  cargo check --workspace 2>/dev/null && echo "  cargo check: OK" || echo "  cargo check: FAIL"
  if [ -f tsconfig.json ]; then
    pnpm exec tsc --noEmit 2>/dev/null && echo "  tsc: OK" || echo "  tsc: FAIL"
  fi
fi
