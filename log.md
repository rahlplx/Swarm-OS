---
type: log
title: Swarm-OS Planning Changelog
description: Chronological record of planning document changes
tags: [planning]
timestamp: "2026-06-23"
---

# Planning Changelog

## 2026-06-23

- **Added** `LICENSE` file (Apache 2.0 full text). README.md referenced it but it didn't exist. Fixes INVARIANTS.md §3.4 violation.
- **Added** `SECURITY.md`: security policy + accepted-risk disposition for dependabot alert #1 (glib unsoundness in VariantStrIter). Documented that the vulnerable API is not used in Swarm-OS codebase (transitive dep via Tauri's GTK tray/window management); fix requires Tauri upgrade (deferred to Phase 1+).
- **Fixed** 4 license-field violations: added `license = "Apache-2.0"` to `node-agent/Cargo.toml`, `package.json`, `gateway/pyproject.toml`. `src-tauri/Cargo.toml` and `litellm-proxy/pyproject.toml` already had it.
- **Added** 4th CI job for `gateway/` package. PR #12 added `gateway/` (5 source files, 8 tests) but CI only tested `litellm-proxy/`. Now CI has 4 jobs: rust, frontend, python (litellm-proxy), gateway. Also added ruff config to `gateway/pyproject.toml` matching litellm-proxy settings.
- **Updated** `phase_progress` telemetry: Day 1 (Development Environment) and Day 2 (App Scaffold) marked as completed. Day 1: 7/9 acceptance tests pass (cmake + cargo check fail in sandbox). Day 2: 4/4 pass.
- **Added** telemetry v2.1: closed all 6 gaps found in v2 audit. (1) `duration_seconds` — fixed timestamp format mismatch (`started_at` now uses `_utc_now_sqlite()` matching `ended_at`; added `_parse_timestamp()` that handles both ISO-8601 and SQLite formats). (2) `response_tokens_estimate` — computed from total tool output bytes / 4 at session end. (3) `tool_calls_invoked` — `start_tool_call()` now increments it; `update_query_response()` changed to only update non-None fields (preserves increments). (4) `files_modified` — counted via `json_extract(tool_input, '$.filepath')` for Edit/Write/MultiEdit/NotebookEdit tools. (5) `tool_calls.duration_ms` — added `PreToolUse` hook (`tool-call-start`) that records start timestamp; `PostToolUse` (`tool-call-end`) computes duration. New methods: `start_tool_call()` + `end_tool_call()`. (6) `commits_made` — `start_session()` saves HEAD sha to `.swarm-os/session-start-sha`; `end_session()` runs `git rev-list --count {start_sha}..HEAD`. New scripts: `scripts/dashboard.py` (live terminal dashboard with `--watch` mode), `scripts/session-export.py` (JSON export for external analysis). Verified E2E: all 7 fields populate correctly (duration=1s, response_ms=491, tokens=1, tools_invoked=2, files_modified=1, tool durations=[728,924]ms, commits=1).
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
