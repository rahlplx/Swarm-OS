#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

pass=0
fail=0

check() {
  local name="$1"
  local cmd="$2"
  local min_ver="${3:-}"

  if eval "$cmd" >/dev/null 2>&1; then
    local ver
    ver=$(eval "$cmd" 2>&1 | head -1)
    echo -e "  ${GREEN}✓${NC} $name: $ver"
    ((pass++))
  else
    echo -e "  ${RED}✗${NC} $name: NOT FOUND"
    ((fail++))
  fi
}

echo "Swarm-OS Environment Check"
echo "=========================="

echo ""
echo "Build Tools:"
check "cargo"   "cargo --version"
check "rustc"   "rustc --version"
check "cmake"   "cmake --version"
check "clang"   "clang --version"

echo ""
echo "Frontend:"
check "node"    "node --version"
check "pnpm"    "pnpm --version"

echo ""
echo "Python:"
check "python3" "python3 --version"

echo ""
echo "Database:"
check "sqlite3" "sqlite3 --version"

echo ""
echo "=========================="
echo -e "Results: ${GREEN}${pass} passed${NC}, ${RED}${fail} failed${NC}"

if [ "$fail" -gt 0 ]; then
  echo -e "${RED}Missing dependencies! Install them before proceeding.${NC}"
  exit 1
fi

echo -e "${GREEN}All dependencies satisfied.${NC}"
