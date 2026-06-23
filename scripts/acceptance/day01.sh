#!/usr/bin/env bash
set -uo pipefail

echo "Day 1 Acceptance: Project scaffold"
pass=0
fail=0

check() {
  if eval "$2" >/dev/null 2>&1; then
    echo "  ✓ $1"
    pass=$((pass + 1))
  else
    echo "  ✗ $1"
    fail=$((fail + 1))
  fi
}

check "cargo available"       "cargo --version"
check "pnpm available"        "pnpm --version"
check "cmake available"       "cmake --version"
check "Cargo.toml exists"     "test -f Cargo.toml"
check "src-tauri/Cargo.toml"  "test -f src-tauri/Cargo.toml"
check "rust-toolchain.toml"   "test -f rust-toolchain.toml"
check "package.json exists"   "test -f package.json"
check "tsconfig.json exists"  "test -f tsconfig.json"
check "cargo check passes"    "cargo check --workspace"

echo ""
echo "Day 1: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
