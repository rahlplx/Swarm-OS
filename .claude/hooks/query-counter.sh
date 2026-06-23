#!/bin/bash
# Auto-run cargo check every N queries to surface compile errors early.
# Resets counter on trigger; output appears in Claude Code's hook output panel.
set -euo pipefail

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
_proj_hash=$(printf '%s' "$PROJECT_DIR" | md5sum | cut -c1-8)
COUNTER_FILE="/tmp/swarm-os-query-count-${_proj_hash}"
TRIGGER_EVERY=5

# Sanitize: strip non-digits so corrupted file doesn't crash arithmetic expansion
count=$(cat "$COUNTER_FILE" 2>/dev/null | tr -cd '0-9')
count=$(( ${count:-0} + 1 ))

if [ "$count" -ge "$TRIGGER_EVERY" ]; then
    printf '0' > "$COUNTER_FILE"
    # Source cargo env if cargo is installed but not yet on PATH
    if [ -f "$HOME/.cargo/env" ] && ! command -v cargo &>/dev/null; then
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    fi
    if [ -f "$PROJECT_DIR/Cargo.toml" ] && command -v cargo &>/dev/null; then
        echo "[swarm-os] Auto-check (every ${TRIGGER_EVERY} queries):"
        cargo check --manifest-path "$PROJECT_DIR/Cargo.toml" -p node-agent --quiet 2>&1 | head -30 || true
    fi
else
    printf '%d' "$count" > "$COUNTER_FILE"
fi
