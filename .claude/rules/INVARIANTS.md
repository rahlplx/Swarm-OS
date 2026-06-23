---
type: rules
title: Swarm-OS Hard Invariants — Must Obey
description: Non-negotiable rules extracted from governance.md, architecture.md §7, and license constraints. Violating any of these is a critical error.
tags: [rules, security, ledger, license, invariants]
timestamp: "2026-06-23"
status: active
---

# Swarm-OS Hard Invariants

> **These rules are non-negotiable.** They are extracted from `governance.md`, `architecture.md §7`, and `tech_stack.md`. Violating any of them is a critical error. When in doubt, read the authoritative source doc (linked in each rule).

## 1. Security Invariants (architecture.md §7, governance.md §1)

### 1.1 API Key Hashing
- **MUST** hash API keys with Argon2id (time=3, mem=64MB, parallelism=4, per-key random salt).
- **MUST NOT** use SHA-256, bcrypt, or PBKDF2 for API key hashing.
- **MUST NOT** store plaintext API keys anywhere — not in etcd, not in logs, not in job payloads.

### 1.2 API Key Prefixes (governance.md §1)
- `swrm_sk_` — consumer
- `swrm_ops_` — operator
- `swrm_adm_` — super admin
- `swrm_node_` — inter-node contributor
- **MUST NOT** create keys without the correct prefix.
- **MUST NOT** accept keys with wrong/missing prefixes in any authenticated endpoint.

### 1.3 Prompt Privacy
- **MUST NOT** write prompt content to etcd. Job payloads in etcd contain only: `{model, job_token, max_tokens, stream}`.
- **MUST** deliver prompts P2P directly to assigned nodes.
- **MUST NOT** log prompt content at INFO level. If debugging is required, log at DEBUG with explicit redaction.

### 1.4 API Keys in Job Payloads
- **MUST NOT** include API keys in job payloads. Use one-time job authorization tokens with 60s TTL.

### 1.5 Network Security
- **MUST** use WireGuard (ChaCha20-Poly1305) for all inter-node traffic.
- **MUST** use etcd client certificate auth. etcd **MUST NOT** be exposed to the internet.
- **MUST** use TLS for all client-facing APIs (LiteLLM proxy, admin portal).

### 1.6 Inference Sandboxing
- **MUST** run inference under seccomp (Linux) or App Sandbox (macOS).
- **MUST NOT** allow cross-node KV cache access.

### 1.7 Node Authentication
- **MUST** generate Ed25519 keypair on first run.
- **MUST** register public key in Blackboard at join time.
- **MUST** sign all ledger entries with node private key.

### 1.8 Hardware Fingerprint (governance.md §5.3)
- **MUST** compute as `SHA-256(gpu_uuid:motherboard_serial:cpu_id:machine_id)` for NVIDIA.
- **MUST** compute as `SHA-256(motherboard_serial:cpu_id:machine_id)` for AMD/CPU-only.
- **MUST** use `:` delimiters between fields (prevents prefix-collision attacks).
- **MUST** use `/etc/machine-id` as primary anchor.
- **MUST** treat OEM sentinel strings (`"To be filled by O.E.M."`, empty) as absent.

### 1.9 Fake Contribution Detection (governance.md §5.1)
- **MUST** use challenge-response with known greedy-decode (temp=0) outputs only.
- **MUST NOT** use peer output comparison (non-deterministic at temp > 0 causes false positives).

## 2. Ledger Invariants (governance.md §3.4, §4)

### 2.1 Hash Chain Integrity
- **MUST** sign every ledger entry with Ed25519.
- **MUST** use SHA-256 for the hash chain.
- **MUST** store `credits_delta` as integer microcredits (1 credit = 1,000,000 microcredits).
- **MUST NOT** use float `credits_delta` in hash inputs — IEEE 754 rounding differs by architecture.
- **MUST** set `prev_entry_hash` = SHA-256(serialized previous entry).

### 2.2 ADMIN_ADJUST Storage
- **MUST** store manual credit adjustments in a separate `admin_adjustments` SQLite table.
- **MUST NOT** write ADMIN_ADJUST entries to the hash chain.
- **MUST** sign each ADMIN_ADJUST entry individually with the operator key.
- **MUST** include `id`, `node_id`, `credits_delta_microcredits`, `reason`, `operator_key_id`, `timestamp` in the signature input.

### 2.3 Ledger Storage Location
- **MUST** store the ledger in SQLite WAL on the orchestrator.
- **MUST NOT** store the ledger in etcd — auto-compaction breaks SHA-256 hash chains.
- **MUST** store only `/swarm/ledger/{node_id}/head_hash` pointer in etcd.

### 2.4 Credit Formula (governance.md §3.4)
- Earn: `tokens_generated × 0.008` (× 1.1 if node_score > 80)
- Spend: `(input_tokens × 0.006) + (output_tokens × 0.01)`
- Abuse cap: `max_credits_per_hour = node_score × 2`
- Top-up rounding: floor (always round down to whole credit)
- **MUST** verify `earn (0.008) < spend (0.01)` — no infinite credit creation possible.

## 3. License Invariants (tech_stack.md, CLAUDE.md)

### 3.1 GPL-3.0 — Strict Isolation
- **MUST** treat `exo-labs/exo` (GPL-3.0) as reference/study ONLY.
- **MUST NOT** port, copy, or derive code from exo into Swarm-OS.
- **MUST** implement cross-node sharding as clean-room Rust (study the algorithm, write fresh code).

### 3.2 AGPL-3.0 — Self-Host Only
- **MUST** use Grafana via self-host or cloud tier.
- **MUST NOT** link Grafana code into the Swarm-OS codebase.

### 3.3 AVOID — distributed-llama
- **MUST NOT** use `b4rtaz/distributed-llama` despite its MIT license.
- Reasons: matrix-parallel (not pipeline-parallel), 2^k node constraint, custom `.bin` format (not GGUF), 972ms/forward-pass on WAN.

### 3.4 Swarm-OS License
- **MUST** license all Swarm-OS code as Apache 2.0.
- **MUST** preserve license headers in all source files.

## 4. Blackboard (etcd) Invariants (architecture.md §2)

### 4.1 Key Space
- `/swarm/nodes/{id}/alive` — TTL=10s, write every 5s
- `/swarm/jobs/{id}/*` — job routing metadata only (NO prompts, NO API keys)
- `/swarm/config/*` — scheduler policy, ledger policy, alerting config, model allowlist
- `/swarm/ledger/{node_id}/head_hash` — pointer only (NOT the ledger itself)

### 4.2 etcd Watch Reliability
- **MUST** implement re-watch logic with point-in-time Get on `ErrCompacted`.
- **MUST NOT** assume etcd watch streams deliver all events — compaction can drop them.

### 4.3 Value Size
- **MUST** keep etcd values under 1.5 MB (etcd default max).
- **MUST NOT** store model files, prompts, or large payloads in etcd.

## 5. Scheduler Invariants (architecture.md §3)

### 5.1 Two-Phase Scheduling
- **MUST** run hard-gate pre-filter first (VRAM, backend, locality).
- **MUST** run weighted capability scoring second: `vram×4 + ram×0.5 + cpu×0.25 + backend_bonus + locality_bonus`.
- **MUST** use backend bonuses: cuda=10, metal=9, rocm=8, vulkan=5, cpu=0.

### 5.2 Job Re-queue
- **MUST** re-queue on node dropout (up to 3 attempts).
- **MUST** treat KV cache failover as full restart from token 0 — no checkpoint resume.

## 6. CI / Build Invariants

### 6.1 Tests Must Pass
- **MUST** run `cargo test --workspace` before merge.
- **MUST** run `pnpm test` before merge.
- **MUST** run `pytest tests/ -v` before merge.
- **MUST NOT** merge with failing tests unless explicit admin override (document the reason in the merge commit).

### 6.2 Lint Must Pass
- **MUST** run `cargo fmt --check` + `cargo clippy --workspace -- -D warnings`.
- **MUST** run `pnpm lint` (ESLint v9 flat config).
- **MUST** run `ruff check .`.

### 6.3 Commit Messages
- **MUST** use conventional commits: `feat:`, `fix:`, `docs:`, `chore:`, `ci:`, `refactor:`, `test:`.
- **MUST** reference PR number in merge commit: `feat(...): ... (#NN)`.
- **MUST** include `Co-Authored-By:` trailer for AI-assisted commits.

### 6.4 Branch Hygiene
- **MUST** delete feature branches after merge.
- **MUST** squash-merge to main.
- **MUST NOT** push directly to main for feature work (docs-only direct pushes are acceptable for small onboarding/governance updates).

## 7. Documentation Invariants

### 7.1 Authority Map
- **MUST** update `index.md` when adding a new top-level doc.
- **MUST** update `log.md` when making significant doc changes.
- **MUST** update `.claude/memory/PROJECT.md` when project state changes.

### 7.2 Frontmatter
- **MUST** include YAML frontmatter (`type`, `title`, `description`, `tags`, `timestamp`, `status`, `phase`) on all root-level `.md` docs.

### 7.3 Cross-References
- **MUST** link to the authoritative source when mentioning a concept covered elsewhere.
- **MUST NOT** duplicate authoritative content in non-authoritative docs — link instead.
