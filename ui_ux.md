# Swarm-OS: UI/UX Specification

> Design principle: **Progressive disclosure.** A new contributor joins in under 3 minutes. An advanced operator can configure every scheduler knob. Neither should see what the other doesn't need.
>
> Stack: Tauri v2 (system tray desktop agent) + Next.js (web portal). Both share the same shadcn/ui component library and Tailwind dark theme. Bangla/English toggle on all surfaces.

---

## Design System

| Token | Value |
|-------|-------|
| Primary color | `#6366F1` (indigo-500) |
| Background | `#0F0F11` (near-black) |
| Surface | `#1A1A1F` |
| Border | `#2A2A32` |
| Text primary | `#F4F4F5` |
| Text muted | `#71717A` |
| Success | `#22C55E` |
| Warning | `#F59E0B` |
| Danger | `#EF4444` |
| Font | Inter (Latin) + Hind Siliguri (Bangla) |
| Radius | `8px` (components), `12px` (cards) |
| Motion | 150ms ease-out for transitions; no animations during inference |

**Dark mode first.** Light mode is optional and deprioritized for v1.

---

## Surface 1: Web Portal (Next.js)

Used for: account creation, API key management, ledger top-up, admin governance.  
Deployed at: `app.swarm-os.dev` (Cloudflare Pages)

---

### Screen 1.1 — Landing / Login

**Goal:** Convert visitor to registered contributor or consumer in < 60 seconds.

```
┌──────────────────────────────────────────────────────────┐
│  SWARM-OS                                    [EN | বাং]  │
├──────────────────────────────────────────────────────────┤
│                                                          │
│         Your idle GPU earns.                             │
│         Your AI runs free.                               │
│                                                          │
│   [▶ Join the Swarm — Download Agent]                    │
│   [→ Use the API — Get API Key]                          │
│                                                          │
│   ─────────── or sign in ───────────                     │
│                                                          │
│   [GitHub OAuth]   [Google OAuth]   [Email + Password]   │
│                                                          │
│   Live stats (auto-refreshed):                           │
│   ● 47 nodes active   ● 12.3M tokens today   ● 8 models │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

**UX rules:**
- No modal on landing. Inline sign-in form expands on button click.
- Live stats fetched from public `/api/v1/swarm/stats` — no auth required.
- "Join the Swarm" CTA is primary (contributor-first positioning).
- Language toggle persists to `localStorage`; Bangla renders Hind Siliguri font.

---

### Screen 1.2 — Onboarding: Contributor Path

**Goal:** Node agent installed, resource limit configured, node registered — in < 3 minutes.

**Step 1 of 4 — Download Agent**
```
┌──────────────────────────────────────────────────────────┐
│  Welcome to Swarm-OS                          Step 1 / 4 │
│  ══════════════════════════════════════                   │
│                                                          │
│  Install the Node Agent on this device                   │
│                                                          │
│  Detected OS: Linux x86_64                               │
│                                                          │
│  [↓ Download swarm-agent-linux-x64.AppImage  (18 MB)]   │
│  [↓ Windows .exe]  [↓ macOS .dmg]  [↓ Debian .deb]     │
│                                                          │
│  SHA-256: a3f9... (verify before running)                │
│                                                          │
│  Already installed?  [Skip → I have the agent]           │
│                                                          │
│                               [Next →]                   │
└──────────────────────────────────────────────────────────┘
```

**Step 2 of 4 — Hardware Detection** *(rendered in agent after launch)*
```
┌──────────────────────────────────────────────────────────┐
│  What can your device contribute?             Step 2 / 4 │
│  ══════════════════════════════════════                   │
│                                                          │
│  Detected hardware:                                      │
│  ┌────────────────────────────────────────────────┐      │
│  │  GPU   NVIDIA RTX 3090   24 GB VRAM   [CUDA]  │      │
│  │  RAM   64 GB available                         │      │
│  │  CPU   16 cores  (AMD Ryzen 9 5900X)           │      │
│  │  Score ████████████████░░  97 / 100            │      │
│  └────────────────────────────────────────────────┘      │
│                                                          │
│  Models you can run solo:  Llama-3.1-70B, Qwen-2.5-72B  │
│  Models needing pooling:   Llama-3.1-405B                │
│                                                          │
│  [← Back]                             [Next →]           │
└──────────────────────────────────────────────────────────┘
```

**Step 3 of 4 — Resource Limits**
```
┌──────────────────────────────────────────────────────────┐
│  How much will you share?                     Step 3 / 4 │
│  ══════════════════════════════════════                   │
│                                                          │
│  GPU compute to donate                                   │
│  ░░░░░░░░░░░░████████████  75%                           │
│  (drag slider — 0% = paused, 100% = all-in)             │
│                                                          │
│  Active hours                                            │
│  ● Always on                                             │
│  ○ Schedule:  [22:00] → [08:00]  (idle hours only)      │
│                                                          │
│  Auto-pause when:                                        │
│  ☑ I start gaming / GPU load > 80%                      │
│  ☑ Battery below 20% (laptops)                           │
│  ☐ Screen saver is not active                            │
│                                                          │
│  [← Back]                             [Next →]           │
└──────────────────────────────────────────────────────────┘
```

**Step 4 of 4 — Join Swarm**
```
┌──────────────────────────────────────────────────────────┐
│  You're ready to join                         Step 4 / 4 │
│  ══════════════════════════════════════                   │
│                                                          │
│  Your join token (auto-filled from account):             │
│  swrm_node_xxxxxxxxxxxxxxxx  [Copy]                      │
│                                                          │
│  ┌─ Agent Status ────────────────────────────────────┐   │
│  │  ● Connecting to mesh...                          │   │
│  │  ● Registering node...                            │   │
│  │  ✓ Node online! ID: node_7f3a...                  │   │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  You're now earning credits.                             │
│  Estimated: ~120 credits/day at current settings         │
│                                                          │
│              [Open Dashboard →]                          │
└──────────────────────────────────────────────────────────┘
```

---

### Screen 1.3 — Dashboard (Command Bridge)

**Goal:** Real-time swarm health at a glance + primary inference interface.

```
┌──────────────────────────────────────────────────────────────────┐
│  SWARM-OS    Dashboard              [⚙ Settings]  [👤 rahim@..]  │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─ Swarm Health ──────────────────────────────────────────┐     │
│  │                                                         │     │
│  │  ●47 Nodes    ⚡ 2,340 tok/s    📋 3 queued    0 errors │     │
│  │                                                         │     │
│  │  VRAM pool: ████████████████████████░░░░░  72% used    │     │
│  │  Capacity:  320 GB total  |  89 GB free                 │     │
│  └─────────────────────────────────────────────────────────┘     │
│                                                                  │
│  ┌─ Your Credits ────────────────┐  ┌─ Quick Inference ───────┐  │
│  │                               │  │                         │  │
│  │  Balance:  847 credits        │  │  Model                  │  │
│  │  Earned today:  +120          │  │  [Llama-3.1-70B      ▼] │  │
│  │  Spent today:   -34           │  │                         │  │
│  │                               │  │  Prompt                 │  │
│  │  Your node: ● Online          │  │  ┌─────────────────┐    │  │
│  │  Contributing: 75% GPU        │  │  │                 │    │  │
│  │  Score: 97                    │  │  └─────────────────┘    │  │
│  │                               │  │  [▶ Run  (12 credits)]  │  │
│  │  [Top Up →]                   │  │                         │  │
│  └───────────────────────────────┘  └─────────────────────────┘  │
│                                                                  │
│  ┌─ Recent Jobs ─────────────────────────────────────────────┐   │
│  │  #  Model             Status    Tokens   Time    Credits   │   │
│  │  1  llama-3.1-70b    ✓ Done    1,204    4.2s    -14       │   │
│  │  2  mistral-7b        ✓ Done      487    1.1s    -6        │   │
│  │  3  llama-3.1-405b   ● Running    ...    ...     ...       │   │
│  └───────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

**UX rules:**
- Swarm Health bar auto-refreshes every 5s via WebSocket.
- "Quick Inference" runs a single completion; full playground is at `/playground`.
- Credit cost is estimated before submission — user sees cost before clicking Run.
- Running jobs stream tokens live in a results panel that slides in from the right.

---

### Screen 1.4 — Playground (Full Inference UI)

```
┌──────────────────────────────────────────────────────────────────┐
│  Playground                                                       │
├────────────────────────────────┬─────────────────────────────────┤
│  LEFT: Configuration           │  RIGHT: Response Stream          │
│                                │                                  │
│  Model                         │  ┌──────────────────────────┐   │
│  [Llama-3.1-70B           ▼]  │  │                          │   │
│  Routed to: 3 nodes            │  │  The quick brown fox...  │   │
│                                │  │  ▌ (streaming)           │   │
│  System Prompt                 │  │                          │   │
│  ┌──────────────────────────┐  │  └──────────────────────────┘   │
│  │ You are a helpful...     │  │                                  │
│  └──────────────────────────┘  │  Stats                           │
│                                │  Tokens: 312 in / 1,024 out      │
│  User Message                  │  Speed: 48 tok/s                 │
│  ┌──────────────────────────┐  │  Latency (first token): 1.8s    │
│  │                          │  │  Credits: -18                    │
│  │                          │  │  Nodes used: A (layers 0-19)    │
│  └──────────────────────────┘  │             B (layers 20-39)    │
│                                │                                  │
│  Temperature   [0.7      ]     │  [Copy]  [Save]  [New Chat]      │
│  Max tokens    [2048     ]     │                                  │
│  Stream        [● On     ]     │                                  │
│                                │                                  │
│  [▶ Generate  (~18 credits)]   │                                  │
└────────────────────────────────┴─────────────────────────────────┘
```

---

### Screen 1.5 — Ledger

**Goal:** Full transparency on credit earning and spending.

```
┌──────────────────────────────────────────────────────────────────┐
│  Ledger                                            [Export CSV]  │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─ Summary (30 days) ─────────────────────────────────────┐     │
│  │  Earned:   +3,240 credits   (27,000 tokens generated)   │     │
│  │  Spent:    -1,080 credits   (9,000 tokens consumed)      │     │
│  │  Balance:  2,160 credits    Net contributor ✓            │     │
│  └─────────────────────────────────────────────────────────┘     │
│                                                                  │
│  Earning Chart  (credits/day)                                    │
│  ┌───────────────────────────────────────────────────────┐       │
│  │  120 ┤  ██  ██  ██     ██  ██  ██  ██  ██  ██  ██   │       │
│  │   80 ┤  ██  ██  ██  ▄  ██  ██  ██  ██  ██  ██  ██   │       │
│  │   40 ┤  ██  ██  ██  █  ██  ██  ██  ██  ██  ██  ██   │       │
│  │    0 └──────────────────────────────────────── days   │       │
│  └───────────────────────────────────────────────────────┘       │
│                                                                  │
│  Transaction History                         [Filter ▼] [Search] │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │  Date         Type      Model            Tokens  Credits │     │
│  │  Jun 22 14:32  Earned   llama-3.1-70b    1,204   +14    │     │
│  │  Jun 22 14:28  Spent    mistral-7b          487    -6    │     │
│  │  Jun 22 14:10  Earned   qwen-2.5-72b      3,100   +37   │     │
│  │  Jun 22 09:00  Top-up   SSLCommerz/bKash    —    +500   │     │
│  └─────────────────────────────────────────────────────────┘     │
│                                                                  │
│  [← Prev Page]                               [Next Page →]       │
└──────────────────────────────────────────────────────────────────┘
```

---

### Screen 1.6 — Top-Up (BD Payment)

```
┌──────────────────────────────────────────────────────────┐
│  Add Credits                                             │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  Select amount                                           │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│  │ 500 cr   │ │ 1,200 cr │ │ 2,600 cr │ │ Custom   │   │
│  │ ৳200     │ │ ৳450     │ │ ৳900     │ │          │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │
│                                                          │
│  Pay with                                                │
│  ● bKash          ○ Nagad          ○ Card (Visa/MC)     │
│                                                          │
│  [Pay ৳200 via bKash →]                                 │
│                                                          │
│  Credits never expire. 1 credit ≈ 100 output tokens.    │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

---

### Screen 1.7 — API Keys

```
┌──────────────────────────────────────────────────────────┐
│  API Keys                                [+ New Key]     │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  Base URL:  https://api.swarm-os.dev/v1                  │
│  Compatible with any OpenAI SDK — just change baseURL.   │
│                                                          │
│  ┌─────────────────────────────────────────────────┐     │
│  │  Key Name    Created      Last Used    Credits   │     │
│  │  prod-app    Jun 15       2h ago       847 ●     │     │
│  │  test-key    Jun 20       Never        50        │     │
│  │                           [Copy] [Revoke]        │     │
│  └─────────────────────────────────────────────────┘     │
│                                                          │
│  Quick start (Python):                                   │
│  ┌─────────────────────────────────────────────────┐     │
│  │  from openai import OpenAI                       │     │
│  │  client = OpenAI(                                │     │
│  │    api_key="swrm_sk_...",                        │     │
│  │    base_url="https://api.swarm-os.dev/v1"        │     │
│  │  )                                               │     │
│  └─────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────┘
```

---

## Surface 2: Tauri Desktop Agent (System Tray)

Used for: resource contribution, node status, earning display. Minimal UI — lives in system tray.

---

### Screen 2.1 — Tray Popup (Collapsed)

```
  ┌──────────────────────────────┐
  │  SWARM-OS  ●  Online         │
  │  ─────────────────────────── │
  │  Earned today   +120 credits │
  │  Active jobs    2            │
  │  Tokens/sec     47           │
  │  ─────────────────────────── │
  │  [Pause]  [Dashboard ↗]      │
  └──────────────────────────────┘
```

### Screen 2.2 — Tray Popup (Expanded / Settings)

```
  ┌──────────────────────────────────┐
  │  SWARM-OS  ●  Online             │
  │  ────────────────────────────── │
  │  GPU donate    [████████░░] 75%  │
  │  Auto-pause    ☑ on GPU > 80%   │
  │  Schedule      22:00 – 08:00     │
  │  ────────────────────────────── │
  │  Node ID:  node_7f3a...  [Copy]  │
  │  Version:  0.2.1  [Check update] │
  │  ────────────────────────────── │
  │  [Quit Agent]                    │
  └──────────────────────────────────┘
```

### Screen 2.3 — Notification (Toast — OS native)

Triggered on: first job received, daily credit summary, node dropout detected.

```
  ┌──────────────────────────────────────┐
  │  🟢 Swarm-OS                         │
  │  Job assigned: llama-3.1-70b         │
  │  Running shard 0–19 (24GB VRAM)      │
  └──────────────────────────────────────┘
```

---

## Surface 3: Grafana Analytics (Operator View)

Pre-built dashboards — operators self-host or use Grafana Cloud free tier.

### Dashboard 1 — Swarm Overview

**Panels:**
1. Active Nodes (gauge: 0 → 100)
2. Aggregate Tokens/sec (time series, 24h)
3. Job Queue Depth (time series)
4. VRAM Pool Utilization % (stat)
5. P95 First-Token Latency (time series)

### Dashboard 2 — Node Inspector

**Variables:** `$node_id` dropdown

**Panels:**
1. Node score over time
2. VRAM used / free (area chart)
3. Tokens generated (counter)
4. Credits earned (counter)
5. Job success / failure ratio (bar)

### Dashboard 3 — Ledger Analytics

**Panels:**
1. Credits issued vs. spent (stacked area, 30d)
2. Top 10 earners (table)
3. Top 10 consumers (table)
4. Credit balance distribution (histogram)

### Dashboard 4 — Special Days View

**Goal:** Understand demand spikes (Eid, exam season, hackathons).

**Panels:**
1. Annotated time series with manual event markers (Eid, SSC exams, etc.)
2. Peak hours heatmap (hour of day × day of week)
3. Capacity vs. demand ratio over time
4. BD public holiday overlay (imported from `date-holidays` npm package data)

---

## UX Flow Summary

```
New User
    │
    ├── Contributor Path
    │     Landing → Sign Up → Download Agent → Hardware Detection
    │     → Set Resource Limits → Join Swarm → Dashboard (earning)
    │     Total time: < 3 minutes
    │
    └── Consumer Path
          Landing → Sign Up → Get API Key → Top Up Credits
          → Paste base_url into OpenAI SDK → First inference
          Total time: < 2 minutes

Returning User (Tray Agent)
    App start → Tray icon appears → Auto-join swarm
    → Passive earning (no interaction required)

Admin / Operator
    Admin Portal → Node Management → Ledger Audit
    → Grafana Dashboards → Alert Configuration
```

---

## Accessibility & BD-Specific Rules

| Requirement | Implementation |
|------------|----------------|
| Bangla UI | `react-i18next`, Hind Siliguri font loaded via Google Fonts |
| BDT currency | Always display `৳` prefix; credits shown alongside BDT equivalent |
| Low-bandwidth | No autoplay video; images lazy-loaded; API responses gzipped |
| Mobile web | Responsive down to 375px; tray agent is desktop-only |
| Color blind | Icons + text labels on every status indicator; never color-only |
| Keyboard nav | All shadcn/ui components are keyboard accessible by default |
| Error messages | Plain language: "Your node disconnected. Check your internet." not error codes |
