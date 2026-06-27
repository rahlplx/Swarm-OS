# AGENTS.md — Swarm-OS Project Memory

## 🎯 Primary Goal
Transform Swarm-OS from a centralized MVP to a **Federated P2P AI Inference Fabric** that survives the "Dirty Network" reality of Bangladesh ISPs.

## 🏗️ Architecture Blueprint
- **Networking:** libp2p Gossip Mesh (replacing etcd) -> headscale bootstrap.
- **Trust:** Merkle-DAG Ledger + Argon2id (t=10, m=256MB).
- **Inference:** Hybrid CPU/GPU Sharding (Clean-room Rust) -> llama.cpp backend.
- **Payments:** SSLCommerz -> Offline-First Credit Sync (CCTs).

## 📜 The Iron Laws
1. **TDD First:** No production code without a failing test.
2. **License Hygiene:** Strictly Apache 2.0. NO GPL-3.0 code (exo-labs/exo is reference only).
3. **Atomic Commits:** Changes to `crates/` and `control-plane/` must be merged as one unit.
4. **Evidence-Based Decisions:** All architectural changes must be backed by telemetry or benchmarks (eBPF/Prometheus).

## 🗺️ Authority Map (OKF)
| Concept | Authoritative Source |
|---|---|
| Scheduler Algorithm | architecture.md |
| Credit Formula | governance.md |
| Security Controls | architecture.md §7 |
| BD-Network Metrics | research.md |

## 🔄 Loop Engineering Workflow
Plan -> Break -> Build -> Harness -> Review -> Ship -> Learn -> Evolve