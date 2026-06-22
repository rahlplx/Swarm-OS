# Swarm-OS: Full-Team Critique Report

> **Team:** Architecture & Infrastructure · Market Research · Product & UX · Business & GTM · Security & Compliance  
> **Date:** 2026-06-22  
> **Verdict:** Strong concept with 3 existence-threatening technical flaws, 2 critical security vulnerabilities, 1 broken business model, and a missing MVP feature (model download UI). All fixable. Priority order below.

---

## Severity Index

| # | Issue | Domain | Severity |
|---|-------|--------|----------|
| 1 | Cross-node llama.cpp sharding doesn't exist in any OSS library | Architecture | BLOCKER |
| 2 | API key stored in etcd job payload — live credential exfiltration | Security | BLOCKER |
| 3 | Ledger will silently corrupt under etcd auto-compaction | Architecture | BLOCKER |
| 4 | No model download UI — contributors cannot onboard | Product | BLOCKER |
| 5 | Zero platform revenue on P2P credit exchange | Business | CRITICAL |
| 6 | BD DERP relay missing — 70B sharding across mobile nodes is physically impossible | Architecture | CRITICAL |
| 7 | SHA-256 for API key hashing → use Argon2id | Security | CRITICAL |
| 8 | Prompt stored in etcd job payload — readable by all node operators | Security | CRITICAL |
| 9 | Onboarding is 8–12 min, not 3 min. AppImage fails on Ubuntu 22.04+ | Product | HIGH |
| 10 | Activation tensor poisoning in shard ring — undetected mid-inference tampering | Security | HIGH |
| 11 | Headscale compromise = full MITM on all inter-node traffic | Security | HIGH |
| 12 | Ledger replay attack — no nonce/monotonic counter | Security | HIGH |
| 13 | KV cache failover is a full restart, not a resume | Architecture | HIGH |
| 14 | No secrets management (likely .env files everywhere) | Security | HIGH |
| 15 | BTRC could classify headscale/WireGuard as unlicensed VPN | Compliance | HIGH |
| 16 | Contributor vs. consumer dashboard merged — two separate products | Product | HIGH |
| 17 | Credit economy: no burn rate display, ৳200 min too high | Product | MEDIUM |
| 18 | bKash payment OTP flow not specified — will fail silently | Product | MEDIUM |
| 19 | Upload bandwidth (5–15 Mbps BD) is the real activation-transfer bottleneck | Architecture | MEDIUM |
| 20 | GTM messaging is wrong — "decentralized inference" vs "earn ৳500/month" | Business | MEDIUM |
| 21 | No conversation history / session management in Playground | Product | MEDIUM |
| 22 | Content liability gap under CSA 2023 — no output moderation layer | Compliance | MEDIUM |
| 23 | Redis rate limiter failure mode unspecified (fail-open = no limits) | Security | MEDIUM |
| 24 | GPU UUID Sybil via QEMU/MIG vGPU passthrough | Security | MEDIUM |
| 25 | Pipeline parallelism bubble: TTFT = sum of all node latencies, not parallelized | Architecture | LOW |
| 26 | LiteLLM Python GIL at 100+ concurrent streams | Architecture | LOW |
| 27 | WebKitGTK 2.32 (Ubuntu 20.04/Debian 11) has CSS grid bugs | Architecture | LOW |
| 28 | "Credits never expire" is a consumer liability commitment — remove in v1 | Business | LOW |

---

## Domain 1: Architecture & Infrastructure

### BLOCKER — Cross-Node llama.cpp Sharding Does Not Exist

This is the product's central value proposition and it has no implementation. `llama.cpp --split-mode` works across GPUs on a single machine via PCIe — not across network nodes. Streaming hidden-state activation tensors (for Llama-3 70B at fp16: **~8MB per forward pass** at seq_len=512, hidden_dim=8192) over WireGuard requires custom serialization, transfer, and deserialization logic.

**Fix:** Replace "Exo ring topology" as the primary reference with **`b4rtaz/distributed-llama`** (MIT license, implements exactly this in C++ with TCP socket transport for GGUF models). This is the correct OSS steal, not exo-labs/exo (GPL-3.0, MLX-only, Apple-only). Add to Phase 0 scope: prototype activation tensor transfer between 2 nodes using distributed-llama as the inference core.

### BLOCKER — Ledger Corrupts Under etcd Auto-Compaction

etcd's `--auto-compaction-retention` permanently deletes old revisions. Your `SHA-256(prev_entry)` hash chain links are etcd revision-based. When a revision is compacted, the chain is unverifiable — not alertable, just silently broken.

**Fix:** Move the ledger out of etcd entirely. Use **SQLite in WAL mode** on the orchestrator with an append-only table and an immutable trigger. Store only the current chain head hash in etcd as a pointer. SQLite WAL is a battle-tested, boring solution — that's correct here.

### CRITICAL — BD DERP Relay: 70B Sharding Over Mobile Is Physically Impossible

Grameenphone and Banglalink use CGNAT. Two mobile nodes cannot establish direct WireGuard P2P — all traffic routes through Tailscale's DERP relay in Singapore. Round-trip Singapore latency from Dhaka: 70–110ms. A 40-layer 70B model split across 3 nodes = 10 pipeline stage boundaries = **10 × 70ms = 700ms added per generated token in relay mode**. This is completely unusable.

**Fix (৳2,000–৳3,000/month):** Deploy a Swarm-OS DERP relay node in Dhaka — Zenlayer BD or any Dhaka co-location VPS with a public IP. This is a hard infrastructure requirement, not a Phase 2 nice-to-have.

### HIGH — KV Cache Failover Is a Full Restart, Not a Resume

The spec says "job resumes from last checkpoint." If Node B dies, its KV cache for layers 20–31 is on its local disk — which is now offline. The etcd pointer references a dead node's filesystem. This is a full restart from token 0, not a resume. 10s TTL expiry = a 10-second user-facing stall before full restart. This must be stated accurately in specs and user-facing SLA.

**Mitigation:** For Phase 1, frame it as "fault-tolerant restart" not "checkpoint resume." True checkpoint resume requires shared ephemeral storage (e.g., Redis, MinIO). Descope for now; set honest expectations.

### Better OSS Stack Additions

| Replace/Add | With | Reason |
|-------------|------|--------|
| Exo (GPL, Apple-only) | **b4rtaz/distributed-llama** (MIT, cross-platform, TCP-native) | Actual cross-node llama.cpp sharding |
| etcd for metrics/heartbeats | **NATS JetStream** (JetStream KV for watch semantics) | 10M msgs/sec vs etcd's 1–2k/sec; reserve etcd for job assignments only |
| Custom scheduler | **Ray Serve** (optional, Phase 3) | Battle-tested heterogeneous cluster scheduling |
| Petals pattern reference | Add **Petals (LAION/BigScience)** as architectural reference | BitTorrent-style weight sharding is more resilient to node failure than ring topology |
| vllm for high-VRAM nodes | Add **vllm-project/vllm** alongside llama.cpp | 2–4× higher throughput on batched requests for datacenter-class GPUs |

### Missing: Circuit Breaker for Unstable Nodes

No exponential backoff or quarantine for flapping nodes (repeatedly joining and dying). Pattern: after N consecutive TTL failures within window W, quarantine node for M minutes. Steal: Hystrix/Resilience4j circuit breaker pattern.

### Missing: Admission Control / Multi-Class Queue

No backpressure when 70B jobs queue behind 7B jobs that could run immediately. Need separate queues per model-size tier with priority-based admission. A 70B job should not block a 7B job that has 5 eligible nodes waiting.

### BD Network Reality

| Scenario | RTT | Upload BW | 70B Sharding Viable? |
|----------|-----|-----------|----------------------|
| Dhaka fiber ↔ Dhaka fiber | 2–8ms | 20–100 Mbps | Yes |
| Dhaka fiber ↔ Chittagong fiber | 15–30ms | 20–50 Mbps | Yes (tight) |
| Grameenphone 4G ↔ Grameenphone 4G | 40–80ms + DERP | 2–10 Mbps up | No (without Dhaka DERP relay) |
| Rural (outside metro) | 100–300ms | 1–5 Mbps | No |

**Critical fix:** Compress activation tensors with zstd before transfer. bf16 activation tensors achieve ~2× compression. Reduces 8MB/forward-pass to ~4MB. Mandatory on all activation transfers.

---

## Domain 2: Security & Compliance

### BLOCKER — API Key in etcd Job Payload

`/swarm/jobs/{id}/request → {model, messages, api_key}` — the consumer's live API key is written to the shared Blackboard and is readable by every node agent with etcd access. Zero exploitation required.

**Fix (P0):** Strip the API key from the job payload entirely. Replace with a **one-time job authorization token** scoped to that specific job ID with a 60-second TTL. The node proves assignment via job token, not the user's permanent API key.

### CRITICAL — Prompt Data in etcd

Raw prompt messages are stored in etcd, readable by all node operators. The "prompts never logged to disk" claim is a policy statement on untrusted hardware — it is unenforceable without TEE (Trusted Execution Environment).

**Fix (P1):** Remove prompts from etcd. Job objects in the Blackboard should contain only routing metadata (model ID, token budget, shard assignment). Prompts are transmitted directly over WireGuard to assigned nodes only, never written to shared state.

### CRITICAL — SHA-256 for API Key Storage → Argon2id

SHA-256 is a fast hash. For user-generated or low-entropy keys, it is rainbow-table vulnerable.

**Fix:** `argon2id(key, salt, time=1, mem=64MB, parallelism=4)` with a per-key random 32-byte salt stored alongside the hash in etcd.

### HIGH — Ledger Replay Attack

The Ed25519 signature chain proves tamper-evidence but does not prevent replay. A compromised node can replay a previous valid signed entry with a forged timestamp. The orchestrator verifies the signature (valid) and hash chain (consistent) — and accepts the fraudulent entry.

**Fix:** Add a **monotonic sequence number** per node, server-side verified. Reject any entry where `seq_num ≤ last_accepted_seq_num` for that node.

### HIGH — Activation Tensor Poisoning

A malicious node in the sharding ring can modify the activation tensor before forwarding. The current output validation (golden-sample spot checks) validates final output, not intermediate activations. An attacker controlling Node B in a 3-node chain can steer Node C's final output arbitrarily.

**Fix (Phase 2):** Implement **Merkle-tree commitment** on activation tensors at each pipeline stage boundary. Node A commits to its output tensor hash before sending; the orchestrator verifies Node B's input hash matches Node A's commitment. Expensive but necessary for any trust model.

### HIGH — BTRC WireGuard/headscale Risk

WireGuard at protocol level is indistinguishable from a commercial VPN. BTRC has previously restricted VPN services. A regulatory interpretation could block the mesh control plane, taking the swarm offline.

**Contingency (P2):** Design an **HTTPS-tunneled transport** (port 443 WebSocket) as a fallback mesh layer. Indistinguishable from normal HTTPS traffic. Implement before public launch, activate if BTRC restricts WireGuard.

### MEDIUM — CSA 2023 Content Liability

Bangladesh's Cyber Security Act 2023 creates broad content liability. There is no output filtering, content classification, or CSAM detection. The operator — not node contributors — is the legally reachable entity.

**Fix:** Integrate **LlamaGuard** (Meta, Apache 2.0) at the API gateway output stream. Lightweight classifier, runs on CPU, adds ~50ms. Filters CSA 2023 high-risk content categories before streaming to client.

### Security Fix Priority Matrix

| Priority | Fix | Effort |
|----------|-----|--------|
| P0 | Remove API key from etcd job payload | 2 hours |
| P0 | Argon2id for key hashing | 1 hour |
| P0 | Secrets management (Vault or cloud-native) | 1 day |
| P1 | Strip prompt from etcd; direct P2P delivery | 3 days |
| P1 | Ledger replay protection (monotonic seq) | 1 day |
| P1 | Redis fail-closed on unavailability | 2 hours |
| P2 | Activation tensor integrity (Merkle commitment) | 1 week |
| P2 | HTTPS-tunnel mesh transport (BTRC contingency) | 1 week |
| P2 | LlamaGuard output filter | 2 days |
| P2 | Auto-update enforcement (min version gate) | 3 days |

---

## Domain 3: Product & UX

### BLOCKER — No Model Download UI

The biggest missing screen. Step 2 of onboarding shows "Models you can run: Llama-3.1-70B" but there is zero UI for downloading 40GB of model weights. Who pays for the egress? When does it happen? Where is it stored?

**Fix:** Gate node registration behind model download. New flow: Detect hardware → Show eligible models with file size + RAM requirement → Download with progress bar + checksum verification → Then register node. Steal: **LM Studio's model browser UI** (searchable, shows quantization options, has "will this run on your device?" indicator per model).

### HIGH — Onboarding Is 8–12 Minutes, Not 3

Actual micro-decision count: OAuth choice → email confirm → OS/package choice → AppImage FUSE2 error on Ubuntu 22.04+ → hardware detection edge cases (Optimus dual-GPU) → 3 resource sliders → context switch back to browser for join token → token paste. Drop-off risk at each step.

**Critical fix 1:** Replace AppImage as Linux default with `.deb`. AppImage on Ubuntu 22.04+ requires `libfuse2` which is not installed by default.

**Critical fix 2:** Steal Ollama's install pattern — `curl -fsSL https://swarm-os.dev/install.sh | sh`. One command handles OS detection, dependency installation, agent setup, and first launch. Step 1 of onboarding becomes a single copy-paste.

**Critical fix 3:** Join token delivery via deep link (`swarm-os://join?token=xxx`) that opens the installed desktop app directly. Eliminates the browser-to-desktop context switch.

### HIGH — Two Products, One Dashboard

Contributors (GPU owners, passive earners) and consumers (API users) have zero psychographic overlap. The current dashboard merges their stats, creating noise for both. 

**Fix:** Separate post-login destinations based on account type set during signup. Contributor dashboard: earnings, node status, resource slider. Consumer dashboard: credits, API keys, job history, playground. Shared nav only for Ledger and Settings.

### MEDIUM — Credit Economy UX Gaps

- No burn rate display: "Balance lasts ~3 days at current usage" must appear next to balance
- ৳200 minimum top-up blocks students and testers — add ৳50 trial tier (100 credits, one-time per account)
- Mid-inference credit exhaustion behavior unspecified — likely a jarring mid-sentence truncation
- "Credits never expire" is a consumer liability commitment — remove this claim in v1
- Show per-model credit cost in the model selector dropdown itself (steal from OpenRouter's UX)

### MEDIUM — bKash Payment Flow Is Underspecified

bKash requires: enter number → receive OTP → enter OTP → PIN confirmation. The spec shows one button. OTP timeout/wrong PIN are common failure paths for BD users. All 4 steps must be mocked with error states before integration work begins.

### Low-Hardware UX (4GB RAM Laptops)

Many BD students have 4GB RAM + no discrete GPU. They cannot meaningfully contribute. The UI must detect this (score < 15) and redirect gracefully: "Your device is better suited for using the API. Join as a consumer →" — non-humiliating, clear action.

---

## Domain 4: Market Research & Target Users

### BD Market Reality

| Metric | Data |
|--------|------|
| Active BD software developers | ~100,000–150,000 (BASIS 2024) |
| BD freelancers | ~650,000 active (Basis/LightCastle) |
| ChatGPT Plus as % of mid dev salary | 4–9% — significant but not impossible |
| **Real payment barrier** | USD payment friction (Bangladesh Bank FX rules on int'l subscriptions) — this is the actual pain point, not just price |
| Gaming PCs in BD | ~200,000–400,000, mostly GTX 1060–RTX 3060, capable of 7B inference |
| BD home upload speed | 5–15 Mbps — the real bottleneck for contributor nodes |
| Power reliability outside Dhaka | 1–4 outages/day — TTL re-queue is essential |

### Target User Ranking by Adoption Probability

1. **BD Freelancers (Upwork/Fiverr)** — Highest. Pain: USD payment friction for ChatGPT + copywriting/code gen needs. bKash/Nagad top-up removes both barriers. 
2. **BD Indie Developers** — High. Want OpenAI-compatible API without monthly commitment. Zero SDK changes needed.
3. **Global GPU Owners (Contributors)** — High if credit economy has consumers. The chicken-and-egg problem is real.
4. **Global Indie Devs (Consumers)** — Moderate. Need credit-for-compute positioning to differentiate from Groq/Together.ai free tiers.
5. **BD University Students** — Low-to-moderate. Weak hardware = pure consumers with near-zero payment capacity. User acquisition story, not revenue.
6. **BD Startups** — Low near-term. Require SLA guarantees the bootstrapped swarm can't deliver in Year 1.

### Why Petals Failed (and What We Must Do Differently)

Petals (LAION) stagnated because: no economic incentive for contributors, research-grade reliability, no observability. Swarm-OS's ledger + Grafana + TTL re-queue directly addresses all three. But Swarm-OS must also solve the **consumption demand** problem — contributors need guaranteed consumers to make credits valuable. **Launch with a specific free use case** (e.g., free Bangla translation API) to guarantee day-1 consumption.

### Unexpected Opportunities

1. **Bangla LLM gap:** No high-quality Bangla LLM exists. 170M Bangla speakers globally. A Swarm-OS-hosted Bangla-specialized 7B model competes in a market Groq/Together.ai completely ignore.
2. **University GPU labs:** BUET, NSU, BRAC, DIU have GPU workstations idle outside class hours. Institutional nodes = stable power + upload + trust. Target for early network bootstrapping.
3. **BD Diaspora compute:** BD diaspora in US/UK/Canada have high-end hardware. "Donate compute to Bangladesh" (Folding@Home-style altruism narrative) is a real acquisition lever.
4. **Distributed fine-tuning:** BD gaming PCs idle 10 PM–8 AM. LoRA training on idle GPUs is less latency-sensitive than inference and commands higher willingness-to-pay globally.
5. **Night-time compute arbitrage for training:** Position this as a fine-tuning platform (higher value) alongside inference — not mentioned in current docs.

---

## Domain 5: Business Model & GTM

### CRITICAL — The Platform Makes Zero Revenue on P2P Exchange

Pure contributor→consumer credit swap: zero platform revenue. Revenue only comes from credit top-ups (BDT for credits). There is no take rate, no transaction fee, no spread on P2P exchange. The entire business model is: sell credits, contributors earn them, consumers spend them, Swarm-OS keeps the difference between BDT received and infra cost.

**Unit economics:** $65/month minimum infra (Hetzner). At ৳200/500 credits with ~60% margin after payment fees: ৳190 net per purchase. Break-even: **38 top-up purchases/month**. Achievable, but zero repeat-purchase lock-in is engineered into the product.

### Revenue Model Fix: Operator SaaS Tier (Highest Viability)

Charge ৳5,000–15,000/month for organizations running a **private swarm** on Swarm-OS orchestration software. They bring GPUs; Swarm-OS provides the coordination layer. Zero compute cost to the platform. Target: BD universities, NGOs (BRAC, CARE), BASIS-member software houses, government ministries.

| Revenue Stream | Viability | Timeline |
|----------------|-----------|----------|
| Operator SaaS (private swarm license) | High | Month 2 |
| B2B API contracts (BD startups, flat rate) | High | Month 3 |
| University/research white-label | Medium-High | Month 4 |
| Government/NGO contracts (a2i, UNDP) | Medium | Month 8+ |
| Consumer credit top-ups | Medium | Launch |
| Mobile operator bundling (GP/Robi) | Low | Year 2 |
| P2P marketplace with take rate | Low | Year 2 |

### GTM: Fix the Messaging First

**Current:** "Decentralized heterogeneous compute swarm for AI inference"  
**Should be:** "আপনার GPU রাতে ঘুমায় — Swarm-OS দিয়ে সেটা থেকে আয় করুন" (Your GPU sleeps at night — earn from it with Swarm-OS)

**PLG motion for first 6 months** (not B2B, not community — pick one):
- First value must be: install app → see credits accumulating within 5 minutes. Everything else comes later.
- Free tier must demonstrate value: increase to 30 RPM + all 7B models (current 10 RPM is too weak)

### Marketing Playbook (Concrete)

**Pre-launch (Week 1–2):**
- 90-second screen recording: gaming PC earning credits overnight. No pitch. Just the number going up.
- HN "Show HN" — Tuesday 9am ET. Technical credibility post.
- r/LocalLLaMA: "Built a P2P inference network from Bangladesh — architecture breakdown"

**BD Launch (Week 3–4):**
- Facebook groups: "Bangladesh Android Developers" (200k+), "BD Freelancers" (500k+)
- Prothom Alo Tech + Daily Star Tech direct pitch: "Made in Bangladesh, competes with OpenAI"
- 3 BD tech YouTubers (Tech Alaap, SkillsN Jobs BD, Gadget Insider BD) — ৳5,000 sponsor or free Pro credits

**University Program (Month 2):**
- Free Pro tier for `.edu.bd` emails who contribute a node — costs ৳0, generates nodes + word-of-mouth
- Sponsor BUET CSE Fest + NSU ACM — prize: ৳50,000 in credits (zero cash cost; credits are liability only if consumed, and contributors provide the supply)

**Affiliate (Month 3):**
- BD Upwork/Fiverr freelancers: refer a paying user, earn 20% of their first top-up in credits

### The Real Moat (Not Price)

Cloud API prices are collapsing (GPT-4o mini: $0.00015/1K tokens, Gemini Flash: free tier). Price competition is unwinnable. The actual defensible bundle:

**BDT payment rails + Bangla UI + data sovereignty + offline-capable inference + local latency**

No US cloud provider will build this bundle for Bangladesh. This is the wedge. Every product and GTM decision should reinforce it.

---

## Consolidated Action Plan

### Fix Before Writing Any Code (Pre-Phase 0)

1. Replace exo/llama.cpp sharding with `distributed-llama` — prototype activation transfer
2. Remove API key from etcd job payload → one-time job token
3. Move ledger to SQLite WAL — remove from etcd
4. Argon2id for API key hashing
5. Strip prompts from etcd — direct P2P delivery to assigned nodes only
6. Add model download flow as the first step after hardware detection

### Fix in Phase 1

7. Deploy Dhaka DERP relay node (৳2,000–3,000/month VPS)
8. Split contributor/consumer onboarding and dashboards
9. Replace AppImage default with .deb; add `curl | sh` installer
10. Add Secrets management (Vault or cloud-native)
11. Ledger replay protection (monotonic sequence number)
12. Add Redis fail-closed behavior
13. Build bKash 4-step payment flow with OTP handling
14. Add burn rate display + ৳50 trial tier
15. Activate Operator SaaS revenue stream — target 3 BD organizations

### Fix in Phase 2

16. Activation tensor zstd compression (mandatory for mobile nodes)
17. Activation tensor Merkle commitment (integrity against poisoning)
18. HTTPS-tunnel mesh transport (BTRC contingency)
19. LlamaGuard output filter (CSA 2023 compliance)
20. Auto-update enforcement (min llama.cpp version gate)
21. Circuit breaker for flapping nodes
22. Multi-class job queue (separate queues per model-size tier)
23. Bangla LLM fine-tune initiative — differentiate from global providers
24. University GPU lab partnership program

---

## Revised Success Metrics (Realistic, Not Vanity)

| Metric | Original | Revised (Realistic) |
|--------|----------|---------------------|
| GitHub stars (6mo) | 200+ | 300–500 (achievable with HN/r/LocalLLaMA launch) |
| Active nodes | 50+ | 20–50 (quality > quantity in Phase 1) |
| Registered users | 500+ | 200–500 |
| Paying users / top-ups | Not specified | **38/month minimum to break even** |
| Operator SaaS contracts | Not specified | **3 contracts by Month 6** |
| Tokens/day | 10M+ | 500K–2M (realistic for &lt;50 nodes) |
| P95 first-token latency | &lt;8s | &lt;8s on fiber; &lt;15s on mobile (honest) |
| Revenue (Month 6) | Not specified | ৳50,000–৳150,000/month (BDT) |
