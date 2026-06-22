# Swarm-OS: Project Specification

> **The Story:** Every night, millions of GPUs sleep unused inside gaming rigs, workstations, and idle servers worldwide. Swarm-OS wakes them up. Contributors pool idle compute into a P2P mesh; consumers tap it to run 7B–70B models at zero recurring cloud cost. Everyone earns credit for what they give; everyone spends credit for what they consume. No AWS bill. No single point of failure. Made in Bangladesh, built for the world.

---

## Product Identity

| Field | Value |
|-------|-------|
| **Product Name** | Swarm-OS |
| **Tagline** | Your idle GPU earns. Your AI runs free. |
| **Origin** | Bangladesh |
| **Category** | Decentralized AI Inference Network |
| **Primary OSS Comps** | exo-labs/exo · GPUStack · headscale · LiteLLM · llama.cpp |
| **UI Shell** | Tauri v2 (Rust backend) + React (frontend) |

---

## Market Context

### The Problem
- ChatGPT Plus = $20/mo → ~BDT 2,200/mo → unaffordable for most BD users
- Running local LLMs requires high-end hardware most people don't have alone
- Cloud GPU rental (RunPod, Lambda) = $0.50–$2/hr → expensive for continuous use
- Existing P2P solutions (Bittensor, Akash) are crypto-first, complex, and inaccessible

### The Opportunity (BD-specific)
- BD has 18M+ freelancers and a fast-growing developer ecosystem
- Millions of gaming PCs, student laptops, and server rooms sit idle at night
- No Bangladeshi AI infrastructure product exists at the OS/platform level
- BASIS, ICT Division, and a2i are actively funding local AI innovation

### Competitive Landscape

| Product | What It Does | Our Edge |
|---------|-------------|----------|
| exo-labs/exo | P2P inference across Apple devices | We add governance, ledger, BD payment, mixed hardware |
| GPUStack | GPU cluster management | We add P2P mesh, contribution economy |
| Bittensor | Crypto-incentivized AI network | No crypto required; simpler UX |
| LM Studio | Local LLM runner | No pooling; single device only |
| Ollama | Local model server | No sharing; no swarm |
| Together.ai / Groq | Cloud free-tier LLM APIs | We're the infrastructure those could run on |

---

## Core Features

### F1: Heterogeneous Resource Pooling
- Auto-detect CPU cores, RAM, VRAM, GPU model via `sysinfo` + `nvml` Rust crates
- Capability Scoring: `score = (vram_gb × 4) + (ram_gb × 0.5) + (cpu_cores × 0.25) + backend_bonus`
- Support: NVIDIA CUDA, AMD ROCm, Apple Metal/MPS, CPU-only (via llama.cpp backends)
- **Stolen from:** GPUStack worker profiler (`gpustack/gpustack/worker/`)

### F2: P2P Mesh Networking (Zero-Config)
- WireGuard overlay via embedded headscale (self-hosted Tailscale control plane)
- Nodes join with a single token; full mesh auto-configured
- NAT traversal, DERP relay fallback for BD ISP quirks (CGNAT is common)
- **Stolen from:** tailscale/tailscale control plane + juanfont/headscale

### F3: Distributed State — The Blackboard
- etcd v3 cluster as the shared "Blackboard" (classic AI coordination pattern)
- Keys: `/nodes/{id}/caps`, `/nodes/{id}/alive` (5s TTL), `/jobs/{id}/status`, `/ledger/{id}/delta`
- Node dropout = TTL expiry → Scheduler re-assigns automatically
- **Stolen from:** etcd usage patterns in k3s + GPUStack scheduler

### F4: AI Inference Engine
- llama.cpp as the universal backend (GGUF models)
- Rust bindings via `llama-rs` or `llama-cpp-rs`
- Model sharding across nodes using Exo's ring topology for 70B+ models
- Supported model families: Llama 3, Mistral, Qwen, Gemma, Phi
- **Stolen from:** exo-labs/exo shard logic, ggerganov/llama.cpp

### F5: OpenAI-Compatible API Gateway
- LiteLLM proxy as the API facade — drop-in for any OpenAI SDK client
- Per-key auth, rate limiting, usage tracking
- Routes to swarm via custom provider plugin
- **Stolen from:** BerriAI/litellm proxy patterns

### F6: Contribution Ledger
- Every node tracks: `tokens_generated` vs. `compute_units_spent`
- Ledger entries are append-only, timestamped, node-signed (Ed25519)
- Credit formula: `credits_earned = tokens_generated × node_score_weight`
- `credits_spent = input_tokens + output_tokens × 1.5`
- Optional future: export ledger to bKash/Nagad micropayment rails

### F7: Observability Stack
- Prometheus node_exporter on every node agent
- Custom metrics: `swarm_tokens_per_second`, `swarm_node_vram_used`, `swarm_job_queue_depth`
- Grafana dashboards: hourly/weekly capacity, Special Days view (Eid peaks, exams)
- Alertmanager: node dropout alerts, queue overflow, P95 latency > 5s

### F8: Tauri Desktop Agent
- System tray app: Join Swarm | Pause | View Ledger
- Resource throttle slider: "donate 50% of GPU"
- Live stats: tokens generated today, credits earned, active jobs
- Minimal footprint: ~15MB binary, <50MB RAM when idle

### F9: Admin Governance Portal
- Web dashboard for swarm operators
- Node whitelist/blacklist, capability overrides
- API key management, rate limit tiers
- Ledger audit export (CSV / JSON)
- BD-specific: BDT display, Bangla UI option

---

## Phase Roadmap

### Phase 0 — Local Alpha (Weeks 1–4)
Goal: One device, one model, one API

- [ ] Tauri v2 app scaffold (Rust + React + TypeScript)
- [ ] llama.cpp integration via Rust bindings
- [ ] Resource profiler: CPU/GPU/RAM detection
- [ ] LiteLLM proxy wrapping local llama.cpp
- [ ] Basic tray UI: model select, start/stop

### Phase 1 — Two-Node Swarm (Weeks 5–10)
Goal: Two devices share one job

- [ ] WireGuard mesh (headscale embedded) — manual join token
- [ ] etcd Blackboard: node registration, heartbeat
- [ ] Basic scheduler: route job to highest-scoring node
- [ ] Contribution Ledger v1: flat token counting
- [ ] Prometheus metrics export from nodes

### Phase 2 — Heterogeneous Pool (Weeks 11–18)
Goal: 5–20 nodes, mixed hardware, model sharding

- [ ] Capability scoring algorithm (VRAM/RAM/CPU weighted)
- [ ] Exo-style ring topology for model sharding
- [ ] Node churn handling: job re-queue on TTL expiry
- [ ] mDNS + gossip for LAN auto-discovery
- [ ] API Gateway rate limiting by credit balance

### Phase 3 — Observability & Governance (Weeks 19–24)
Goal: Operator-grade visibility + control

- [ ] Grafana dashboards: 5 core panels (see ui_ux.md)
- [ ] Admin portal: user/key/node management
- [ ] Ledger audit trail: tamper-evident log (HMAC chain)
- [ ] Special Days analytics view
- [ ] Alertmanager integration

### Phase 4 — BD Production Launch (Weeks 25–32)
Goal: Public beta, BD market entry

- [ ] SSLCommerz/bKash credit purchase integration
- [ ] Bangla UI localization
- [ ] GitHub OSS release under Apache 2.0
- [ ] BASIS/ICT Division showcase
- [ ] Public API endpoint: `api.swarm-os.dev`
- [ ] GitHub Actions CI/CD pipeline
- [ ] Security audit (rate limits, node isolation, API auth)

---

## Bangladeshi-Specific Considerations

| Concern | Solution |
|---------|---------|
| CGNAT / ISP NAT | headscale DERP relay servers; fallback to relay if P2P fails |
| Low bandwidth regions | Model streaming in chunks; gzip compression on API responses |
| BDT payment | SSLCommerz integration for credit top-ups |
| Bangla speakers | i18n with `react-i18next`; BD English as default, Bangla optional |
| Power cuts / node dropout | Short TTL heartbeats (5s); aggressive job re-queueing |
| Trust/reputation | Node reputation score based on uptime history in Blackboard |
| Local latency | Prefer same-ASN or same-city nodes in scheduler scoring |

---

## Success Metrics (6-month targets)

| Metric | Target |
|--------|--------|
| Nodes in swarm | 50+ |
| Registered users | 500+ |
| Tokens generated / day | 10M+ |
| P95 response latency | < 8s (first token) |
| Node uptime average | > 85% |
| GitHub stars | 200+ |
| BD press mentions | 3+ (Prothom Alo Tech, Daily Star Tech) |
