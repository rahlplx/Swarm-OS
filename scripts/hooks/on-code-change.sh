#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"
cd "$PROJECT_DIR"

changed_files=$(git diff --name-only HEAD 2>/dev/null || echo "")

if echo "$changed_files" | grep -qE '\.rs$'; then
  cargo check --workspace 2>/dev/null && echo "Rust: OK" || echo "Rust: type error"
fi

if echo "$changed_files" | grep -qE '\.(ts|tsx)$'; then
  if [ -f tsconfig.json ]; then
    pnpm exec tsc --noEmit 2>/dev/null && echo "TypeScript: OK" || echo "TypeScript: type error"
  fi
fi

if echo "$changed_files" | grep -qE '\.py$'; then
  if [ -d litellm-proxy ]; then
    cd litellm-proxy
    python3 -m ruff check . 2>/dev/null && echo "Python: OK" || echo "Python: lint error"
    cd ..
  fi
fi
