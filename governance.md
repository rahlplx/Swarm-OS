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
    "tokens_to_credits_ratio": 0.008,
    "score_weight_enabled": true,
    "score_bonus_threshold": 80,
    "score_bonus_multiplier": 1.1
  },
  "debit_formula": {
    "input_tokens_cost": 0.006,
    "output_tokens_cost": 0.01,
    "stream_surcharge": 0.0
  },
  "chain": {
    "signature_algorithm": "ed25519",
    "hash_algorithm": "sha256",
    "entry_ttl_days": 365,
    "archive_after_days": 90
  },
  "top_up": {
    "min_purchase_bdt": 50,
    "max_purchase_bdt": 50000,
    "gateway": "sslcommerz",
    "tiers": [
      {"min_bdt": 50,  "max_bdt": 449,  "credits_per_bdt": 2.5,  "label": "Standard"},
      {"min_bdt": 450, "max_bdt": 899,  "credits_per_bdt": 2.67, "label": "Plus (+7%)"},
      {"min_bdt": 900, "max_bdt": null, "credits_per_bdt": 2.89, "label": "Pro (+15%)"}
    ],
    "rounding": "floor",
    "rounding_note": "Credits are always floored to 1 decimal place. e.g. ৳450 × 2.67 = 1201.5 → 1201.5 (floor at 0.1 precision = 1201.5). Displayed as integers in UI (always rounded down to whole credit)."
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
      "condition": "swarm_job_duration_seconds{quantile='0.95'} > 8",
      "severity": "warning",
      "channels": ["slack_webhook"]
    },
    {
      "name": "ledger_chain_break",
      "condition": "swarm_ledger_chain_valid == 0",
      "severity": "critical",
      "channels": ["slack_webhook", "email"]
    }
  ],
  "inhibit_rules": [
    {
      "source_match": {"alertname": "node_dropout_spike"},
      "target_match_re": {"alertname": "queue_overflow|high_latency"},
      "equal": ["swarm_instance"]
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

**ADMIN_ADJUST storage:** Manual credit adjustments are stored in a **separate** `admin_adjustments` table in SQLite, NOT interleaved in the node's hash chain. They reference `node_id`, `job_id`, and `operator_key_id` but do not appear as entries in the hash chain sequence. This means `prev_entry_hash` in a regular entry always points to the previous regular entry; no entries are ever "skipped" during chain traversal.

Each row in `admin_adjustments` has its own tamper-evidence:
```
AdminAdjustment {
  id:               uuid-v4
  node_id:          "node_7f3a..."
  credits_delta:    +14.0
  reason:           "job_9b2c ledger write failure"
  operator_key_id:  "swrm_ops_xxx"       // which operator key issued this
  timestamp:        "2025-06-22T15:00:00Z"
  signature:        Ed25519.sign(operator_private_key, SHA-256(id+node_id+credits_delta+reason+operator_key_id+timestamp))
}
```
**Tamper-evidence:** Each `admin_adjustments` row is individually signed by the issuing operator's Ed25519 key. An attacker who can write to the SQLite file can insert or modify rows, but cannot forge a valid signature without the operator's private key. The audit protocol (§4.2 Step 3) verifies all operator signatures when reconciling the balance. This is the same trust model as the main chain — signatures, not chaining, are the tamper-evidence primitive here. Chaining `admin_adjustments` to the main chain would require ADMIN_ADJUST entries to appear in chain traversal, breaking the contiguity invariant.

### 4.2 — Verification Procedure

Run via Admin Portal or CLI: `swarm-admin ledger verify --node node_7f3a`

```
Step 1: Fetch all ledger entries for node from SQLite WAL on orchestrator (ledger_entries table),
        ordered by timestamp. ADMIN_ADJUST records are in a separate admin_adjustments table
        and are NOT fetched here — they do not appear in the hash chain.
        Note: O(n) over total entry count — batch-paginate for nodes with > 100k entries
Step 2: For each entry:
        a. Verify Ed25519 signature using node's registered public key
        b. Verify prev_entry_hash == SHA-256(previous entry serialized)
           (chain is contiguous — no gaps from admin adjustments since they are in a separate table)
        c. Verify credits_delta matches formula: tokens × rate from config at timestamp
Step 3: Report:
        - Total entries verified
        - First/last timestamp
        - Any entries that fail signature check (potential forgery)
        - Any chain breaks (potential deletion/insertion)
        - Net credits from chain (earn - spend) + admin_adjustments sum should match user's current balance
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
| 0–90 days | SQLite WAL on orchestrator | Real-time query |
| 90 days – 1 year | Compressed JSON in object store (S3/R2) | API query with 5s latency |
| > 1 year | Cold archive (Glacier/B2) | Manual request only |
| Never deleted | Chain verification snapshots | Permanent |

---

## 5. Abuse Prevention

### 5.1 — Fake Contribution Detection

A node could claim to have run inference without doing real work.

**Mitigations:**
- **Challenge-response validation:** Orchestrator occasionally sends a job with a known correct output (golden sample, greedy/temp=0 decode); node's response must match exactly. This is the only automated correctness check — LLM outputs are non-deterministic at temp>0 so peer output comparison produces false positives and is not used.
- **Score degradation:** Nodes that fail challenge-response have their score penalized for 24h.
- **Rate cap on earning:** A node cannot earn more credits than its score allows in a 1-hour window. (`max_credits_per_hour = node_score × 2`)

### 5.2 — API Key Abuse

- Keys with > 10× their tier's RPM in a 60s window → auto-suspended, alert sent
- Tor exit nodes and known datacenter IP ranges → require manual approval
- Same user creating > 5 keys in 24h → flagged for review

### 5.3 — Node Sock-Puppeting

- One person cannot register the same physical GPU under multiple node IDs to earn double credits.
- **Detection:** Hardware fingerprint is **required** for all node types. A GPU UUID alone is software-readable and can be spoofed by a malicious driver; binding it to the motherboard serial + CPU ID raises the attack cost significantly.
  - **NVIDIA:** `SHA-256(gpu_uuid + motherboard_serial + cpu_id + machine_id)` — all four fields concatenated. GPU UUID + machine-id provides primary uniqueness; motherboard serial and CPU ID add depth.
  - **AMD/CPU nodes:** `SHA-256(motherboard_serial + cpu_id + machine_id)` via `sysinfo` crate + `/etc/machine-id` (Linux) / `IOPlatformUUID` (macOS) / registry MachineGuid (Windows).
- **Fallback for generic serials:** Many OEM boards return `"To be filled by O.E.M."`, `"None"`, or empty for motherboard serial. The fingerprint algorithm must detect these sentinel values and exclude the serial field from the hash when any field is blank or a known-generic string. `/etc/machine-id` (Linux) is a high-entropy UUID generated on first OS install and is always present — it is the primary uniqueness anchor. The full priority order: `machine_id` (always used) → `gpu_uuid` (if NVIDIA/AMD) → `cpu_id` (always used) → `motherboard_serial` (only if non-generic).
- Duplicate fingerprint hash → second registration rejected; operator alerted.

---

## 6. Compliance & Privacy

| Requirement | Implementation |
|------------|----------------|
| Data stored in BD | Offer BD-region deployment option (Dhaka VPS). User data (email, credits) stored in BD. Inference data is never persisted. |
| ICT Division compliance | Logging of API requests for audit (no prompt content, only metadata: key_hash, model, token_count, timestamp) |
| GDPR (global users) | Account deletion removes PII; ledger entries are anonymized to node_id only |
| Prompt privacy | Prompts are never logged to disk. In-memory only during inference. Cleared on job completion. |
| Node operator liability | Terms of Service: node operators are not liable for content generated via the swarm. |
