# Swarm-OS: Governance

> The swarm is a commons. Governance defines who controls what, how abuse is prevented, how the ledger stays honest, and how operators can configure behavior without touching code.

---

## 1. Role Hierarchy

```
┌──────────────────────────────────────────────────────┐
│                   SUPER ADMIN                        │
│  Full system access. Swarm-OS core team only.        │
│  Can: modify scheduler policy, pause entire swarm,   │
│  reset ledger chain, manage admin accounts.          │
└───────────────────────────┬──────────────────────────┘
                            │
┌───────────────────────────▼──────────────────────────┐
│                     OPERATOR                         │
│  Swarm instance owner (self-hosters, BD partners).   │
│  Can: manage nodes, users, API keys, view all logs,  │
│  configure rate limits, export ledger.               │
└───────────────────────────┬──────────────────────────┘
                            │
        ┌───────────────────┴───────────────────┐
        │                                       │
┌───────▼────────┐                    ┌─────────▼────────┐
│  CONTRIBUTOR   │                    │    CONSUMER      │
│  Runs a node.  │                    │  Uses the API.   │
│  Earns credits.│                    │  Spends credits. │
│  Cannot modify │                    │  Cannot see other│
│  other nodes   │                    │  users' data.    │
│  or user data. │                    │                  │
└────────────────┘                    └──────────────────┘
```

API key prefixes by role:

| Role | Key Prefix | Permissions |
|------|-----------|-------------|
| Super Admin | `swrm_adm_` | Full read/write on all APIs |
| Operator | `swrm_ops_` | Node mgmt, user mgmt, ledger export |
| Contributor | `swrm_node_` | Heartbeat, job result submission only |
| Consumer | `swrm_sk_` | Inference API, own ledger only |

---

## 2. Admin Portal Screens

### 2.1 — Node Management

```
┌──────────────────────────────────────────────────────────────────┐
│  Admin › Nodes                              [+ Manual Register]  │
├──────────────────────────────────────────────────────────────────┤
│  Filter: [All ▼]  [Search node ID or owner...]                  │
│                                                                  │
│  ID           Owner      Score  Status   Jobs  Uptime  Action   │
│  node_7f3a..  rahim@..   97     ● Online  2     99.2%  [⚙][✗]  │
│  node_2b1c..  nadia@..   72     ● Online  1     97.8%  [⚙][✗]  │
│  node_9d4e..  karim@..   34     ○ Idle    0     81.3%  [⚙][✗]  │
│  node_0a8f..  unknown    —      ✗ Banned  0     —      [✓]      │
│                                                                  │
│  [⚙] = Configure   [✗] = Ban   [✓] = Unban                     │
└──────────────────────────────────────────────────────────────────┘
```

**Node configuration modal:**
- Override capability score (if auto-detection is wrong)
- Set max jobs per node (default: 3 concurrent)
- Force-pause node (e.g., maintenance)
- View last 100 jobs run on this node

### 2.2 — User & Key Management

```
┌──────────────────────────────────────────────────────────────────┐
│  Admin › Users                                     [+ Invite]   │
├──────────────────────────────────────────────────────────────────┤
│  User            Tier       Balance  Keys  Joined    Action      │
│  rahim@..        Standard   847 cr   2     Jun 1     [⚙][Ban]   │
│  nadia@..        Pro        4,200 cr 5     May 15    [⚙][Ban]   │
│  spam@..         —          0 cr     0     Jun 22    [✓Banned]  │
├──────────────────────────────────────────────────────────────────┤
│  API Keys (all users)                                            │
│  Key (masked)     Owner      Tier      Req/min  Credits  Status  │
│  swrm_sk_xx..     rahim@..   Standard  60       847      ● Active│
│  swrm_sk_yy..     nadia@..   Pro       300      4,200    ● Active│
│                                        [Revoke] [Edit Limits]   │
└──────────────────────────────────────────────────────────────────┘
```

### 2.3 — Rate Limit Tiers

Operators define tiers. Users are assigned a tier. Tiers control:
- Requests per minute (RPM)
- Tokens per minute (TPM)
- Concurrent jobs allowed
- Model access (some models may be operator-restricted)

```
Admin › Rate Limits

  ┌─────────────┬──────────┬───────────┬──────────┬──────────────┐
  │ Tier        │ RPM      │ TPM       │ Conc.    │ Models       │
  ├─────────────┼──────────┼───────────┼──────────┼──────────────┤
  │ Free        │ 10       │ 10,000    │ 1        │ ≤7B only     │
  │ Standard    │ 60       │ 100,000   │ 3        │ All          │
  │ Pro         │ 300      │ 500,000   │ 10       │ All          │
  │ Enterprise  │ Custom   │ Custom    │ Custom   │ All          │
  └─────────────┴──────────┴───────────┴──────────┴──────────────┘
  [+ Add Tier]  [Edit]  [Delete]
```

---

## 3. Configuration Schemas

All configuration is stored in etcd under `/swarm/config/` and validated against these schemas on write. Operators can update via Admin Portal or REST API.

### 3.1 — Scheduler Policy

```json
// /swarm/config/scheduler
{
  "version": 1,
  "strategy": "capability_weighted",
  "locality_bonus_weight": 0.15,
  "prefilter": {
    "require_vram_headroom_pct": 10,
    "require_ram_headroom_gb": 2,
    "max_node_load_pct": 90,
    "backend_compatibility_strict": true
  },
  "scoring": {
    "vram_weight": 4.0,
    "ram_weight": 0.5,
    "cpu_weight": 0.25,
    "backend_bonus": {
      "cuda": 10,
      "metal": 8,
      "vulkan": 5,
      "cpu": 0
    }
  },
  "sharding": {
    "min_shard_vram_gb": 4,
    "max_nodes_per_job": 8,
    "topology": "ring"
  },
  "failover": {
    "node_timeout_ms": 10000,
    "max_requeue_attempts": 3,
    "requeue_backoff_ms": 500
  }
}
```

### 3.2 — Rate Limit Tier Schema

```json
// /swarm/config/rate_limits/standard
{
  "tier_id": "standard",
  "display_name": "Standard",
  "requests_per_minute": 60,
  "tokens_per_minute": 100000,
  "max_concurrent_jobs": 3,
  "allowed_models": "*",
  "max_context_tokens": 32768,
  "streaming_allowed": true,
  "priority": 5
}
```

### 3.3 — Model Allowlist

```json
// /swarm/config/models
{
  "version": 1,
  "models": [
    {
      "id": "llama-3.1-8b",
      "display_name": "Llama 3.1 8B",
      "gguf_filename": "Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf",
      "min_vram_gb": 6,
      "min_ram_gb": 8,
      "context_length": 131072,
      "required_backend": ["cuda", "metal", "vulkan", "cpu"],
      "shardable": false,
      "enabled": true
    },
    {
      "id": "llama-3.1-70b",
      "display_name": "Llama 3.1 70B",
      "gguf_filename": "Meta-Llama-3.1-70B-Instruct-Q4_K_M.gguf",
      "min_vram_gb": 40,
      "min_ram_gb": 16,
      "context_length": 131072,
      "required_backend": ["cuda", "metal"],
      "shardable": true,
      "min_shard_vram_gb": 8,
      "enabled": true
    }
  ]
}
```

### 3.4 — Ledger Policy

```json
// /swarm/config/ledger
{
  "version": 1,
  "credit_formula": {
    "tokens_to_credits_ratio": 0.012,
    "score_weight_enabled": true,
    "score_bonus_threshold": 80,
    "score_bonus_multiplier": 1.1
  },
  "debit_formula": {
    "input_tokens_cost": 0.006,
    "output_tokens_cost": 0.009,
    "stream_surcharge": 0.0
  },
  "chain": {
    "signature_algorithm": "ed25519",
    "hash_algorithm": "sha256",
    "entry_ttl_days": 365,
    "archive_after_days": 90
  },
  "top_up": {
    "min_purchase_bdt": 100,
    "max_purchase_bdt": 50000,
    "gateway": "sslcommerz",
    "credits_per_bdt": 2.5
  }
}
```

### 3.5 — Alerting Config

```json
// /swarm/config/alerts
{
  "channels": {
    "slack_webhook": "https://hooks.slack.com/...",
    "email": ["ops@swarm-os.dev"]
  },
  "rules": [
    {
      "name": "node_dropout_spike",
      "condition": "swarm_nodes_active < 5",
      "severity": "critical",
      "channels": ["slack_webhook", "email"]
    },
    {
      "name": "queue_overflow",
      "condition": "swarm_job_queue_depth > 50",
      "severity": "warning",
      "channels": ["slack_webhook"]
    },
    {
      "name": "high_latency",
      "condition": "swarm_job_duration_seconds{quantile='0.95'} > 10",
      "severity": "warning",
      "channels": ["slack_webhook"]
    },
    {
      "name": "ledger_chain_break",
      "condition": "swarm_ledger_chain_valid == 0",
      "severity": "critical",
      "channels": ["slack_webhook", "email"]
    }
  ]
}
```

---

## 4. Ledger Audit Protocol

### 4.1 — How the Ledger Works

Every credit event (earn or spend) generates a signed ledger entry:

```
LedgerEntry {
  id:               uuid-v4
  node_id:          "node_7f3a..."       // contributor or consumer key
  job_id:           "job_9b2c..."
  type:             "earn" | "spend"
  tokens:           1204
  credits_delta:    +14.448
  timestamp:        "2025-06-22T14:32:01.123Z"
  prev_entry_hash:  SHA-256(serialized previous entry)
  signature:        Ed25519.sign(node_private_key, SHA-256(this entry minus signature field))
}
```

**Tamper-evidence:** Changing any field in entry N invalidates `prev_entry_hash` in entry N+1, breaking the chain. Verification requires the node's public key (registered in Blackboard at join time).

### 4.2 — Verification Procedure

Run via Admin Portal or CLI: `swarm-admin ledger verify --node node_7f3a`

```
Step 1: Fetch all ledger entries for node from etcd, ordered by timestamp
Step 2: For each entry:
        a. Verify Ed25519 signature using node's registered public key
        b. Verify prev_entry_hash == SHA-256(previous entry serialized)
        c. Verify credits_delta matches formula: tokens × rate from config at timestamp
Step 3: Report:
        - Total entries verified
        - First/last timestamp
        - Any entries that fail signature check (potential forgery)
        - Any chain breaks (potential deletion/insertion)
        - Net credits (earn - spend) should match user's current balance
```

### 4.3 — Ledger Export

**Formats:** JSON (full entry with signatures), CSV (human-readable summary)

**CSV columns:**
```
timestamp, node_id, job_id, type, model, tokens, credits_delta, running_balance, chain_valid
```

**Export via Admin Portal:** Admin › Ledger › Export › [Select date range] › [JSON | CSV]

**Export via CLI:**
```bash
swarm-admin ledger export \
  --node node_7f3a \
  --from 2025-06-01 \
  --to 2025-06-30 \
  --format csv \
  --verify-chain \
  > ledger-june.csv
```

### 4.4 — Dispute Resolution Flow

When a user disputes a credit charge or earning discrepancy:

```
1. User submits dispute via portal: "I didn't receive credits for job_9b2c"

2. Operator action:
   a. Look up job_9b2c in etcd /swarm/jobs/job_9b2c/
   b. Verify job status = "done"
   c. Run: swarm-admin ledger verify --job job_9b2c
   d. Check if ledger entry exists for this job for this node

3. Outcomes:
   a. Entry exists, valid → Dispute rejected; show user the signed entry
   b. Entry exists, signature invalid → Forgery detected; alert + manual review
   c. Entry missing, job completed → Ledger write failure; issue manual credit adjustment
   d. Job status = "failed" → No credits owed; explain to user

4. Manual credit adjustment (operator only):
   swarm-admin ledger adjust --user rahim@swarm-os.dev --credits +14 --reason "job_9b2c ledger write failure"
   (Creates a special ADMIN_ADJUST entry, signed with operator key, not included in chain verification)
```

### 4.5 — Archival Policy

| Period | Storage | Access |
|--------|---------|--------|
| 0–90 days | Live etcd | Real-time query |
| 90 days – 1 year | Compressed JSON in object store (S3/R2) | API query with 5s latency |
| > 1 year | Cold archive (Glacier/B2) | Manual request only |
| Never deleted | Chain verification snapshots | Permanent |

---

## 5. Abuse Prevention

### 5.1 — Fake Contribution Detection

A node could claim to have run inference without doing real work.

**Mitigations:**
- **Challenge-response validation:** Orchestrator occasionally sends a job with a known correct output (golden sample); node's response must match within tolerance.
- **Peer cross-validation:** Same prompt sent to 2 nodes; outputs compared. If they diverge beyond threshold, both flagged for manual review.
- **Score degradation:** Nodes that fail validation have their score penalized for 24h.
- **Rate cap on earning:** A node cannot earn more credits than its score allows in a 1-hour window. (`max_credits_per_hour = node_score × 2`)

### 5.2 — API Key Abuse

- Keys with > 10× their tier's RPM in a 60s window → auto-suspended, alert sent
- Tor exit nodes and known datacenter IP ranges → require manual approval
- Same user creating > 5 keys in 24h → flagged for review

### 5.3 — Node Sock-Puppeting

- One person cannot register the same physical GPU under multiple node IDs to earn double credits.
- **Detection:** NVIDIA GPU UUID is included in capability registration. Duplicate GPU UUIDs → second registration rejected.
- **AMD/CPU nodes:** Use a hardware fingerprint hash (motherboard serial + CPU ID via `sysinfo`).

---

## 6. Compliance & Privacy

| Requirement | Implementation |
|------------|----------------|
| Data stored in BD | Offer BD-region deployment option (Dhaka VPS). User data (email, credits) stored in BD. Inference data is never persisted. |
| ICT Division compliance | Logging of API requests for audit (no prompt content, only metadata: key_hash, model, token_count, timestamp) |
| GDPR (global users) | Account deletion removes PII; ledger entries are anonymized to node_id only |
| Prompt privacy | Prompts are never logged to disk. In-memory only during inference. Cleared on job completion. |
| Node operator liability | Terms of Service: node operators are not liable for content generated via the swarm. |
