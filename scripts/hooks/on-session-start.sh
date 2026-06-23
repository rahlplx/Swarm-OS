#!/usr/bin/env bash
# SessionStart hook — fires when Claude Code opens the repo.
# Emits the bootstrap context banner so the agent loads memory + rules + governance
# before responding to the first user prompt.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"
cd "$PROJECT_DIR"

# Only emit if the key context files exist
if [ ! -f ".claude/memory/PROJECT.md" ]; then
  exit 0
fi

cat <<'BANNER'
────────────────────────────────────────────────────────────────────────
Swarm-OS session start. BEFORE responding to the user, you MUST:
1. Read .claude/memory/PROJECT.md           (persistent state + decisions)
2. Read .claude/rules/INVARIANTS.md         (hard rules — never violate)
3. Read .claude/rules/SESSION_PROTOCOL.md   (session bootstrap checklist)
4. Read CLAUDE.md (Claude Code) or AGENTS.md (other agents)
5. Read log.md for recent changes
Then obey governance.md, architecture.md §7, tech_stack.md license constraints.
────────────────────────────────────────────────────────────────────────
BANNER

# Print current state snapshot from PROJECT.md (first 30 lines for quick context)
echo "── Current state snapshot (from .claude/memory/PROJECT.md) ──"
head -30 .claude/memory/PROJECT.md 2>/dev/null || true
echo ""
echo "── Recent log entries (from log.md, last 10 lines of newest section) ──"
awk '/^## /{section=$0; lines=""} {lines=lines $0 "\n"} END{print lines}' log.md 2>/dev/null | tail -20 || true
echo ""
echo "── Git state ──"
git log --oneline -3 2>/dev/null || true
git branch --show-current 2>/dev/null || echo "(detached HEAD)"
echo ""
echo "Bootstrap complete. Awaiting first prompt."
