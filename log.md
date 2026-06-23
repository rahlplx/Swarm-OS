---
type: log
title: Swarm-OS Planning Changelog
description: Chronological record of planning document changes
tags: [planning]
timestamp: "2026-06-23"
---

# Planning Changelog

## 2026-06-23

- **Added** telemetry pipeline v2: full agent harness capture (sessions, user queries, AI reasoning/thinking, tool calls, tool outputs). Schema expanded from 4 v1 tables to 10 tables. New scripts: `scripts/telemetry_collector.py` (Python API), `scripts/hooks/bridge.py` (Claude Code hook bridge), `scripts/session-replay.py` (CLI viewer with `--list`, `--stats`, and full session replay). Hooks wired in `.claude/settings.json` to capture every SessionStart, UserPromptSubmit, PostToolUse, and Stop event. Verified end-to-end with simulated session — all 7 v2 tables populated, full timeline visible via session-replay.
- **Added** `.claude/memory/PROJECT.md`: persistent cross-session memory file. Single source of truth for current state (Phase 0 Day 1 complete), locked decisions, authority map, operating protocol, open questions. Every agent session MUST read this file first.
- **Added** `.claude/rules/INVARIANTS.md`: hard rules extracted from `governance.md`, `architecture.md §7`, and `tech_stack.md`. Covers security (Argon2id, Ed25519, key prefixes, prompt privacy, hardware fingerprint), ledger (microcredits, hash chain, ADMIN_ADJUST separation), license (exo GPL-3.0 isolation, Grafana AGPL self-host, distributed-llama AVOID), blackboard, scheduler, CI, branch hygiene. Violating any rule is a critical error.
- **Added** `.claude/rules/SESSION_PROTOCOL.md`: session bootstrap checklist + operating protocol. Defines the 6-step startup sequence (load memory → verify env → identify session type → operate → session-end update → auto-trigger config).
- **Added** `scripts/hooks/on-session-start.sh`: SessionStart hook that auto-emits a bootstrap banner + state snapshot on every new Claude Code session. Wired into `.claude/settings.json` via `SessionStart` hook entry.
- **Updated** `.claude/settings.json`: added `SessionStart` hook (on-session-start.sh) + `PostToolUse` hook (on-code-change.sh) + `permissions.allow` block. Hooks now fire on session start, every prompt, every code change, and session end.
- **Updated** `CLAUDE.md`: added "Session Bootstrap (mandatory — do this FIRST)" section at top, referencing the 4 files to read before responding to any prompt.
- **Updated** `AGENTS.md`: same session bootstrap section, with CLAUDE.md as step 4 (since AGENTS.md is the entry point for non-Claude agents).
- **Updated** `index.md`: added 3 new rows to the document map for `.claude/memory/PROJECT.md`, `.claude/rules/INVARIANTS.md`, `.claude/rules/SESSION_PROTOCOL.md`.
- **Merged** PR #11 (feat(phase0): scaffold Tauri v2 app with TDD harness & telemetry) into `main` as squash commit `60f2993`. Phase 0 Day 1 scaffold complete: 56 tests across Rust/React/Python, hardware profiler, capability scoring, BLAKE3 model verification, SwarmProvider for LiteLLM, telemetry DB, CI pipeline, git hooks.
- **Fixed** during PR #11 review: added missing `eslint.config.js` (ESLint v9 flat config — CI was failing because the `--ext` flag was removed in commit 0372bbe but no flat config was added). Fixed `react-hooks/set-state-in-effect` warning in `useModelManager.ts` via `useCallback` + `queueMicrotask`. Updated `.gitignore` to ignore `*.egg-info/`.
- **Verified locally** (CI could not run due to GitHub-hosted ubuntu-latest runner-assignment failures — 5 consecutive attempts, 3-second job failures with zero steps executed): `pnpm lint` ✓, `pnpm test` 19/19 ✓, `pnpm exec tsc --noEmit` ✓, `ruff check .` ✓, `pytest tests/ -v` 6/6 ✓, `cargo fmt --check` ✓. `cargo clippy` + `cargo test` could not run (libwebkit2gtk-4.1-dev not installable in sandbox); source review confirms all 10 Gemini Code Assist findings already addressed in commit 0372bbe.
- **Deleted** all merged branches: `chore/gitignore`, `claude/global-plugins-setup`, `claude/product-planning-research-oqkcsg`, `claude/init-6nkhbq`. Repo now has only `main`.
- **Added** `AGENTS.md`: cross-tool agent onboarding file (companion to `CLAUDE.md`). Tool-agnostic; mirrors CLAUDE.md content but defers to CLAUDE.md for Claude Code sessions. Covers authority map, operating protocol, verification gate, security invariants, license constraints, build toolchain, CI checks, branch/PR conventions.
- **Updated** `CLAUDE.md`: current state changed from "planning/documentation phase" to "Phase 0 in progress — single-device local AI inference app (Tauri v2 + llama-cpp-2 + LiteLLM)".

## 2026-06-22

- **Added** index.md, log.md: OKF v0.1 bundle structure with authority map and reading paths
- **Updated** all docs: Added YAML frontmatter (type, title, tags, authority, phase, token_estimate)
- **Updated** architecture.md: Converted prose sections to structured tables (node churn, activation tensors, topology comparison)
- **Updated** critique.md: Added RESOLVED status markers with cross-links for fixed issues
- **Updated** project.md, tech_stack.md, governance.md: Removed redundant content, added cross-links to authoritative sources
- **Added** guide.md: Non-technical ebook-style guide covering pre-Phase 0 prerequisites and Phase 0 day-by-day roadmap
- **Added** research.md: Component research report with benchmarks for all OSS dependencies
- **Updated** all docs: Corrected llama-cpp-2 binding name, distributed-llama AVOID verdict, activation tensor sizes, Orchestrator gRPC frameworks
- **Added** CLAUDE.md: Claude Code onboarding context file

## 2026-06-21

- **Added** verify-prompt.md: Structured community verification prompt
- **Updated** project.md, tech_stack.md: Pre-phase OSS analysis protocol and P2P ecosystem research
- **Fixed** 16 issues from critique review: API key removed from etcd, ledger moved to SQLite WAL, Argon2id for key hashing, prompt privacy, bKash flow, trial tier

## 2026-06-20

- **Added** project.md, architecture.md, tech_stack.md, ui_ux.md, governance.md: Initial planning docs
- **Added** critique.md: 28 severity-ranked issues with action plan
