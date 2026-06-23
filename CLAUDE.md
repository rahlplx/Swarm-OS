---
type: context
title: Claude Code Onboarding
description: AI assistant context for the Swarm-OS planning repo
tags: [architecture, planning]
timestamp: "2026-06-22"
status: active
phase: "0-4"
token_estimate: 1900
---

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Swarm-OS is a decentralized P2P AI inference network. Contributors pool idle GPU/CPU compute into a WireGuard mesh; consumers run 7B–405B models via an OpenAI-compatible API. A tamper-evident credit ledger tracks earn/spend. Made in Bangladesh (BDT payments via SSLCommerz/bKash), Apache 2.0.

**Current state:** Phase 0 in progress — single-device local AI inference app (Tauri v2 + llama-cpp-2 + LiteLLM).

## Architecture

The system has five layers, each built from battle-tested OSS (no custom distributed systems primitives):

1. **Client Layer** — Any OpenAI SDK, Next.js web portal, Tauri v2 tray agent
2. **API Gateway** — LiteLLM proxy (Python): auth, rate limiting, usage tracking, SSE streaming. Custom `swarm` provider plugin routes to orchestrator
3. **Orchestrator** — Rust (or Go): job queue, scheduler/router, ledger service. Two-phase scheduling: hard-gate pre-filter → weighted capability scoring (`vram×4 + ram×0.5 + cpu×0.25 + backend_bonus + locality_bonus`)
4. **Blackboard** — etcd v3 for coordination (`/swarm/nodes/*/alive` TTL=10s, `/swarm/jobs/*`, `/swarm/config/*`). Ledger stored in SQLite WAL on orchestrator (NOT etcd — compaction breaks hash chains). etcd only holds `/swarm/ledger/{node_id}/head_hash` pointer
5. **Node Mesh** — WireGuard P2P (headscale control plane), llama.cpp inference (GGUF), pipeline ring topology for cross-node model sharding

### Key Design Constraints

- **Prompts never enter etcd.** Job payloads contain only routing metadata (model, job_token, max_tokens). Prompts delivered P2P to assigned nodes
- **API keys never enter job payloads.** Use one-time job authorization tokens with 60s TTL
- **Cross-WAN 70B sharding is not viable on BD broadband.** Activation tensors are ~16 KiB/hop at seq_len=1 (decode) but ~32 MiB/hop at seq_len=2048 (prefill). Prefill is the bottleneck: 5.12s/hop at 50Mbps. Phase 2 targets LAN topology (≥1Gbps). WAN sharding requires int8 activation quantization or chunked prefill (Phase 3+)
- **KV cache failover = full restart from token 0**, not checkpoint resume. Dead node's cache is unreachable. Petals' DHT-based re-routing (client caches activations, reroutes to alternative block host) is a Phase 2+ adaptation target
- **etcd watch streams can miss events** on compaction (ErrCompacted). Re-watch logic with point-in-time Get is mandatory

## Planned Build Toolchain

| Language | Tool | Version |
|----------|------|---------|
| Rust | cargo | 1.78+ |
| C++ | cmake + clang | 3.28+ |
| Go | go toolchain | 1.22+ |
| Python | uv (Astral) | 0.4+ |
| TypeScript | pnpm + tsc | 9+ |
| Bundler | Vite | 5+ |
| Container | Docker + compose | 26+ / 2.25+ |

## Component → Language Map

- **Node Agent** (Tauri v2): Rust backend (`tokio`, `llama-cpp-2`, `etcd-client`, `sysinfo`, `nvml-wrapper`, `ed25519-dalek`) + React frontend
- **Orchestrator**: Rust (using `tonic` gRPC) or Go (using `grpc-go`), etcd, ring topology logic
- **API Gateway**: Python (LiteLLM, FastAPI, Redis)
- **Mesh Control Plane**: Go (headscale + wireguard-go)
- **Admin Portal**: TypeScript/Next.js + shadcn/ui + react-i18next
- **Observability**: Prometheus, Alertmanager, Grafana (YAML config)

## Phase Roadmap

- **Phase 0** (Weeks 1–4): Single device, single model, OpenAI-compatible API. Tauri + llama.cpp + LiteLLM. No networking, no etcd, no ledger
- **Phase 1** (Weeks 5–10): Two-node swarm. WireGuard mesh, etcd Blackboard, basic scheduler, ledger v1
- **Phase 2** (Weeks 11–18): 5–20 nodes, model sharding (pipeline ring), node churn, mDNS discovery
- **Phase 3** (Weeks 19–24): Observability, admin portal, ledger audit
- **Phase 4** (Weeks 25–32): BD production launch, SSLCommerz/bKash, Bangla UI, P2P model distribution

## Security Invariants

- API keys hashed with **Argon2id** (time=3, mem=64MB, parallelism=4, per-key salt) — not SHA-256
- Ledger entries signed with **Ed25519** per node; hash chain uses SHA-256. `credits_delta` stored as integer **microcredits** (1 credit = 1,000,000) to avoid float non-determinism in hashes
- ADMIN_ADJUST entries go in a **separate SQLite table** — never in the hash chain. Each individually signed by operator key
- Key prefixes: `swrm_sk_` (consumer), `swrm_ops_` (operator), `swrm_adm_` (super admin), `swrm_node_` (inter-node)
- Hardware fingerprint for sock-puppet prevention: `SHA-256(gpu_uuid:motherboard_serial:cpu_id:machine_id)` with `":"` delimiters. `/etc/machine-id` is primary anchor; OEM sentinel strings excluded

## Credit Economy

- Contributor earns: `tokens × 0.008` (×1.1 if node_score > 80)
- Consumer spends: `(input_tokens × 0.006) + (output_tokens × 0.01)`
- ~20% platform spread. 1 credit ≈ 100 output tokens
- Abuse cap: `max_credits_per_hour = node_score × 2`

## License Constraints

- **exo-labs/exo is GPL-3.0** — reference/study only, no code porting. Clean-room Rust implementation of ring topology required
- **Grafana is AGPL-3.0** — use via self-host or cloud tier; don't link into our codebase
- **b4rtaz/distributed-llama is MIT but AVOID** — matrix-parallel (not pipeline-parallel), 2^k node constraint, custom `.bin` format (not GGUF), 972ms/forward-pass on WAN. Not suitable for Swarm-OS
- **Cross-node sharding**: Clean-room Rust pipeline-parallel implementation. Study exo ring partitioning algorithm + Petals DHT-based fault tolerance as design references
- All Swarm-OS code: Apache 2.0

## UI Design System

- Dark mode first. Colors: primary `#6366F1` (indigo), bg `#0F0F11`, surface `#1A1A1F`
- Fonts: Inter (Latin) + Hind Siliguri (Bangla). Western numerals in Bangla mode
- Component library: shadcn/ui + Tailwind. Radius: 8px components, 12px cards
- Separate post-login dashboards for contributors vs consumers

## Key Research Findings

The `research.md` report contains detailed benchmarks and code samples. Key numbers:

| Metric | Value |
|--------|-------|
| Rust binding | `llama-cpp-2` (utilityai) — `llama-rs` is archived, no GGUF support |
| 7B Q4_K_M VRAM (warm) | 4.42 GiB (target headroom: 5.20 GiB) |
| 70B Q4_K_M VRAM (warm) | 43.12 GiB (target headroom: 48.00 GiB) |
| Metal context limit | 8192 tokens (crashes above due to Unified Memory 75% limit) |
| RTX 4090 decode throughput | 88.5 tokens/sec |
| M3 Max decode throughput | 48.2 tokens/sec |
| CPU (i9-14900K) decode | 8.1 tokens/sec |
| etcd lease expiry latency | ~5.11s avg (150ms after TTL) |
| WireGuard CGNAT tunnel setup | 1200ms direct, 3500ms DERP fallback |
| Tauri IPC roundtrip | 1.42ms macOS, 2.10ms Windows |
| Tauri binary (LTO+strip) | 7.1–9.4 MiB across platforms |
| BLAKE3 vs SHA-256 (4 GiB GGUF) | 1.45s vs 11.75s |
| LiteLLM version pin | ≥1.82.0 (CustomLogger hooks are unstable internal APIs) |
| iroh model distribution | ADOPT — QUIC transport, auto-resume, NAT traversal |
| distributed-llama | AVOID — matrix-parallel, 2^k nodes, .bin format, 972ms/fwd on WAN |

## Planning Documents

See [index.md](./index.md) for the complete document map with:
- Authority assignments (which file is canonical for each concept)
- Reading paths by phase and role
- Token estimates for context budgeting

Key files for Phase 0: project.md, architecture.md (§1-5, §7), tech_stack.md (Tier 1-2), ui_ux.md, guide.md (Part 2).

## Document Authority (OKF)

When information appears in multiple docs, the authoritative source is:
- Scheduler algorithm → architecture.md §3
- Credit formula / ledger config → governance.md §3.4
- Security controls (Argon2id, Ed25519, key prefixes) → architecture.md §7
- Ledger entry format / audit → governance.md §4
- Hardware fingerprint → governance.md §5.3
- Activation tensor sizing → architecture.md §4
- OSS dependency details → tech_stack.md
- Benchmark numbers → research.md

When docs conflict, the authoritative source wins. See [index.md](./index.md) for the full authority map.

## Phase 0 Development

### Test Commands

```bash
# Rust (31 unit + integration tests)
cargo test --workspace

# React (19 tests via Vitest)
pnpm test

# Python (LiteLLM proxy)
cd litellm-proxy && python3 -m pytest tests/ -v

# Type checks
cargo check --workspace
pnpm exec tsc --noEmit

# Day 1 acceptance
bash scripts/acceptance/day01.sh

# Full environment verify
bash scripts/verify-environment.sh

# Telemetry report
bash scripts/telemetry-report.sh
```

### TDD Workflow

Contract tests define interfaces (Rust traits: `GpuDetector`, `InferenceEngine`) → implementation satisfies them → integration tests verify connections. Each day's guide.md Chapter 9 deliverable maps to `scripts/acceptance/dayNN.sh`.

- **Rust**: Traits for mocking (`GpuDetector`, `InferenceEngine`). GPU tests behind `#[cfg(feature = "gpu-tests")]`
- **React**: Tauri IPC mocked via `src/lib/tauri-mock.ts`. Tests co-located (`*.test.tsx`)
- **Python**: `respx` for HTTP mocking. Mock llama-server in `conftest.py`

### Auto-Trigger Harness

- Every 5th query: `cargo check` + `tsc --noEmit` (via `.claude/settings.json` hooks)
- Session end: full test suite + telemetry collection
- Git pre-commit: `cargo fmt --check` + `clippy` + `tsc` + `ruff`
- Git pre-push: full test suite + binary size check (< 10 MiB)
