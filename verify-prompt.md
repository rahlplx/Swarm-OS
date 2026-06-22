<swarm_os_verify>

<meta>
  <task>Pre-build community verification of Swarm-OS — a decentralised P2P AI inference network</task>
  <goal>Find every assumption that will cause a production bug, integration failure, or security hole BEFORE a single line of implementation code is written. Prioritise Phase 0 blockers.</goal>
  <persona>You are a panel of three experts acting in concert: (1) a senior distributed-systems engineer with deep OSS knowledge, (2) an application-security researcher, (3) a product economist who models incentive systems. Each domain below is owned by the most relevant expert. Cross-domain findings are mandatory when one domain's answer changes another domain's answer.</persona>
  <instruction>Work through every verification domain sequentially. For each item, search community sources (GitHub issues, RFCs, CVE databases, benchmark papers, forum posts) before answering. Never answer from training-data intuition alone — cite the concrete artifact (issue number, commit, RFC section, paper title) that supports your verdict. If no community artifact exists, mark confidence ≤ 40 and flag as UNKNOWN.</instruction>
  <output>Return a single JSON object matching <output_schema>. No prose outside the JSON block.</output>
</meta>

<project_context>
  <identity>
    <name>Swarm-OS</name>
    <tagline>Your idle GPU earns. Your AI runs free.</tagline>
    <origin>Bangladesh</origin>
    <license>Apache 2.0</license>
    <phase_target>Phase 0 — local alpha (single device, single model, OpenAI-compatible API) in 4 weeks</phase_target>
  </identity>

  <architecture_decisions>
    <decision id="A1" domain="inference">
      Inference engine: llama.cpp (ggerganov/llama.cpp, MIT) with GGUF model format.
      Rust FFI binding: llama-cpp-rs or mdrokz/rust-llama.cpp.
      Cross-node sharding reference: b4rtaz/distributed-llama (MIT) — ring pipeline parallelism.
      llama.cpp --split-mode is single-machine only; cross-node requires distributed-llama protocol.
    </decision>
    <decision id="A2" domain="networking">
      P2P mesh: WireGuard via headscale (juanfont/headscale, BSD-3) self-hosted Tailscale control plane.
      DERP relay fallback for CGNAT (common in Bangladesh ISPs).
      A Dhaka-region DERP relay node is required — Singapore relay adds ~700ms/token latency for BD mobile nodes.
      Prometheus scrapes /metrics (port 9100) on WireGuard mesh IPs — Prometheus must join the headscale mesh.
    </decision>
    <decision id="A3" domain="coordination">
      Blackboard: etcd v3. Keys: /swarm/nodes/{id}/alive (TTL=10s, write every 5s), /swarm/jobs/{id}/*, /swarm/config/*.
      Ledger storage: SQLite WAL on orchestrator (NOT etcd — etcd auto-compaction breaks SHA-256 hash chains).
      etcd stores only /swarm/ledger/{node_id}/head_hash pointer.
    </decision>
    <decision id="A4" domain="api_gateway">
      LiteLLM proxy (BerriAI/litellm, MIT) as OpenAI-compatible facade.
      Custom swarm provider plugin via litellm custom_llm_provider pattern.
      Rate limiting: Redis sliding window counter.
      Post-completion hook: LiteLLM success_callback debits consumer credits, writes ledger delta.
      Job payload in etcd: {model, job_token, max_tokens, stream} — NO api_key, NO prompt messages.
    </decision>
    <decision id="A5" domain="desktop_agent">
      Tauri v2 (stable since 2024) — Rust backend + React frontend.
      Target: ~15 MB binary, less than 50 MB RAM idle.
      Resource profiling: sysinfo crate (CPU/RAM), nvml-wrapper (NVIDIA), metal crate (Apple).
      Ed25519 signing: ed25519-dalek crate.
      Async runtime: tokio.
    </decision>
    <decision id="A6" domain="observability">
      Prometheus direct scrape (NOT pushgateway — wrong tool for long-running daemons).
      Prometheus must be a headscale mesh member to reach CGNAT nodes on mesh IPs.
      Alertmanager with inhibit_rules: node_dropout_spike suppresses queue_overflow + high_latency.
      Alert threshold: P95 latency > 8s (SLA target: < 8s first token).
    </decision>
  </architecture_decisions>

  <security_decisions>
    <decision id="S1">API key hashing: Argon2id, time=3, mem=64MB, parallelism=4, per-key random salt. (OWASP 2025 minimum: time=3)</decision>
    <decision id="S2">Ledger: Ed25519 signature per entry + SHA-256 hash chain. ADMIN_ADJUST entries in separate admin_adjustments SQLite table — NOT in the hash chain. Chain traversal is always contiguous.</decision>
    <decision id="S3">Key prefixes: swrm_sk_ (consumer), swrm_ops_ (operator), swrm_adm_ (super admin), swrm_node_ (inter-node contributor).</decision>
    <decision id="S4">Node auth: Ed25519 keypair generated on first run; public key registered in Blackboard at join. All ledger entries signed by node private key.</decision>
    <decision id="S5">Inference sandboxing: seccomp on Linux, App Sandbox on macOS. No cross-node KV cache access.</decision>
    <decision id="S6">WireGuard (ChaCha20-Poly1305) for all inter-node traffic. etcd uses client certificate auth, not exposed to internet.</decision>
    <decision id="S7">Fake-contribution detection: challenge-response with known greedy-decode (temp=0) outputs only. Peer output comparison NOT used (non-deterministic at temp > 0 causes false positives).</decision>
    <decision id="S8">Sock-puppet prevention: NVIDIA GPU UUID deduplicated on registration. AMD/CPU: motherboard serial + CPU ID fingerprint via sysinfo.</decision>
    <decision id="S9">Prompt privacy: prompts delivered P2P directly to assigned nodes; never stored in etcd.</decision>
  </security_decisions>

  <economic_model>
    <earn>credits_earned = tokens_generated × 0.008 [× 1.1 if node_score > 80]</earn>
    <spend>credits_spent = (input_tokens × 0.006) + (output_tokens × 0.01)</spend>
    <rule_of_thumb>1 credit ≈ 100 output tokens for consumers</rule_of_thumb>
    <platform_spread>~20% margin (earn 0.008 per output token, spend 0.01 per output token)</platform_spread>
    <top_up_tiers>
      [
        {"min_bdt": 50,  "max_bdt": 449,  "credits_per_bdt": 2.5,  "label": "Standard"},
        {"min_bdt": 450, "max_bdt": 899,  "credits_per_bdt": 2.67, "label": "Plus (+7%)"},
        {"min_bdt": 900, "max_bdt": null, "credits_per_bdt": 2.89, "label": "Pro (+15%)"}
      ]
    </top_up_tiers>
    <currency>BDT via SSLCommerz (bKash, Nagad, Visa/MC). REST API — NOT Python SDK.</currency>
    <node_scoring>score = (vram_gb × 4) + (ram_gb × 0.5) + (cpu_cores × 0.25) + backend_bonus + locality_bonus
      backend_bonus: cuda=10, metal=8, vulkan=5, cpu=0</node_scoring>
    <abuse_cap>max_credits_per_hour = node_score × 2</abuse_cap>
  </economic_model>

  <bd_specific>
    <constraint id="BD1">CGNAT prevalent in BD ISPs (Grameenphone, Robi, Banglalink). WireGuard + headscale DERP required.</constraint>
    <constraint id="BD2">Singapore DERP relay adds ~700ms/token for BD mobile nodes. Dhaka-region DERP is a hard requirement.</constraint>
    <constraint id="BD3">Payment: SSLCommerz REST API. bKash OTP flow: phone → OTP via SMS → PIN → SSLCommerz callback → credit top-up.</constraint>
    <constraint id="BD4">Font: Hind Siliguri (Google Fonts) for Bangla UI. Western numerals in Bangla mode.</constraint>
    <constraint id="BD5">Power cuts / node dropout: 10s TTL heartbeat, aggressive job re-queueing, up to 3 requeue attempts.</constraint>
  </bd_specific>

  <oss_dependencies>
    [
      {"id": "ggerganov/llama.cpp",      "license": "MIT",         "role": "inference engine"},
      {"id": "b4rtaz/distributed-llama", "license": "MIT",         "role": "cross-node sharding reference"},
      {"id": "BerriAI/litellm",          "license": "MIT",         "role": "API gateway proxy"},
      {"id": "juanfont/headscale",        "license": "BSD-3",       "role": "WireGuard mesh control plane"},
      {"id": "etcd-io/etcd",             "license": "Apache-2.0",  "role": "Blackboard KV store"},
      {"id": "tauri-apps/tauri",         "license": "MIT/Apache",  "role": "desktop agent shell"},
      {"id": "prometheus/prometheus",    "license": "Apache-2.0",  "role": "metrics"},
      {"id": "prometheus/alertmanager",  "license": "Apache-2.0",  "role": "alert routing"},
      {"id": "grafana/grafana",          "license": "AGPL-3.0",    "role": "dashboards"},
      {"id": "exo-labs/exo",            "license": "GPL-3.0",     "role": "reference only — NO code porting"},
      {"id": "gpustack/gpustack",        "license": "Apache-2.0",  "role": "scheduler pattern reference"},
      {"id": "shadcn/ui",               "license": "MIT",         "role": "React components"},
      {"id": "WireGuard/wireguard-go",   "license": "MIT",         "role": "P2P data plane"},
      {"id": "tokio-rs/tokio",           "license": "MIT",         "role": "Rust async runtime"},
      {"id": "ed25519-dalek",           "license": "MIT/Apache",  "role": "Ed25519 signing in Rust"},
      {"id": "sslcommerz-lts (npm)",     "license": "MIT",         "role": "BD payment JS wrapper"}
    ]
  </oss_dependencies>

  <phase_0_scope>
    Single device. llama.cpp local inference. LiteLLM proxy wrapping local llama.cpp. Tauri v2 tray agent. No mesh networking. No etcd. No ledger. Goal: OpenAI-compatible API working locally in 4 weeks.
  </phase_0_scope>
</project_context>

<verification_domains>

  <domain id="V1" title="OSS Library Feasibility">
    For each OSS dependency, verify:
    (a) The specific capability we claim exists (e.g., distributed-llama ring topology, headscale DERP, LiteLLM custom_llm_provider)
    (b) Current maintenance status (last commit, open critical issues, deprecation notices)
    (c) Known incompatibilities with our stack (Rust FFI, Tauri v2, llama.cpp version pinning)
    (d) Any breaking changes since the referenced API/pattern was documented
    Flag any dependency where the claimed feature does not exist or is unstable.
  </domain>

  <domain id="V2" title="License Compatibility">
    Verify no license conflict exists in our dependency graph under Apache 2.0 distribution.
    Specifically: GPL-3.0 (exo-labs/exo) is reference-only — verify our clean-room strategy is legally sound.
    Verify AGPL-3.0 (Grafana) does not infect our codebase when self-hosted by operators.
    Flag any dependency whose license requires source disclosure or prohibits commercial use.
  </domain>

  <domain id="V3" title="Security Threat Model">
    For each security decision (S1–S9), verify:
    (a) Cryptographic parameters are current best-practice (Argon2id params, Ed25519, ChaCha20-Poly1305)
    (b) Known attack vectors not covered by our mitigations
    (c) The challenge-response validation (S7) is sufficient at Phase 2 scale (50 nodes)
    (d) GPU UUID deduplication (S8) can be spoofed and what the actual attack surface is
    (e) Whether etcd client-cert auth is sufficient given the Blackboard contains job routing data
    Additional: model the "colluding nodes" attack — two nodes owned by same actor submitting jobs to each other to earn credits faster than spend rate.
  </domain>

  <domain id="V4" title="Economic Model Integrity">
    Verify the earn/spend formula is self-consistent:
    (a) Confirm earn (0.008/token) < spend (0.01/output token) — no infinite credit creation possible
    (b) Model a realistic workload (100 input + 500 output tokens): compute platform margin in credits
    (c) Model the high-score bonus edge case: node_score > 80, earn = 0.0088 — still below 0.01 spend?
    (d) Verify abuse_cap (node_score × 2 credits/hour) is binding at realistic token rates
    (e) Verify the BDT tier math: ৳450 × 2.67 = 1,201.5 credits — round-down or round-up policy needed
    (f) Flag any scenario where credit supply grows faster than demand (hyperinflation risk)
  </domain>

  <domain id="V5" title="Network Architecture — CGNAT and BD Constraints">
    Verify:
    (a) headscale DERP relay implementation — does it work bidirectionally under CGNAT without port forwarding?
    (b) WireGuard kernel module availability on common BD ISP router/modem firmware (Huawei HG8245, TP-Link Archer)
    (c) Prometheus scrape-via-WireGuard-mesh — what happens when a node's mesh IP changes (headscale re-key)?
    (d) The 700ms/token latency claim for Singapore DERP — is this empirically documented or estimated?
    (e) Whether SSLCommerz bKash integration works with both prepaid and postpaid bKash accounts
    (f) Known BD regulatory constraints on running relay servers or encrypted mesh networks
  </domain>

  <domain id="V6" title="Data Layer Correctness">
    Verify:
    (a) etcd TTL=10s with 5s write interval — what is the false-positive node-dropout rate under high load?
    (b) SQLite WAL suitability for append-only ledger at 50-node scale (concurrent writers, WAL size growth)
    (c) SHA-256 hash chain integrity under concurrent ledger writes (race condition between entries)
    (d) etcd watch stream reliability — does it miss events under compaction or leader election?
    (e) The /swarm/jobs/{id}/request payload size limit — etcd default max value size is 1.5 MB; is this sufficient?
    (f) Whether etcd Raft leader election (typically 150-300ms) causes scheduler downtime and how jobs in-flight are affected
  </domain>

  <domain id="V7" title="Inference Engine Compatibility">
    Verify:
    (a) llama.cpp GGUF support for all claimed model families (Llama 3.1, Mistral, Qwen 2.5, Phi-3.5, DeepSeek-R1)
    (b) distributed-llama's ring topology — does it require a specific llama.cpp version or fork?
    (c) VRAM requirements for Q4_K_M quantization match our model support matrix (e.g., Llama 3.1 8B = 6 GB claim)
    (d) llama.cpp Metal backend stability on Apple Silicon for MPS inference
    (e) ROCm support status in llama.cpp for AMD GPUs (we claim Vulkan fallback — is ROCm better?)
    (f) Rust FFI binding (llama-cpp-rs or mdrokz/rust-llama.cpp) — maintenance status and Tauri v2 compatibility
  </domain>

  <domain id="V8" title="Phase 0 Implementation Risk">
    For the 4-week Phase 0 scope (single device, llama.cpp + LiteLLM + Tauri tray agent):
    (a) Identify the single highest-risk integration point
    (b) Verify LiteLLM custom_llm_provider pattern still exists and is documented (API may have changed)
    (c) Verify Tauri v2 IPC mechanism for Rust→React communication of real-time inference streaming
    (d) Verify Tauri v2 builds on Windows (MSVC toolchain), macOS (Xcode), and Linux (apt dependencies) without custom patches
    (e) Estimate realistic time to first token from a cold Tauri + llama.cpp integration (not counting model download)
    Flag any item that makes the 4-week timeline unrealistic.
  </domain>

  <domain id="V9" title="Cross-Document Consistency">
    Scan for remaining inconsistencies across project.md, architecture.md, tech_stack.md, ui_ux.md, governance.md:
    (a) Every numeric value that appears in more than one doc must match exactly
    (b) Every OSS library cited must have a consistent license across all docs
    (c) The scheduler scoring formula must be identical wherever it appears
    (d) The credit formula must be identical wherever it appears
    (e) API key prefixes must be the same in every doc that lists them
    Return a table of every inconsistency found with source locations.
  </domain>

</verification_domains>

<output_schema>
{
  "$schema": "swarm-os-verify/1.0",
  "generated_at": "<ISO-8601 timestamp>",
  "phase_0_go_no_go": "GO | NO-GO | GO-WITH-MITIGATIONS",
  "phase_0_blockers": ["<concise blocker description>"],
  "domains": [
    {
      "id": "V1",
      "title": "<domain title>",
      "status": "VERIFIED | WARNING | CRITICAL | UNKNOWN",
      "confidence": <0-100>,
      "findings": [
        {
          "item": "<specific claim or component verified>",
          "verdict": "CONFIRMED | CAVEAT | REFUTED | UNKNOWN",
          "severity": "BLOCKER | HIGH | MEDIUM | LOW | INFO",
          "detail": "<one sentence: what the community source says>",
          "source": "<GitHub issue URL, RFC, paper, commit, or forum post>",
          "mitigation": "<concrete fix if verdict is not CONFIRMED>"
        }
      ],
      "cross_domain_impact": ["<V3: finding X changes security assumption Y>"]
    }
  ],
  "recommended_action_before_build": [
    {
      "priority": 1,
      "action": "<what to do>",
      "blocks_phase": 0
    }
  ]
}
</output_schema>

<rules>
  MUST: Cite a real community artifact for every non-INFO finding. "Common knowledge" is not a citation.
  MUST: If a finding in one domain changes an assumption in another domain, add a cross_domain_impact entry.
  MUST: Any BLOCKER or CRITICAL finding that affects Phase 0 must appear in phase_0_blockers.
  MUST NOT: Invent URLs or issue numbers. If you cannot find a real source, set confidence ≤ 40 and verdict = UNKNOWN.
  MUST NOT: Emit prose outside the JSON block.
  FORMAT: Minified JSON is acceptable. Keys must match schema exactly.
</rules>

</swarm_os_verify>
