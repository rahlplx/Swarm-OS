---
type: spec
title: System Architecture
description: System layers, Blackboard coordination, scheduler/router, pipeline ring sharding, API gateway, observability, security, failure modes
tags: [architecture, security, networking, inference, scheduling, observability, technical]
timestamp: "2026-06-22"
status: active
phase: "0-4"
authority:
  - system_layers
  - blackboard_keyspace
  - scheduler_algorithm
  - pipeline_ring_topology
  - activation_tensor_sizes
  - api_gateway_flow
  - observability_architecture
  - security_model
  - failure_modes
depends_on: []
token_estimate: 5600
---

# Swarm-OS: Architecture

> Design principle: **No custom distributed systems code.** Every coordination primitive is stolen from a battle-tested OSS project. We assemble; we don't invent.

---

## 1. High-Level System Diagram

```
┌────────────────────────────────────────────────────────────────────┐
│                         CLIENT LAYER                               │
│                                                                    │
│   Any OpenAI SDK     Swarm-OS Web UI     Swarm-OS Tauri App       │
│   (Python/JS/etc)    (Next.js / React)   (Tray Agent)             │
└───────────────────────────────┬────────────────────────────────────┘
                                │ HTTPS + Server-Sent Events
┌───────────────────────────────▼────────────────────────────────────┐
│                      API GATEWAY LAYER                             │
│                                                                    │
│   ┌─────────────────────────────────────────────────────────────┐  │
│   │                    LiteLLM Proxy                            │  │
│   │  Auth → Rate Limit → Usage Track → Route → Stream Response  │  │
│   │  (BerriAI/litellm — steal proxy.py + router.py)            │  │
│   └─────────────────────────────┬───────────────────────────────┘  │
└─────────────────────────────────┼──────────────────────────────────┘
                                  │ gRPC / HTTP/2
┌─────────────────────────────────▼──────────────────────────────────┐
│                      ORCHESTRATOR CORE                             │
│                                                                    │
│  ┌─────────────┐   ┌──────────────────┐   ┌────────────────────┐  │
│  │  Job Queue  │   │   BLACKBOARD     │   │   Ledger Service   │  │
│  │  (etcd)     │◄─►│   (etcd v3)      │◄─►│   (append-only     │  │
│  │             │   │   /nodes/*       │   │    HMAC chain)     │  │
│  │  FIFO +     │   │   /jobs/*        │   │                    │  │
│  │  priority   │   │   /ledger/*      │   │  Ed25519 per entry │  │
│  └──────┬──────┘   └────────┬─────────┘   └────────────────────┘  │
│         │                   │                                       │
│  ┌──────▼───────────────────▼──────────────────────────────────┐   │
│  │                  SCHEDULER / ROUTER                         │   │
│  │                                                             │   │
│  │  1. Read node capabilities from Blackboard                  │   │
│  │  2. PRE-FILTER (hard gates — exclude before scoring):       │   │
│  │     - free_vram_gb < shard_min_vram  → exclude              │   │
│  │     - free_ram_gb  < shard_min_ram   → exclude              │   │
│  │     - backend incompatible with model → exclude             │   │
│  │     - node alive TTL expired         → exclude              │   │
│  │  3. Score eligible nodes: VRAM×4 + RAM×0.5 + CPU×0.25      │   │
│  │     + backend_bonus (cuda=10/metal=8/vulkan=5/cpu=0)        │   │
│  │     + locality_bonus (same ASN/city preferred)              │   │
│  │  4. For large models: split into K shards (pipeline ring)   │   │
│  │  5. Assign shards to top-K scoring nodes                    │   │
│  │  6. Write assignment to /jobs/{id}/shards in etcd           │   │
│  └─────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────┬─────────────────────────────────┘
                                   │
                     WireGuard P2P Mesh (headscale control plane)
                                   │
         ┌─────────────────────────┼──────────────────────────┐
         │                         │                           │
    ┌────▼──────┐          ┌───────▼────┐           ┌────────▼──────┐
    │  NODE A   │          │   NODE B   │           │    NODE C     │
    │ RTX 3090  │          │  i9 CPU    │           │  M2 MacBook   │
    │ 24GB VRAM │          │  No GPU    │           │  MPS/Metal    │
    │           │          │            │           │               │
    │ llama.cpp │◄────────►│ llama.cpp  │◄─────────►│  llama.cpp    │
    │ (CUDA)    │  ring    │ (CPU)      │   ring    │  (Metal)      │
    │           │  forward │            │   forward │               │
    │ Tauri     │          │  Tauri     │           │  Tauri Agent  │
    │ Agent     │          │  Agent     │           │               │
    └───────────┘          └────────────┘           └───────────────┘
```

---

## 2. The Blackboard Pattern (Deep Dive)

The Blackboard is a coordination pattern from classic distributed AI: agents share a mutable workspace without direct coupling. We implement it on top of etcd v3.

### Key Space Design

```
/swarm/
  nodes/
    {node_id}/
      caps          → {"vram_gb": 24, "ram_gb": 64, "cpu_cores": 16, "backend": "cuda", "score": 97.5}
      alive         → "1"  (TTL: 10 seconds — expires = node is dead)
      load          → {"active_jobs": 2, "tokens_per_sec": 45.2, "vram_used_gb": 18.1}
      meta          → {"version": "0.2.1", "join_time": "2025-01-15T08:00:00Z", "country": "BD"}
  jobs/
    {job_id}/
      request       → {model, job_token, max_tokens, stream}  // api_key and prompt stripped; delivered direct P2P to assigned nodes only
      status        → "queued" | "scheduled" | "running" | "streaming" | "done" | "failed"
      shards/
        {shard_idx} → {"node_id": "...", "layer_start": 0, "layer_end": 16, "status": "running"}
      result        → streaming token buffer reference
  ledger/
    {node_id}/
      head_hash     → "sha256_of_current_sqlite_wal_chain_head"  // deltas stored in SQLite WAL on orchestrator, not etcd
  config/
    scheduler/      → scheduling policy overrides
    rate_limits/    → per-tier API limits (one key per tier name)
    keys/
      {key_hash}    → {"credits": 120.50, "tier": "standard", "owner": "user@example.com"}
    models/         → model allowlist and per-model VRAM/backend requirements
    ledger/         → credit formula config (tokens_to_credits_ratio, debit costs)
    alerts/         → alerting rules and channel config
```

### Node Churn Handling

Normal operation: Node Agent writes `/swarm/nodes/{id}/alive` with TTL=10s every 5s.

| Step | Event | Action |
|------|-------|--------|
| 0 | Normal | Node writes `/alive` TTL=10s every 5s |
| 1 | Heartbeat stops | No action yet |
| 2 | 10s passes | etcd TTL expires, fires WATCH DELETE event |
| 3 | Scheduler receives event | Reads `/jobs/{id}/shards`, finds shards on dead node |
| 4 | Re-assignment | Marks dead shards "failed", assigns to next-best node |
| 5 | Restart | Job restarts from token 0 — dead node's KV cache unreachable, etcd pointer cleared. User sees up to 10s stall then full re-generation |

No single coordinator. Any orchestrator instance watching etcd handles it.

**Alternative studied (Petals):** DHT-based re-routing where the client caches activations locally and reroutes to an alternative block host, resuming from the failed boundary. Phase 2+ adaptation target — requires client-side activation caching.

**etcd watch caveat** (etcd/etcd#19179): Compaction can drop watch events. On `ErrCompacted`, the Orchestrator must: (1) fetch current state with point-in-time Get, (2) re-establish watch from returned revision. Missing a TTL expiry means a dead node is never cleaned up.

**Source:** k3s embedded etcd patterns (`k3s-io/k3s/pkg/etcd/`) + etcd official watch API docs + etcd/etcd#19179

---

## 3. Node Agent Architecture

Each participating device runs a Tauri v2 app. The Rust backend handles all compute; React renders the UI.

```
┌──────────────────────────────────────────────────────────────────┐
│                    NODE AGENT (Tauri v2)                         │
│                                                                  │
│  Rust Core (src-tauri/src/)                                      │
│  ┌────────────────┐  ┌─────────────────┐  ┌──────────────────┐  │
│  │ Resource       │  │ WireGuard Mgr   │  │ Blackboard       │  │
│  │ Profiler       │  │                 │  │ Client           │  │
│  │                │  │ headscale API   │  │                  │  │
│  │ sysinfo crate  │  │ wg-go bindings  │  │ etcd gRPC client │  │
│  │ nvml crate     │  │ (or wireguard-rs│  │ watch streams    │  │
│  │ metal crate    │  │  crate)         │  │ 5s heartbeat     │  │
│  └───────┬────────┘  └────────┬────────┘  └────────┬─────────┘  │
│          │                    │                     │             │
│  ┌───────▼────────────────────▼─────────────────────▼─────────┐  │
│  │                    Agent Controller                         │  │
│  │   (Tokio async runtime — receives job assignments via etcd) │  │
│  └───────────────────────────┬─────────────────────────────────┘  │
│                              │                                     │
│  ┌───────────────────────────▼─────────────────────────────────┐  │
│  │                 Inference Engine                             │  │
│  │                                                              │  │
│  │  llama-cpp-rs bindings (or llama-rs)                        │  │
│  │  Backend auto-select: CUDA → Metal → Vulkan → CPU           │  │
│  │  GGUF model loading from ~/.swarm-os/models/                 │  │
│  │  Model integrity: BLAKE3 hash (3-5× faster than SHA-256 on  │  │
│  │  4-7GB GGUFs); SHA-256 reserved for ledger chain only        │  │
│  │  KV cache written to local disk (/tmp/swarm/kv/); pointer    │  │
│  │  registered in etcd — never stored in etcd directly          │  │
│  └───────────────────────────┬─────────────────────────────────┘  │
│                              │                                     │
│  ┌───────────────────────────▼─────────────────────────────────┐  │
│  │              Metrics Exporter                                │  │
│  │  Prometheus /metrics endpoint (port 9100) — scraped directly │  │
│  │  by Prometheus server every 30s (no pushgateway needed for  │  │
│  │  long-running daemons; pushgateway is for batch/cron only)  │  │
│  │  Metrics: tokens/s, VRAM%, job count, queue depth           │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  React UI (src/  — Tauri webview)                               │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │  Tray Panel: Status | Credits Earned | Active Jobs           │  │
│  │  Settings: Resource Limit Slider | Model Download | API Keys │  │
│  │  Ledger: Credit history, top-up link                         │  │
│  └─────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 4. Model Sharding — Pipeline Ring Topology

For models exceeding a single node's VRAM (e.g., 70B model needs 40+ GB VRAM):

```
Model: Llama-3-70B (40 transformer layers)

Node A (24GB VRAM) → layers 0–19   (50%)
Node B (16GB VRAM) → layers 20–31  (30%)
Node C (8GB VRAM)  → layers 32–39  (20%)

Pipeline ring forward pass:
  Input Tokens
      │
      ▼
  Node A: embed + layers 0-19
      │ activations tensor (WireGuard P2P transfer)
      ▼
  Node B: layers 20-31
      │ activations tensor
      ▼
  Node C: layers 32-39 + lm_head → logits
      │ token stream
      ▼
  API Gateway → Client (SSE stream)
```

**Topology choice — pipeline ring (not STAR):**

| Reference | License | Topology | Adopted? | Reason |
|-----------|---------|----------|----------|--------|
| exo-labs/exo | GPL-3.0 | Pipeline ring | Study only | GPL incompatible; clean-room Rust impl required |
| b4rtaz/distributed-llama | MIT | STAR (matrix-parallel) | AVOID | 2^k node constraint, .bin format, 972ms/fwd on WAN |
| bigscience-workshop/petals | MIT | Pipeline chain | Study | DHT re-routing for Phase 2+ fault tolerance |

We implement ring partitioning from scratch in Rust (clean-room), since exo is GPL-3.0. The algorithm (weighted memory split across pipeline stages) is a standard technique in published ML systems papers.

### Activation Tensor Size (70B, hidden_dim=8192, fp16)

Formula: `Size = seq_len x hidden_dim x 2 bytes`

| Phase | seq_len | Size/hop | Time @ 50 Mbps (BD) | Time @ 1 Gbps (LAN) |
|-------|---------|----------|----------------------|----------------------|
| Decode | 1 | 16 KiB | 2.6 ms | negligible |
| Prefill | 2048 | 32 MiB | 5.12 s | 256 ms |

3 WAN hops at 50 Mbps = 15.4s prefill (exceeds 8s TTFT budget). LAN (>=1 Gbps): 768ms — acceptable.

**Phase 2 WAN mitigation options:**

| Option | Mechanism | Prefill overhead (3 hops) | Status |
|--------|-----------|---------------------------|--------|
| (a) LAN topology | >=1 Gbps campus/lab | 768 ms | Phase 2 target |
| (b) int8 activation quant | fp16 -> int8, 2x smaller | 7.7 s (borderline) | Phase 3+ |
| (c) Chunked prefill | Split seq_len into micro-batches | Varies | Phase 3+ |

**Source pattern (design reference only):** `exo-labs/exo/exo/topology/ring_memory_weighted_partitioning.py`

**Key difference from Exo:** We use GGUF/llama.cpp instead of MLX, enabling cross-platform (Windows/Linux/Mac/CPU).

---

## 5. API Gateway Data Flow

```
POST /v1/chat/completions
Authorization: Bearer swrm_sk_xxx
{
  "model": "llama-3-70b",
  "messages": [...],
  "stream": true
}
         │
         ▼
LiteLLM Proxy (litellm.acompletion router)
  1. Validate API key → lookup credits in etcd /config/keys/{key}
  2. Check rate limit → Redis sliding window counter
  3. Select model routing: "llama-3-70b" → swarm_provider plugin
  4. SwarmProvider.completion() → POST to Orchestrator gRPC
         │
         ▼
Orchestrator
  1. Write job to /swarm/jobs/{uuid}
  2. Scheduler picks nodes (within 200ms SLA)
  3. Job starts on nodes
         │
         ▼
SSE Stream back through LiteLLM → Client
  (LiteLLM handles SSE formatting, token counting)
         │
         ▼
Post-completion hook (LiteLLM success callback):
  - Debit credits from requester key
  - Write ledger delta for each contributing node
```

**Source:** `BerriAI/litellm/litellm/proxy/` — proxy_server.py, router.py, custom_provider pattern. See [tech_stack.md](./tech_stack.md) for LiteLLM version pinning and integration caveats.

---

## 6. Observability Architecture

```
Every Node Agent
  /metrics endpoint on port 9100 (Prometheus text format)
  Exposed on WireGuard mesh IP — NOT the public internet
       │
       ▼ HTTP GET every 30s via WireGuard mesh IPs
Prometheus Server (must be joined to the WireGuard mesh
  to reach nodes behind CGNAT — scrapes mesh IPs directly)
       │
       ├──► Grafana (dashboards — see ui_ux.md)
       └──► Alertmanager
                │
                ├──► Slack webhook (operator alerts)
                └──► Email (node dropout, queue overflow)
```

**Network requirement:** Prometheus must join the headscale mesh (register as a non-contributing node) so it can reach each node agent's WireGuard IP on port 9100. Nodes behind CGNAT are unreachable from the public internet — direct scrape only works via the mesh. No Pushgateway is needed; Pushgateway is for short-lived batch jobs, not long-running daemons.

**Dynamic service discovery:** Static `scrape_configs` won't scale — WireGuard IPs change as nodes join/leave. Use `file_sd_config`: Orchestrator watches etcd `/swarm/nodes/*`, rewrites `/etc/prometheus/targets/swarm_nodes.json` on join/drop, Prometheus polls every 30s. Alternative: `http_sd_config` via Orchestrator endpoint.

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'swarm_nodes'
    file_sd_configs:
      - files: ['/etc/prometheus/targets/swarm_nodes.json']
        refresh_interval: 30s
```

### Custom Metrics

| Metric Name | Type | Description |
|-------------|------|-------------|
| `swarm_nodes_active` | Gauge | Live nodes in blackboard |
| `swarm_tokens_per_second` | Gauge | Aggregate inference speed |
| `swarm_job_queue_depth` | Gauge | Waiting jobs |
| `swarm_node_vram_used_bytes` | Gauge | Per-node VRAM usage |
| `swarm_tokens_total` | Counter | All-time tokens generated |
| `swarm_credits_issued_total` | Counter | All-time credits issued |
| `swarm_job_duration_seconds` | Histogram | P50/P95/P99 latency |
| `swarm_node_score` | Gauge | Per-node capability score |
| `swarm_ledger_chain_valid` | Gauge | 1=chain intact, 0=break detected (triggers ledger_chain_break alert) |

---

## 7. Security Architecture

### Node Authentication
- Every node gets an Ed25519 keypair on first run
- Public key registered in Blackboard at join time
- All ledger entries signed by originating node's private key
- Orchestrator verifies signatures before writing ledger

### API Key Security
- Keys hashed (Argon2id, time=3, mem=64MB, parallelism=4, per-key random salt) at rest in etcd — OWASP 2025 minimum is time=3 (time=1 is insufficient against modern hardware)
- Keys prefixed: `swrm_sk_` (consumer), `swrm_ops_` (operator), `swrm_adm_` (super admin), `swrm_node_` (inter-node)
- Ed25519 signature chain for credits deduction log — each entry contains `SHA-256(prev_entry)` + node's Ed25519 signature; tamper-evident without a shared secret

### Network Security
- All inter-node traffic: WireGuard (ChaCha20-Poly1305)
- API Gateway: TLS 1.3, rate limiting, IP allowlist option
- Node agent communicates with Orchestrator only over mesh
- Blackboard (etcd): client certificate auth, not exposed to internet

### Isolation
- Inference runs in a sandboxed subprocess (seccomp on Linux, App Sandbox on macOS)
- No node can read another node's model weights or KV cache directly
- Jobs are ephemeral: all state cleared after completion

Scheduler policy config schema: [governance.md §3.1](./governance.md). Threat analysis: [critique.md Domain 2](./critique.md).

---

## 8. Failure Mode Matrix

| Failure | Detection | Recovery | Source Pattern |
|---------|-----------|----------|----------------|
| Node dropout mid-job | etcd TTL expiry (10s) | Re-queue shards | k3s etcd watch |
| Orchestrator crash | etcd leader election | Standby takes over | etcd Raft |
| Network partition | DERP relay fallback | Route via relay | Tailscale DERP |
| Model download fail | Checksum verify | Retry + alternate mirror | GPUStack model mgr |
| Ledger corruption | Ed25519 signature chain break (hash mismatch) | Alert + audit rollback from last valid entry | Bitcoin chain concept |
| Credit exhaustion | Pre-flight credit check | 402 response, notify user | LiteLLM budget mgr |
| Queue overflow | Depth metric alert | Reject with 503 + retry-after | LiteLLM rate limiter |
