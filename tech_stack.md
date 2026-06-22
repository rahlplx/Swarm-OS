# Swarm-OS: Tech Stack

> Rule: **Steal like an artist.** Every component is sourced from a production-grade OSS repo. No custom distributed systems primitives. Links + specific files cited for every component.

---

## OSS Repos to Integrate (Steal From)

### Tier 1 — Core Architecture (Must Have)

#### 1. exo-labs/exo
- **Repo:** https://github.com/exo-labs/exo
- **What We Steal:**
  - Ring topology model sharding: `exo/topology/ring_memory_weighted_partitioning.py`
  - P2P node discovery: `exo/networking/udp_discovery.py`
  - Shard inference protocol: `exo/inference/shard_inference_engine.py`
  - Device capability detection: `exo/helpers.py` (memory profiling)
- **How We Adapt:** Port the topology and sharding logic to Rust; replace MLX with llama.cpp GGUF backend for cross-platform support (Exo is Apple-only)
- **License:** GPL-3.0

#### 2. ggerganov/llama.cpp
- **Repo:** https://github.com/ggerganov/llama.cpp
- **What We Steal:**
  - The entire inference engine — GGUF model format, CUDA/Metal/CPU backends
  - Server mode: `examples/server/server.cpp` (OpenAI-compatible REST server per node)
  - Splitting: `src/llama.cpp` tensor offloading across GPUs (adapt for cross-node)
- **Rust Binding:** `utilityai/llama-rs` or `mdrokz/rust-llama.cpp`
- **License:** MIT

#### 3. BerriAI/litellm
- **Repo:** https://github.com/BerriAI/litellm
- **What We Steal:**
  - Proxy server: `litellm/proxy/proxy_server.py` — auth, rate limiting, routing
  - Budget manager: `litellm/proxy/budget_manager.py` — credit deduction logic
  - Custom provider pattern: `litellm/main.py` custom_llm_provider — we add "swarm" provider
  - Usage tracking: `litellm/proxy/utils.py` log_success_event callback
  - OpenAI-compatible SSE streaming
- **License:** MIT

#### 4. juanfont/headscale
- **Repo:** https://github.com/juanfont/headscale
- **What We Steal:**
  - Entire self-hosted Tailscale control plane
  - Node registration: `hscontrol/` registration flow
  - DERP relay server: `derp/` for NAT traversal fallback
  - ACL policy engine: for node isolation policies
- **Integration:** Embed headscale as a Go binary; Node Agent calls headscale API to register
- **License:** BSD-3-Clause

#### 5. gpustack/gpustack
- **Repo:** https://github.com/gpustack/gpustack
- **What We Steal:**
  - Worker resource profiling: `gpustack/worker/collector.py` — GPU/CPU/RAM detection
  - Model distribution: `gpustack/scheduler/` — scheduler patterns
  - Admin UI design patterns: `gpustack/ui/` — React admin panel layout
  - Model file management: download, hash-verify, store in local cache
- **License:** Apache-2.0

---

### Tier 2 — Infrastructure (Use As-Is)

#### 6. etcd-io/etcd
- **Repo:** https://github.com/etcd-io/etcd
- **Role:** The Blackboard — distributed KV store for all swarm state
- **Usage:** Embed as a sidecar via `go.etcd.io/etcd/client/v3` Go SDK; node agents use Rust etcd client (`etcd-client` crate)
- **Why not Redis?** etcd's watch streams + TTL + Raft consensus are a perfect match for node heartbeat pattern. Redis Pub/Sub lacks strong consistency guarantees.
- **License:** Apache-2.0

#### 7. prometheus/prometheus + node_exporter
- **Repo:** https://github.com/prometheus/prometheus
- **Role:** Metrics collection from all nodes
- **Usage:**
  - Each node agent pushes to Prometheus Pushgateway (for devices behind NAT)
  - Prometheus scrapes pushgateway every 30s
  - Node exporter on each device for system-level metrics
- **License:** Apache-2.0

#### 8. grafana/grafana
- **Repo:** https://github.com/grafana/grafana
- **Role:** Analytics dashboards
- **Pre-built Dashboards We Adapt:**
  - Dashboard ID 1860: Node Exporter Full (adapt for swarm nodes)
  - Dashboard ID 15172: Ollama metrics (adapt for llama.cpp metrics)
- **License:** AGPL-3.0 (use via Grafana Cloud free tier or self-host)

#### 9. tauri-apps/tauri
- **Repo:** https://github.com/tauri-apps/tauri
- **Role:** Desktop app shell (Node Agent UI)
- **Why Tauri over Electron:**
  - 15MB binary vs 150MB Electron
  - Rust backend = no Node.js runtime
  - Lower RAM: ~50MB idle vs ~200MB Electron
  - Native system tray, notifications
- **Version:** Tauri v2 (stable as of 2024)
- **License:** MIT / Apache-2.0

#### 10. wireguard/wireguard-go
- **Repo:** https://github.com/WireGuard/wireguard-go
- **Role:** P2P encrypted mesh transport
- **Integration:** headscale manages the control plane; WireGuard handles data plane
- **BD Note:** WireGuard handles CGNAT better than OpenVPN via headscale DERP relay
- **License:** MIT

---

### Tier 3 — Application Layer

#### 11. shadcn/ui
- **Repo:** https://github.com/shadcn-ui/ui
- **Role:** React component library for Tauri UI + Admin portal
- **What We Use:** Button, Card, Table, Badge, Dialog, Slider, Chart (Recharts wrapper)
- **Why:** Zero runtime dependency (copies components), Tailwind-native, dark mode first
- **License:** MIT

#### 12. vercel/next.js (Admin Portal)
- **Repo:** https://github.com/vercel/next.js
- **Role:** Admin governance portal (web app, separate from Tauri)
- **Deployment:** Static export → Cloudflare Pages (free tier)
- **License:** MIT

#### 13. tokio-rs/tokio
- **Repo:** https://github.com/tokio-rs/tokio
- **Role:** Async runtime for all Rust backend code in Node Agent
- **Why:** Industry-standard Rust async; used by headscale, Tauri, etc.
- **License:** MIT

#### 14. seanmonstar/reqwest
- **Repo:** https://github.com/seanmonstar/reqwest
- **Role:** HTTP client in Rust for Node Agent → Orchestrator API calls
- **License:** MIT / Apache-2.0

#### 15. open-telemetry/opentelemetry-rust
- **Repo:** https://github.com/open-telemetry/opentelemetry-rust
- **Role:** Distributed tracing for job lifecycle across nodes
- **Integration:** Export traces to Grafana Tempo (free cloud tier)
- **License:** Apache-2.0

---

### Tier 4 — BD-Specific

#### 16. SSLCommerz (Payment Gateway)
- **Repo:** https://github.com/sslcommerz/SSLCommerz-Python (adapt to Next.js)
- **Role:** BDT credit purchases (bKash, Nagad, Visa, MasterCard)
- **Integration:** Admin portal checkout → SSLCommerz → webhook → credit top-up in etcd
- **License:** Proprietary SDK (free to use)

#### 17. i18next/react-i18next
- **Repo:** https://github.com/i18next/react-i18next
- **Role:** Bangla/English UI localization
- **License:** MIT

---

## Full Dependency Map

```
Swarm-OS
├── Node Agent (Rust/Tauri)
│   ├── tauri v2 (app shell)
│   ├── tokio (async runtime)
│   ├── llama-cpp-rs (inference)
│   ├── etcd-client (blackboard)
│   ├── reqwest (HTTP)
│   ├── sysinfo (resource profiling)
│   ├── nvml-wrapper (NVIDIA GPU)
│   └── opentelemetry (tracing)
│
├── Orchestrator (Rust or Go service)
│   ├── etcd v3 (blackboard state)
│   ├── tonic (gRPC server)
│   ├── tokio (async)
│   └── exo topology logic (ported)
│
├── API Gateway (Python)
│   ├── litellm proxy
│   ├── fastapi (if customizing)
│   └── redis (rate limit counters)
│
├── Mesh Control Plane
│   ├── headscale (Go binary, embedded)
│   └── wireguard-go (data plane)
│
├── Observability
│   ├── prometheus
│   ├── prometheus/node_exporter
│   ├── prometheus/pushgateway
│   └── grafana
│
└── Admin Portal (TypeScript/Next.js)
    ├── next.js
    ├── shadcn/ui
    ├── react-i18next
    ├── recharts (analytics)
    └── SSLCommerz SDK (payments)
```

---

## Language Breakdown

| Layer | Language | Rationale |
|-------|----------|-----------|
| Node Agent Core | Rust | Memory safety, zero-cost async, Tauri native |
| Inference Engine | C++ (llama.cpp) via Rust FFI | Best GPU performance, GGUF ecosystem |
| Orchestrator | Rust (or Go) | Performance + type safety; Go if headscale integration is simpler |
| API Gateway | Python | LiteLLM is Python; don't rewrite it |
| Mesh Control | Go | headscale is Go |
| Admin Portal | TypeScript/React | Next.js + shadcn is fastest for admin UIs |
| Observability Config | YAML | Prometheus/Grafana native |

---

## Model Support Matrix

| Model Family | Size | Min VRAM | Quantization | Backend |
|-------------|------|----------|--------------|---------|
| Llama 3.2 | 1B–3B | 2GB | Q4_K_M | CPU/CUDA/Metal |
| Llama 3.1 | 8B | 6GB | Q4_K_M | CUDA/Metal/CPU |
| Mistral 7B | 7B | 5GB | Q4_K_M | CUDA/Metal/CPU |
| Qwen 2.5 | 7B–14B | 6–10GB | Q4_K_M | CUDA/Metal/CPU |
| Phi-3.5 | 3.8B | 3GB | Q4_K_M | CPU/CUDA/Metal |
| DeepSeek-R1 | 7B–70B | 6GB–40GB | Q4_K_M | CUDA (sharded) |
| Llama 3.1 | 70B | 40GB | Q4_K_M | Multi-node shard |
| Llama 3.1 | 405B | 200GB | Q4_K_M | 5+ node shard |

---

## Build Toolchain

```
Language    Tool            Version
─────────────────────────────────
Rust        cargo           1.78+
C++         cmake + clang   3.28+
Go          go toolchain    1.22+
Python      uv (Astral)     0.4+
TypeScript  pnpm + tsc      9+
Bundler     Vite            5+
Container   Docker          26+
Compose     docker compose  2.25+
CI          GitHub Actions  —
```
