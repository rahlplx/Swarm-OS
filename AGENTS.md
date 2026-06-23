---
type: context
title: AGENTS.md — Cross-Tool Agent Onboarding
description: Generic onboarding context for any coding agent (Claude Code, Cursor, Aider, etc.) working on Swarm-OS
tags: [architecture, planning, agents]
timestamp: "2026-06-23"
status: active
phase: "0-4"
token_estimate: 1200
---

# AGENTS.md

This file provides onboarding context for **any** AI coding agent working in the Swarm-OS repository. It is the tool-agnostic companion to [`CLAUDE.md`](./CLAUDE.md) (which is Claude Code-specific). When the two conflict, `CLAUDE.md` wins for Claude Code sessions; `AGENTS.md` wins for all other agents.

## Why this file exists

Different agent tools look for different onboarding filenames: Claude Code reads `CLAUDE.md`, Cursor reads `.cursorrules`, Aider reads `CONVENTIONS.md`, etc. `AGENTS.md` is the emerging cross-tool convention. Keeping it in sync with `CLAUDE.md` means any agent dropped into this repo gets the same operating context.

## Project Overview

Swarm-OS is a decentralized P2P AI inference network. Contributors pool idle GPU/CPU compute into a WireGuard mesh; consumers run 7B–405B models via an OpenAI-compatible API. A tamper-evident credit ledger tracks earn/spend. Made in Bangladesh (BDT payments via SSLCommerz/bKash), Apache 2.0.

**Current state:** Phase 0 in progress — single-device local AI inference app (Tauri v2 + llama-cpp-2 + LiteLLM). The repo contains both planning docs (`*.md` at root) and the Phase 0 implementation scaffold (`src-tauri/`, `src/`, `litellm-proxy/`).

## Authority Map (OKF)

When information appears in multiple docs, the authoritative source wins. See [`index.md`](./index.md) for the full map. Key authorities:

| Concept | Authoritative Source |
|---------|---------------------|
| Scheduler scoring formula | `architecture.md §3` |
| Credit earn/spend formula | `governance.md §3.4` |
| API key hashing (Argon2id) | `architecture.md §7` |
| Ledger entry format | `governance.md §4.1` |
| Key prefixes (`swrm_*`) | `governance.md §1` |
| Hardware fingerprint | `governance.md §5.3` |
| Activation tensor sizes | `architecture.md §4` |
| Pipeline ring topology | `architecture.md §4` |
| OSS dependency verdicts | `tech_stack.md` |
| Benchmark numbers | `research.md` |
| Phase 0 day-by-day roadmap | `guide.md` Ch.9 |
| Design system (colors, fonts) | `ui_ux.md §1` |

## Operating Protocol (all agents)

### Session Bootstrap (mandatory — do this FIRST)
Read these files in order before responding to any user prompt:

1. **`.claude/memory/PROJECT.md`** — persistent state, decisions, invariants, progress
2. **`.claude/rules/INVARIANTS.md`** — hard rules that must never be violated (security, ledger, license)
3. **`.claude/rules/SESSION_PROTOCOL.md`** — session bootstrap checklist + operating protocol
4. **`CLAUDE.md`** (even if you're not Claude — it's the most detailed onboarding doc)
5. **`log.md`** — recent changes that may affect your work

### Before writing code
1. Read `index.md` to find the authoritative source for any concept you're touching.
2. Check `log.md` for recent changes that may affect your work.
3. Check `.claude/memory/PROJECT.md` §9 (Open Questions / Deferred Items) for known blockers.

### Verification gate (mandatory)
Do not mark a task DONE until:
1. The change was actually written (tool call returned success).
2. For code: tests pass OR an explicit reason why tests are skipped.
3. For docs: the diff is visible in git status.
"Saying 'this should work'" without verification = incomplete.

### Token efficiency
- Batch independent tool calls in a single message.
- Do not re-read a file you already read this turn.
- Do not re-derive facts already established in the conversation.
- For mechanical tasks (boilerplate, docstrings, commit messages), delegate to a cheaper model if available.

### Security invariants (never violate)
- **Prompts never enter etcd.** Job payloads contain only routing metadata.
- **API keys never enter job payloads.** Use one-time job tokens with 60s TTL.
- **API keys hashed with Argon2id** (time=3, mem=64MB, parallelism=4, per-key salt) — never SHA-256.
- **Ledger entries signed with Ed25519** per node; hash chain uses SHA-256. `credits_delta` stored as integer microcredits.
- **ADMIN_ADJUST entries go in a separate SQLite table** — never in the hash chain.
- **Key prefixes:** `swrm_sk_` (consumer), `swrm_ops_` (operator), `swrm_adm_` (super admin), `swrm_node_` (inter-node).

### License constraints
- **exo-labs/exo is GPL-3.0** — reference/study only, no code porting. Clean-room Rust implementation of ring topology required.
- **Grafana is AGPL-3.0** — use via self-host or cloud tier; don't link into our codebase.
- **b4rtaz/distributed-llama is MIT but AVOID** — matrix-parallel (not pipeline-parallel), 2^k node constraint, custom `.bin` format.
- All Swarm-OS code: Apache 2.0.

### Build toolchain

| Language | Tool | Version |
|----------|------|---------|
| Rust | cargo | 1.78+ (currently 1.96 stable) |
| Python | pip / uv | 3.10+ |
| TypeScript | pnpm + tsc | 9+ / 5.6+ |
| Bundler | Vite | 5+ |
| Container | Docker + compose | 26+ / 2.25+ |

### CI checks (must pass before merge)
- **Rust:** `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
- **React:** `pnpm exec tsc --noEmit`, `pnpm test`, `pnpm lint`
- **Python:** `ruff check .`, `pytest tests/ -v`
- **Tauri system deps (Linux):** `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libssl-dev`

### Branch / PR conventions
- Branch naming: `claude/<task-id>` for Claude-driven work, `<type>/<scope>` for human work (e.g. `chore/gitignore`, `feat/phase0-scaffold`).
- Squash-merge to `main`. Delete the source branch after merge.
- Commit messages: conventional commits (`feat:`, `fix:`, `docs:`, `chore:`, `ci:`).

## Cross-Session Auto-Trigger

To make every new agent session auto-load this context:
- **Claude Code:** run `bash global-plugins/install.sh` locally (registers fablize hooks + zen-router MCP + writes `~/.claude/CLAUDE.md` token-efficiency block).
- **Other agents:** ensure your tool is configured to read `AGENTS.md` on session start (most modern agents do this by default).

## File Quick Reference

```
CLAUDE.md                  ← Claude Code onboarding (canonical, most detailed)
AGENTS.md                  ← this file — cross-tool agent onboarding
index.md                   ← document map + authority assignments
log.md                     ← chronological changelog of planning docs
project.md                 ← product identity, features F1-F9, phase roadmap
architecture.md            ← system layers, scheduler, sharding, security
tech_stack.md              ← OSS deps with licenses, build toolchain
governance.md              ← roles, keys, credit formula, ledger, abuse prevention
ui_ux.md                   ← design system, screen wireframes
critique.md                ← 28 severity-ranked issues with action plan
guide.md                   ← non-technical Phase 0 day-by-day guide
research.md                ← component research with benchmarks
verify-prompt.md           ← community verification prompt
src-tauri/                 ← Rust backend (Tauri v2)
src/                       ← React frontend
litellm-proxy/             ← Python LiteLLM custom provider
global-plugins/            ← fablize + zen-router + stitch MCP installer
.github/workflows/ci.yml   ← CI pipeline
scripts/                   ← telemetry, git hooks, setup scripts
```
