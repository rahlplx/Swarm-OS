---
type: memory
title: Swarm-OS Persistent Project Memory
description: Cross-session memory — authoritative state, decisions, invariants, and progress for Swarm-OS
tags: [memory, context, persistent]
timestamp: "2026-06-23"
status: active
---

# Swarm-OS Project Memory

> **This file is the cross-session memory.** Every new agent session MUST read this file first, before any other context. It captures decisions, state, and invariants that persist across sessions. Update this file when state changes — do not create parallel memory files.

Last updated: 2026-06-23

---

## 1. Current State (single source of truth)

- **Phase:** 0 (single-device local AI inference app)
- **Phase 0 Day:** 1 complete (scaffold merged to `main` as `60f2993`)
- **Phase 0 Day:** 2+ not started
- **Branch:** `main` only (all feature branches deleted after merge)
- **Open PRs:** 0
- **Merged PRs:** 11
- **HEAD of main:** `f11684a fix(code): audit-level fixes across Rust, React, Python`
- **CI status:** GitHub-hosted ubuntu-latest runners had assignment failures on 2026-06-23 (transient infra issue). Local verification: all checks pass.
- **Known alerts:** 1 moderate dependabot alert — see https://github.com/rahlplx/Swarm-OS/security/dependabot/1

## 2. What Has Been Built (Phase 0 Day 1)

| Component | Location | Status |
|-----------|----------|--------|
| Tauri v2 Rust backend | `src-tauri/` | scaffold + 31 tests passing |
| React frontend | `src/` | scaffold + 19 tests passing |
| LiteLLM SwarmProvider | `litellm-proxy/` | scaffold + 6 tests passing |
| CI pipeline | `.github/workflows/ci.yml` | 3 jobs (rust, frontend, python) |
| Nightly GPU CI | `.github/workflows/nightly.yml` | scaffold |
| Git hooks | `scripts/setup-git-hooks.sh` | pre-commit + pre-push |
| Telemetry DB | `scripts/schema.sql` + `telemetry.sh` | scaffold |
| 20-day acceptance tests | `scripts/acceptance/day01.sh`–`day20.sh` | day01 passes, day02–20 stubs |
| Claude Code hooks | `.claude/settings.json` + `scripts/hooks/` | UserPromptSubmit + Stop hooks |
| Design skills | `.claude/skills/*.md` | 14 stitch skills installed |

### Phase 0 Day 1 components (Rust)
- `hardware/profiler.rs` — `SystemProfiler` with `GpuDetector` trait (mockable), `NoGpuDetector` default
- `hardware/capability.rs` — `compute_capability()` per architecture.md §3 formula (`vram×4 + ram×0.5 + cpu×0.25 + backend_bonus`)
- `inference/model_manager.rs` — BLAKE3 streaming hash, GGUF model listing, verification
- `inference/engine.rs` — `InferenceEngine` trait
- `inference/streaming.rs` — SSE token stream
- `litellm/process.rs` — LiteLLM subprocess manager with `Stdio::null()` (prevents pipe-buffer hangs)
- `litellm/bridge.rs` — config bridge
- `ipc/commands.rs` — `detect_hardware`, `get_capability_score`, `list_models`, `health_check`

## 3. Critical Decisions Locked In

These decisions are FINAL until a Super Admin (per governance.md §1) explicitly reverses them. Do not re-litigate.

### Architecture
- **Inference engine:** llama.cpp (MIT) + GGUF format
- **Rust binding:** `llama-cpp-2` (utilityai) — NOT `llama-rs` (archived, no GGUF)
- **Coordination:** etcd v3 for live state, SQLite WAL for ledger (NOT etcd — compaction breaks hash chains)
- **P2P mesh:** WireGuard + headscale (self-hosted Tailscale control plane)
- **Cross-node sharding:** Clean-room Rust pipeline-parallel ring topology (study exo-labs/exo design, NO code porting — GPL-3.0)
- **API gateway:** LiteLLM proxy with custom `swarm` provider plugin
- **Desktop agent:** Tauri v2 (Rust backend + React frontend)

### Security (architecture.md §7)
- **API key hashing:** Argon2id (time=3, mem=64MB, parallelism=4, per-key random salt) — never SHA-256
- **Ledger signatures:** Ed25519 per entry; SHA-256 hash chain
- **Credits:** integer microcredits (1 credit = 1,000,000 microcredits) — never float in hash inputs
- **ADMIN_ADJUST:** separate SQLite table, never in hash chain, each individually signed by operator key
- **Key prefixes:** `swrm_sk_` (consumer), `swrm_ops_` (operator), `swrm_adm_` (super admin), `swrm_node_` (inter-node)
- **Hardware fingerprint:** `SHA-256(gpu_uuid:motherboard_serial:cpu_id:machine_id)` with `:` delimiters; `/etc/machine-id` is primary anchor; OEM sentinel strings excluded

### Credit Economy (governance.md §3.4)
- Earn: `tokens × 0.008` (× 1.1 if node_score > 80)
- Spend: `(input_tokens × 0.006) + (output_tokens × 0.01)`
- Platform spread: ~20%
- Abuse cap: `max_credits_per_hour = node_score × 2`
- BDT top-up tiers: ৳50→Standard (2.5 cr/BDT), ৳450→Plus (2.67, +7%), ৳900→Pro (2.89, +15%); rounding=floor

### License Constraints (tech_stack.md)
- **exo-labs/exo (GPL-3.0):** reference/study ONLY — no code porting
- **Grafana (AGPL-3.0):** self-host only, don't link into codebase
- **b4rtaz/distributed-llama (MIT but AVOID):** matrix-parallel, 2^k nodes, `.bin` format, 972ms/fwd on WAN
- All Swarm-OS code: Apache 2.0

## 4. Authority Map (conflict resolution)

When info appears in multiple docs, the authoritative source wins. See `index.md` for the full map.

| Concept | Authority |
|---------|-----------|
| Scheduler scoring formula | `architecture.md §3` |
| Credit formula / ledger config | `governance.md §3.4` |
| API key hashing | `architecture.md §7` |
| Ledger entry format / audit | `governance.md §4` |
| Hardware fingerprint | `governance.md §5.3` |
| Activation tensor sizes | `architecture.md §4` |
| Pipeline ring topology | `architecture.md §4` |
| OSS dependency verdicts | `tech_stack.md` |
| Benchmark numbers | `research.md` |
| Phase 0 day-by-day roadmap | `guide.md` Ch.9 |
| Design system | `ui_ux.md §1` |
| Failure modes | `architecture.md §8` |

## 5. Operating Protocol (every session, every agent)

1. **Read this file first** — `.claude/memory/PROJECT.md`
2. **Read `CLAUDE.md`** (Claude Code) or `AGENTS.md` (other agents) for detailed onboarding
3. **Read `.claude/rules/INVARIANTS.md`** — hard rules that must never be violated
4. **Read `.claude/rules/SESSION_PROTOCOL.md`** — session bootstrap checklist
5. **Check `log.md`** for recent changes that may affect your work
6. **Check `/home/z/my-project/worklog.md`** for prior session audit trail (if running in the Super Z sandbox)

## 6. Sandbox vs. Local Machine

This session may be running in either:
- **Super Z sandbox** at `/home/z/my-project/Swarm-OS/` — has Python, Node, pnpm, Rust toolchain. Cannot install apt packages (no sudo). Cannot run `claude` CLI. Cannot modify `~/.claude/`.
- **User's local machine** — has full system access, can run `claude` CLI, can install apt deps, can run `global-plugins/install.sh`.

If in sandbox, use `--break-system-packages` for pip and document any local-only steps for the user to run.

## 7. Token Efficiency (always active)

- Batch independent tool calls in a single message
- Do not re-read a file already read this turn
- Do not re-derive facts already in conversation
- For mechanical tasks (boilerplate, docstrings, commit messages), delegate to cheaper model if available
- Never write "I'll now..." or "Let me..." — just do it
- Never restate the user's request before answering

## 8. Verification Gate (fablize — mandatory)

Do not mark a task DONE until:
1. The change was actually written (tool call returned success)
2. For code: tests pass OR an explicit reason why tests are skipped
3. For docs: the diff is visible in git status

"Saying 'this should work'" without verification = incomplete.

## 9. Open Questions / Deferred Items

- **AGENTS.md / agentsm.md:** `AGENTS.md` created 2026-06-23. `agentsm.md` does not exist — appears to be a typo for `AGENTS.md`. If user confirms, no action needed.
- **CI runner failures:** GitHub-hosted ubuntu-latest runners failed to assign across 5 consecutive attempts on 2026-06-23. Transient infra issue. Monitor.
- **Dependabot alert:** 1 moderate vulnerability on main. Needs triage.
- **`cargo clippy` + `cargo test` in sandbox:** Cannot run (libwebkit2gtk-4.1-dev not installable). Source review confirms all 10 Gemini fixes in place. User should run on local machine to confirm.

## 10. Change Log (memory updates)

- **2026-06-23 (session 3):** Code-level audit of all source. Fixed 12 issues: Rust case-insensitive .gguf matching, React set-state-in-effect pattern (4 components), React key anti-patterns (3 components), React NaN/clamp guards (2 components), Python __init__.py exports + provider error wrapping + callback type hints. All tests + lint pass. HEAD=f11684a.
- **2026-06-23 (session 2):** Added cross-session memory + rules system: .claude/memory/PROJECT.md, .claude/rules/INVARIANTS.md, .claude/rules/SESSION_PROTOCOL.md, scripts/hooks/on-session-start.sh, SessionStart hook in .claude/settings.json. HEAD=b748fd5.
- **2026-06-23 (session 1):** Initial memory file created. State: Phase 0 Day 1 complete, PR #11 merged, all branches deleted, AGENTS.md added.
