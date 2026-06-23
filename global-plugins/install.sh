#!/usr/bin/env bash
# Swarm-OS Global Plugins Installer
# Installs fablize + zen-router MCP server + token-efficiency layer globally.
# Run on your LOCAL machine (not in a remote Claude Code session).
#
# Usage:
#   OPENCODE_ZEN_API_KEY=sk-... bash install.sh
#
# What this does:
#   1. Installs fablize via Claude Code plugin marketplace (global mode)
#   2. Installs zen-router MCP server Python dependencies
#   3. Writes ~/.claude/settings.json (merges if exists)
#   4. Appends token-efficiency block to ~/.claude/CLAUDE.md

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLAUDE_DIR="$HOME/.claude"
ZEN_KEY="${OPENCODE_ZEN_API_KEY:-}"
STITCH_KEY="${STITCH_API_TOKEN:-}"

# ── Colours ───────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
info()  { echo -e "${GREEN}[✓]${NC} $*"; }
warn()  { echo -e "${YELLOW}[!]${NC} $*"; }
error() { echo -e "${RED}[✗]${NC} $*" >&2; exit 1; }

echo ""
echo "═══════════════════════════════════════════════════════"
echo "  Swarm-OS Global Plugins: fablize + zen-router"
echo "═══════════════════════════════════════════════════════"
echo ""

# ── Prerequisite checks ───────────────────────────────────────────────────────
command -v python3 >/dev/null || error "python3 is required"
command -v pip3   >/dev/null || error "pip3 is required"
command -v claude >/dev/null || error "Claude Code CLI (claude) is required"
command -v git    >/dev/null || error "git is required"

if [[ -z "$ZEN_KEY" ]]; then
    warn "OPENCODE_ZEN_API_KEY not set. zen-router will be installed but won't work until you set it."
    warn "Add to ~/.bashrc: export OPENCODE_ZEN_API_KEY=sk-..."
fi

if [[ -z "$STITCH_KEY" ]]; then
    warn "STITCH_API_TOKEN not set. stitch MCP will be installed but won't work until you set it."
    warn "Get your key: stitch.withgoogle.com → Settings → API Tokens"
    warn "Add to ~/.bashrc: export STITCH_API_TOKEN=AQ...."
fi

# ── 1. Install fablize ────────────────────────────────────────────────────────
info "Installing fablize plugin..."
PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT:-$HOME/.claude/plugins}"
mkdir -p "$PLUGIN_ROOT"

if [[ -d "$PLUGIN_ROOT/fablize" ]]; then
    info "fablize already cloned — pulling latest"
    git -C "$PLUGIN_ROOT/fablize" pull --ff-only 2>/dev/null || true
else
    git clone --depth=1 https://github.com/fivetaku/fablize.git "$PLUGIN_ROOT/fablize"
fi

# Run fablize global setup (auto-selects global, non-interactive)
if [[ -f "$PLUGIN_ROOT/fablize/setup/setup.sh" ]]; then
    info "Running fablize global setup..."
    # Pass 'g' to select global scope when prompted
    echo "g" | bash "$PLUGIN_ROOT/fablize/setup/setup.sh" || \
        warn "fablize setup returned non-zero — check output above"
else
    warn "fablize setup.sh not found at expected path — injecting block manually"
    # Fallback: inject a minimal always-on block
    CLAUDE_MD="$CLAUDE_DIR/CLAUDE.md"
    touch "$CLAUDE_MD"
    if ! grep -q "fablize" "$CLAUDE_MD" 2>/dev/null; then
        cat >> "$CLAUDE_MD" <<'FABLIZE_BLOCK'

<!-- fablize:start -->
## Fablize Operating Protocol
- Complete every task to a verified, executable state before reporting done.
- For code: run it or show test output. For docs: show the diff.
- Decompose multi-part tasks into goals; verify each goal before moving to the next.
- Never assert "this should work" — prove it.
<!-- fablize:end -->
FABLIZE_BLOCK
        info "Injected fablize block into $CLAUDE_MD"
    fi
fi

# ── 2. Install zen-router ─────────────────────────────────────────────────────
info "Installing zen-router MCP server..."
ZEN_DIR="$PLUGIN_ROOT/zen-router"
mkdir -p "$ZEN_DIR"
cp "$SCRIPT_DIR/zen-router/server.py" "$ZEN_DIR/server.py"
chmod +x "$ZEN_DIR/server.py"

info "Installing Python dependencies..."
pip3 install --quiet mcp httpx

# ── 3. Write settings.json ────────────────────────────────────────────────────
info "Configuring ~/.claude/settings.json..."
SETTINGS="$CLAUDE_DIR/settings.json"
mkdir -p "$CLAUDE_DIR"
touch "$SETTINGS"

# Determine fablize hook paths
ROUTER_HOOK=""
STOP_HOOK=""
ROUTER_HOOK="$PLUGIN_ROOT/fablize/hooks/router.sh"
ROUTER_GATE="$PLUGIN_ROOT/fablize/hooks/gate_prompt.py"
POST_TOOL_GATE="$PLUGIN_ROOT/fablize/hooks/gate_post_tool.py"
STOP_HOOK="$PLUGIN_ROOT/fablize/hooks/gate_stop.py"

# Build the new settings block
# Pass all shell vars as env so the quoted heredoc (no shell expansion) can read them safely.
# A special character in ZEN_KEY or a path cannot inject code this way.
SETTINGS="$SETTINGS" ZEN_DIR="$ZEN_DIR" ZEN_KEY="$ZEN_KEY" \
STITCH_KEY="$STITCH_KEY" \
ROUTER_HOOK="$ROUTER_HOOK" ROUTER_GATE="$ROUTER_GATE" \
POST_TOOL_GATE="$POST_TOOL_GATE" STOP_HOOK="$STOP_HOOK" \
python3 - <<'PYEOF'
import json, os
from pathlib import Path

settings_path = Path(os.environ["SETTINGS"])
try:
    existing = json.loads(settings_path.read_text()) if settings_path.exists() else {}
except json.JSONDecodeError:
    existing = {}

mcp = existing.setdefault("mcpServers", {})
mcp["stitch"] = {
    "command": "npx",
    "args": ["-y", "@_davideast/stitch-mcp", "proxy"],
    "env": {"STITCH_API_KEY": os.environ["STITCH_KEY"]},
}
mcp["zen-router"] = {
    "command": "python3",
    "args": [f"{os.environ['ZEN_DIR']}/server.py"],
    "env": {"OPENCODE_ZEN_API_KEY": os.environ["ZEN_KEY"]},
}

hooks = existing.setdefault("hooks", {})

def add_hook(event, command, matcher=""):
    if not command:
        return
    entries = hooks.setdefault(event, [])
    for e in entries:
        for h in e.get("hooks", []):
            if h.get("command") == command:
                return
    entries.append({"matcher": matcher, "hooks": [{"type": "command", "command": command, "timeout": 10000}]})

def add_hook_matcher(event, matcher, command):
    add_hook(event, command, matcher=matcher)

add_hook("UserPromptSubmit", os.environ["ROUTER_HOOK"])
add_hook("UserPromptSubmit", f"python3 \"{os.environ['ROUTER_GATE']}\"")
add_hook_matcher("PostToolUse", "^(Bash|Edit|Write|NotebookEdit|MultiEdit)$", f"python3 \"{os.environ['POST_TOOL_GATE']}\"")
add_hook("Stop", os.environ["STOP_HOOK"])

settings_path.write_text(json.dumps(existing, indent=2))
print("settings.json updated")
PYEOF

# ── 4. Install stitch-skills into .claude/skills/ ────────────────────────────
info "Installing stitch-skills (google-labs-code)..."
SKILLS_DEST="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel 2>/dev/null || echo "")/.claude/skills"
if [[ -n "$SKILLS_DEST" ]]; then
    mkdir -p "$SKILLS_DEST"
    SKILLS_TMP="$(mktemp -d)"
    git clone --depth=1 https://github.com/google-labs-code/stitch-skills.git "$SKILLS_TMP" 2>/dev/null
    find "$SKILLS_TMP" -name "SKILL.md" | while read f; do
        skill_name=$(basename "$(dirname "$f")")
        cp "$f" "$SKILLS_DEST/${skill_name}.md"
    done
    rm -rf "$SKILLS_TMP"
    skill_count=$(ls "$SKILLS_DEST"/*.md 2>/dev/null | wc -l)
    info "Installed $skill_count stitch skills → $SKILLS_DEST"
else
    warn "Could not determine git repo root — skipping stitch-skills install"
fi

# ── 5. Append token-efficiency block to CLAUDE.md ────────────────────────────
info "Updating ~/.claude/CLAUDE.md with token-efficiency layer..."
CLAUDE_MD="$CLAUDE_DIR/CLAUDE.md"
touch "$CLAUDE_MD"
if ! grep -q "Token-Efficiency Rules" "$CLAUDE_MD" 2>/dev/null; then
    echo "" >> "$CLAUDE_MD"
    cat "$SCRIPT_DIR/config/claude-md-additions.md" >> "$CLAUDE_MD"
    info "Token-efficiency block appended"
else
    info "Token-efficiency block already present — skipping"
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════════════════"
echo "  Installation complete."
echo ""
echo "  fablize:       $PLUGIN_ROOT/fablize"
echo "  zen-router:    $ZEN_DIR"
echo "  stitch MCP:    npx @_davideast/stitch-mcp proxy"
echo "  stitch-skills: .claude/skills/ (14 skills)"
echo "  settings:      $SETTINGS"
echo "  CLAUDE.md:     $CLAUDE_MD"
echo ""
if [[ -z "$ZEN_KEY" ]]; then
echo "  ⚠  Set OpenCode Zen key:"
echo "     export OPENCODE_ZEN_API_KEY=sk-..."
fi
if [[ -z "$STITCH_KEY" ]]; then
echo "  ⚠  Set Stitch API token:"
echo "     export STITCH_API_TOKEN=AQ...."
fi
echo ""
echo "  Restart Claude Code for hooks to take effect."
echo "═══════════════════════════════════════════════════════"
