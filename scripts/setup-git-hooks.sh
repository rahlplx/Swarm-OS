#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
HOOKS_DIR="$PROJECT_DIR/.git/hooks"

mkdir -p "$HOOKS_DIR"

cat > "$HOOKS_DIR/pre-commit" << 'HOOK'
#!/usr/bin/env bash
set -euo pipefail

echo "Running pre-commit checks..."

echo "  Rust: fmt"
cargo fmt --check 2>/dev/null || { echo "cargo fmt failed"; exit 1; }

echo "  Rust: clippy"
cargo clippy --workspace -- -D warnings 2>/dev/null || { echo "clippy failed"; exit 1; }

if command -v pnpm >/dev/null 2>&1 && [ -f package.json ]; then
  echo "  TypeScript: type-check"
  pnpm exec tsc --noEmit 2>/dev/null || { echo "tsc failed"; exit 1; }
fi

if [ -d litellm-proxy ]; then
  echo "  Python: ruff"
  cd litellm-proxy
  ruff check . 2>/dev/null || python3 -m ruff check . 2>/dev/null || { echo "ruff failed"; exit 1; }
  cd ..
fi

echo "Pre-commit checks passed."
HOOK
chmod +x "$HOOKS_DIR/pre-commit"

cat > "$HOOKS_DIR/pre-push" << 'HOOK'
#!/usr/bin/env bash
set -euo pipefail

echo "Running pre-push test suite..."

echo "  Rust tests"
cargo test --workspace 2>/dev/null || { echo "Rust tests failed"; exit 1; }

if command -v pnpm >/dev/null 2>&1 && [ -f package.json ]; then
  echo "  React tests"
  pnpm test 2>/dev/null || { echo "React tests failed"; exit 1; }
fi

if [ -d litellm-proxy ]; then
  echo "  Python tests"
  cd litellm-proxy
  python3 -m pytest tests/ 2>/dev/null || { echo "Python tests failed"; exit 1; }
  cd ..
fi

# Binary size check
if [ -f target/release/swarm-os ]; then
  size=$(stat -c%s target/release/swarm-os 2>/dev/null || echo 0)
  max=$((10 * 1024 * 1024))
  if [ "$size" -gt "$max" ]; then
    echo "Binary too large: $(($size / 1048576)) MiB (max 10 MiB)"
    exit 1
  fi
fi

echo "Pre-push checks passed."
HOOK
chmod +x "$HOOKS_DIR/pre-push"

echo "Git hooks installed at $HOOKS_DIR"
