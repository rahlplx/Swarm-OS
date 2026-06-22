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
│  │  4. For large models: split into K shards (ring topology)   │   │
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

```
Normal operation:
  Node Agent → writes /swarm/nodes/{id}/alive with TTL=10s every 5s

Node failure:
  1. Heartbeat stops
  2. etcd TTL expires after 10s
  3. etcd fires WATCH event: key deleted
  4. Scheduler receives event via etcd Watch stream
  5. Scheduler reads /jobs/{id}/shards → finds shards on dead node
  6. Scheduler marks those shards "failed"
  7. Scheduler re-assigns to next best available node
  8. Job RESTARTS from token 0 on the new node — NOT a checkpoint resume.
     The dead node's KV cache (/tmp/swarm/kv/{id}.bin) is unreachable.
     The etcd pointer /swarm/jobs/{id}/kvcache_ref is cleared.
     User-facing impact: up to 10s stall, then full re-generation from the beginning.

No single coordinator. Any orchestrator instance watching etcd handles it.
```

**Source:** k3s embedded etcd patterns (`k3s-io/k3s/pkg/etcd/`) + etcd official watch API docs

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

## 4. Model Sharding — Exo Ring Topology

For models exceeding a single node's VRAM (e.g., 70B model needs 40+ GB VRAM):

```
Model: Llama-3-70B (40 transformer layers)

Node A (24GB VRAM) → layers 0–19   (50%)
Node B (16GB VRAM) → layers 20–31  (30%)
Node C (8GB VRAM)  → layers 32–39  (20%)

Ring forward pass:
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

**Source pattern:** `exo-labs/exo/exo/inference/mlx/models/` shard assignment + `exo/topology/ring_memory_weighted_partitioning.py`

**Key difference from Exo:** We use GGUF/llama.cpp instead of MLX, enabling cross-platform (Windows/Linux/Mac).

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

**Source:** `BerriAI/litellm/litellm/proxy/` — proxy_server.py, router.py, custom_provider pattern

---

## 6. Observability Architecture

```
Every Node Agent (every 15s)
       │
       ▼ HTTP POST
Prometheus Pushgateway (central)
       │
       ▼ scrape every 30s
Prometheus Server
       │
       ├──► Grafana (dashboards — see ui_ux.md)
       └──► Alertmanager
                │
                ├──► Slack webhook (operator alerts)
                └──► Email (node dropout, queue overflow)
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
- Keys hashed (Argon2id, time=1, mem=64MB, parallelism=4, per-key random salt) at rest in etcd
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
