#!/bin/bash
# Auto-run cargo check every N queries to surface compile errors early.
# Resets counter on trigger; output appears in Claude Code's hook output panel.
set -euo pipefail

COUNTER_FILE="/tmp/swarm-os-query-count"
TRIGGER_EVERY=5

count=$(cat "$COUNTER_FILE" 2>/dev/null || echo 0)
count=$((count + 1))

if [ "$count" -ge "$TRIGGER_EVERY" ]; then
    printf '0' > "$COUNTER_FILE"
    PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
    if [ -f "$PROJECT_DIR/Cargo.toml" ] && command -v cargo &>/dev/null; then
        echo "[swarm-os] Auto-check (every ${TRIGGER_EVERY} queries):"
        cargo check --manifest-path "$PROJECT_DIR/Cargo.toml" --quiet 2>&1 | head -30 || true
    fi
else
    printf '%d' "$count" > "$COUNTER_FILE"
fi
