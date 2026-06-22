---
type: context
title: Swarm-OS README
description: Project overview, architecture summary, credit economy, roadmap
tags: [planning]
timestamp: "2026-06-22"
status: active
phase: "0-4"
token_estimate: 900
---

# Swarm-OS

> Your idle GPU earns. Your AI runs free.

Swarm-OS is a decentralized P2P AI inference network. Contributors pool idle GPU/CPU compute into a mesh; consumers run 7B–405B models at zero recurring cloud cost. Everyone earns credit for what they give; everyone spends credit for what they consume.

Made in Bangladesh, built for the world. Apache 2.0.

---

## Planning Docs

See [index.md](./index.md) for the full document map with authority assignments and reading paths.

Credit formula canonical source: [governance.md §3.4](./governance.md#34--ledger-policy).

---

## Architecture at a Glance

```
Client (OpenAI SDK / Web Portal / Tauri Agent)
    │ HTTPS + SSE
LiteLLM Proxy (API Gateway — auth, rate limit, usage)
    │ gRPC
Orchestrator (Scheduler + Blackboard client)
    │ etcd Blackboard (/swarm/nodes, /swarm/jobs, /swarm/config)
WireGuard P2P Mesh (headscale control plane)
    │
Node A (CUDA)  ──ring──  Node B (CPU)  ──ring──  Node C (Metal)
llama.cpp               llama.cpp               llama.cpp
```

Key design choices:
- **No custom distributed systems code** — every primitive is stolen from a battle-tested OSS project
- **etcd** for live coordination (node heartbeat, job routing); **SQLite WAL** for the tamper-evident credit ledger
- **llama.cpp** (GGUF) as the universal inference backend; cross-node sharding via ring pipeline parallelism
- **Ed25519** signature chain on every ledger entry — tamper-evident without a shared secret

---

## Credit Economy

| Direction | Formula |
|-----------|---------|
| Contributor earns | `tokens_generated × 0.008` (× 1.1 if node score > 80) |
| Consumer spends | `(input_tokens × 0.006) + (output_tokens × 0.01)` |
| Rule of thumb | 1 credit ≈ 100 output tokens |
| BDT top-up | ৳50 → 125 cr · ৳200 → 500 cr · ৳450 → 1,201 cr (+7%) |

Platform retains the ~20% spread between earn and spend as operational reserve.

---

## Roadmap

| Phase | Timeline | Goal |
|-------|----------|------|
| 0 | Weeks 1–4 | Local alpha: single device, single model, OpenAI-compatible API |
| 1 | Weeks 5–10 | Two-node swarm: WireGuard mesh, etcd Blackboard, basic scheduler |
| 2 | Weeks 11–18 | Heterogeneous pool: 5–20 nodes, model sharding, node churn handling |
| 3 | Weeks 19–24 | Observability & governance: Grafana, admin portal, ledger audit |
| 4 | Weeks 25–32 | BD production launch: SSLCommerz/bKash, Bangla UI, OSS release |

---

## Quick Start (Phase 0 target)

```python
from openai import OpenAI

client = OpenAI(
    api_key="swrm_sk_...",
    base_url="https://api.swarm-os.dev/v1"
)

response = client.chat.completions.create(
    model="llama-3.1-8b",
    messages=[{"role": "user", "content": "Hello from Bangladesh!"}]
)
```

---

## License

Apache 2.0 — see [LICENSE](./LICENSE).
