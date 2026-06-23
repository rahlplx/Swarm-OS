#!/usr/bin/env bash
set -euo pipefail

echo "Starting Swarm-OS dev environment..."

cleanup() {
  echo "Shutting down..."
  kill 0 2>/dev/null
  wait 2>/dev/null
}
trap cleanup EXIT

if command -v cargo-watch >/dev/null 2>&1; then
  cargo watch -x 'check --workspace' -x 'test --workspace --lib' &
else
  echo "cargo-watch not found, skipping Rust watch mode"
fi

if command -v pnpm >/dev/null 2>&1 && [ -f vitest.config.ts ]; then
  pnpm exec vitest --watch &
fi

if command -v cargo >/dev/null 2>&1 && [ -f src-tauri/tauri.conf.json ]; then
  cargo tauri dev
else
  echo "Tauri not available, running Vite dev server only"
  pnpm dev
fi
