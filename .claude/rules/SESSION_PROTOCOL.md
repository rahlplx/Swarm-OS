---
type: rules
title: Swarm-OS Session Bootstrap Protocol
description: Checklist every agent session MUST execute on startup to load full context and obey governance
tags: [rules, session, bootstrap, auto-trigger]
timestamp: "2026-06-23"
status: active
---

# Swarm-OS Session Bootstrap Protocol

> **Every new agent session (Claude Code, Cursor, Aider, etc.) MUST execute this checklist on startup before doing any work.** This is the auto-trigger sequence that ensures full context is loaded and governance is obeyed.

## Step 1: Load Persistent Memory (mandatory first read)

Read in this exact order — each file builds on the previous:

1. **`.claude/memory/PROJECT.md`** — current state, decisions, invariants, progress
2. **`CLAUDE.md`** (Claude Code) OR **`AGENTS.md`** (any other agent) — detailed onboarding
3. **`.claude/rules/INVARIANTS.md`** — hard rules that must never be violated
4. **`.claude/rules/SESSION_PROTOCOL.md`** — this file
5. **`log.md`** — recent changes that may affect your work
6. **`index.md`** — document map + authority assignments

**Token budget:** ~6,000 tokens for the full bootstrap. This is intentional — skipping context leads to rework that costs far more.

## Step 2: Verify Environment

Run `scripts/verify-environment.sh` (or equivalent manual checks):

```bash
# Rust
cargo --version        # 1.78+ required, 1.96 stable confirmed
rustc --version
# Node
node --version         # 22+
pnpm --version         # 10+
# Python
python3 --version      # 3.10+
ruff --version         # 0.8+
# Git
git --version
```

If any tool is missing, document it and proceed only if the task doesn't require that tool.

## Step 3: Identify Session Type

Determine which type of session this is:

| Type | Trigger | Action |
|------|---------|--------|
| **Implementation** | User asks for code change | Read relevant `architecture.md` / `governance.md` sections + existing source before writing |
| **Planning/Docs** | User asks for doc change | Read `index.md` authority map, identify authoritative source, update that doc + cross-links |
| **Review** | User asks for PR/commit review | Read `critique.md` for known issue patterns, check `INVARIANTS.md` against changes |
| **Research** | User asks for analysis | Read `research.md` + `tech_stack.md` for benchmarks and OSS verdicts |
| **Debugging** | User reports a failure | Read `architecture.md §8` (failure mode matrix) + `critique.md` |

## Step 4: Operating Protocol (during session)

### Tool Call Batching
- Call all INDEPENDENT tools in ONE message.
- Only serialize when output of call A is an input to call B.

### Response Length Discipline
| Task type | Max response length |
|-----------|-------------------|
| Single-file edit | 1-sentence summary after edit |
| Bug fix | Root cause + what changed. No narration. |
| Research / explain | Direct answer first, supporting detail below. |
| Multi-step plan | Bullet list only. No prose paragraphs. |
| Code review finding | One sentence per finding. |

### Verification Gate (mandatory before marking DONE)
1. The change was actually written (tool call returned success).
2. For code: tests pass OR explicit reason why tests are skipped.
3. For docs: the diff is visible in git status.
"Saying 'this should work'" without verification = incomplete.

### Security Checklist (before any code merge)
- [ ] No API keys in job payloads?
- [ ] No prompts in etcd?
- [ ] Argon2id used for key hashing (not SHA-256)?
- [ ] Ledger entries use integer microcredits (not float)?
- [ ] ADMIN_ADJUST in separate table (not hash chain)?
- [ ] No GPL-3.0 code ported from exo?
- [ ] No Grafana code linked into codebase?
- [ ] Key prefixes correct (`swrm_sk_`, `swrm_ops_`, `swrm_adm_`, `swrm_node_`)?

## Step 5: Session End (mandatory)

Before ending the session:

1. **Update `.claude/memory/PROJECT.md`** if any of these changed:
   - Phase / Day progress
   - Branch state
   - Open PRs
   - Critical decisions
   - Open questions / deferred items
   - Append a line to the "Change Log" section (§10) with date + summary.

2. **Update `log.md`** if any docs changed (per repo convention).

3. **Update `/home/z/my-project/worklog.md`** (if in Super Z sandbox) with a new `---`-delimited section:
   ```
   ---
   Task ID: <next integer>
   Agent: <agent name>
   Task: <what you were asked to do>
   Work Log:
   - <step 1>
   - <step 2>
   Stage Summary:
   - <key results>
   ```

4. **Run the Stop hook** (if configured): `bash scripts/hooks/on-session-end.sh` — runs full test suite + telemetry.

## Step 6: Cross-Session Auto-Trigger

To make this protocol fire on EVERY new Claude Code session (not just when the user mentions Swarm-OS):

### Option A: Local install (recommended)
On the user's local machine:
```bash
cd Swarm-OS
OPENCODE_ZEN_API_KEY=sk-... STITCH_API_TOKEN=AQ... bash global-plugins/install.sh
```
This writes `~/.claude/settings.json` with hooks that auto-load context on every session.

### Option B: Repo-level hooks (already configured)
The repo's `.claude/settings.json` defines:
- `UserPromptSubmit` → `scripts/hooks/on-prompt.sh` (every-5th-query checkpoint)
- `Stop` → `scripts/hooks/on-session-end.sh` (full test suite + telemetry)

These fire automatically when Claude Code opens the repo. No user action needed.

### Option C: Manual
At the start of each session, the user mentions "Swarm-OS" and the agent reads `.claude/memory/PROJECT.md` first.

## Quick Reference: Where to Find Things

```
.claude/memory/PROJECT.md          ← READ FIRST — persistent state + decisions
.claude/rules/INVARIANTS.md        ← READ SECOND — hard rules
.claude/rules/SESSION_PROTOCOL.md  ← READ THIRD — this file
.claude/settings.json              ← Claude Code hook config
.claude/skills/*.md                ← 14 stitch skills (design, react, shadcn, etc.)
CLAUDE.md                          ← Claude Code detailed onboarding
AGENTS.md                          ← Cross-tool agent onboarding
index.md                           ← document map + authority assignments
log.md                             ← chronological changelog
project.md                         ← product identity, features, roadmap
architecture.md                    ← system layers, scheduler, security (§7)
governance.md                      ← roles, keys, credit, ledger, abuse prevention
tech_stack.md                      ← OSS deps, licenses, build toolchain
ui_ux.md                           ← design system, wireframes
critique.md                        ← 28 severity-ranked issues
guide.md                           ← Phase 0 day-by-day guide
research.md                        ← benchmarks, code samples, OSS verdicts
verify-prompt.md                   ← community verification prompt
src-tauri/                         ← Rust backend
src/                               ← React frontend
litellm-proxy/                     ← Python LiteLLM provider
global-plugins/                    ← fablize + zen-router + stitch installer
.github/workflows/                 ← CI + nightly pipelines
scripts/                           ← telemetry, hooks, acceptance tests
```
