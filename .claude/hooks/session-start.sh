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
    if [ -f "$HOME/.cargo/bin/cargo" ]; then
        # Already installed but not on PATH (common in new remote sessions)
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    else
        echo "[swarm-os] Installing Rust (minimal profile)..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --quiet
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    fi
fi

# rust-toolchain.toml pins the channel; rustup show triggers the install
if [ -f "$PROJECT_DIR/rust-toolchain.toml" ]; then
    if command -v rustup &>/dev/null; then
        rustup show active-toolchain &>/dev/null || rustup show
    else
        echo "[swarm-os] rust-toolchain.toml detected, but rustup unavailable; skipping."
    fi
fi

# ── 2. Python deps (gateway) — use venv to comply with PEP 668 ────────────────
if [ -f "$PROJECT_DIR/gateway/pyproject.toml" ] && command -v python3 &>/dev/null; then
    VENV_DIR="$PROJECT_DIR/gateway/.venv"
    if [ ! -d "$VENV_DIR" ]; then
        echo "[swarm-os] Creating Python virtual environment..."
        python3 -m venv "$VENV_DIR"
    fi
    # shellcheck source=/dev/null
    source "$VENV_DIR/bin/activate"
    echo "[swarm-os] Installing Python dependencies..."
    if command -v uv &>/dev/null; then
        uv pip install -e "$PROJECT_DIR/gateway[test]" --quiet
    else
        pip3 install -e "$PROJECT_DIR/gateway[test]" --quiet
    fi
fi

# ── 3. Rust tests ─────────────────────────────────────────────────────────────
# Only test node-agent; src-tauri requires GTK libs unavailable in headless CI
if [ -f "$PROJECT_DIR/Cargo.toml" ]; then
    echo "[swarm-os] Running Rust tests (node-agent)..."
    cargo test --manifest-path "$PROJECT_DIR/Cargo.toml" -p node-agent --quiet 2>&1
fi

# ── 4. Python tests ───────────────────────────────────────────────────────────
if [ -f "$PROJECT_DIR/gateway/pyproject.toml" ] && [ -f "$PROJECT_DIR/gateway/.venv/bin/activate" ]; then
    echo "[swarm-os] Running Python tests..."
    # shellcheck source=/dev/null
    source "$PROJECT_DIR/gateway/.venv/bin/activate"
    cd "$PROJECT_DIR/gateway"
    python3 -m pytest tests/ -q 2>&1
fi

echo "[swarm-os] Session start complete."
