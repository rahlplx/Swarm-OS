#!/bin/bash
# Swarm-OS Phase 0 session start hook.
# Installs deps and runs the TDD test harness on every remote Claude Code session start.
set -euo pipefail

echo '{"async": true, "asyncTimeout": 300000}'

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
    exit 0
fi

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"

echo "[swarm-os] Session start — installing deps and running tests..."

# ── 1. Rust toolchain ──────────────────────────────────────────────────────────
if ! command -v cargo &>/dev/null; then
    echo "[swarm-os] Installing Rust (minimal profile)..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --quiet
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
fi

# rust-toolchain.toml pins the channel; rustup show triggers the install
if [ -f "$PROJECT_DIR/rust-toolchain.toml" ]; then
    rustup show active-toolchain &>/dev/null || rustup show
fi

# ── 2. Python deps (gateway) ──────────────────────────────────────────────────
if [ -f "$PROJECT_DIR/gateway/pyproject.toml" ]; then
    if command -v uv &>/dev/null; then
        uv pip install -e "$PROJECT_DIR/gateway[test]" --quiet
    elif command -v pip3 &>/dev/null; then
        pip3 install -e "$PROJECT_DIR/gateway[test]" --quiet
    fi
fi

# ── 3. Rust tests ─────────────────────────────────────────────────────────────
if [ -f "$PROJECT_DIR/Cargo.toml" ]; then
    echo "[swarm-os] Running Rust tests..."
    cargo test --manifest-path "$PROJECT_DIR/Cargo.toml" --quiet 2>&1
fi

# ── 4. Python tests ───────────────────────────────────────────────────────────
if [ -f "$PROJECT_DIR/gateway/pyproject.toml" ] && command -v python3 &>/dev/null; then
    echo "[swarm-os] Running Python tests..."
    cd "$PROJECT_DIR/gateway"
    python3 -m pytest tests/ -q 2>&1
fi

echo "[swarm-os] Session start complete."
