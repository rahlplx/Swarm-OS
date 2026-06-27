# Swarm-OS UI/UX Verification & Redesign Prompt

> **Purpose:** Give this prompt to an AI reviewer to audit, verify, and redesign Swarm-OS UI screens against enterprise-grade UX standards using shadcn/ui components. The reviewer must produce device-agnostic, accessible, production-ready screen specifications.

---

## 1. PROJECT CONTEXT

**Swarm-OS** is a decentralized P2P AI inference network built in Bangladesh. Contributors pool idle GPU/CPU compute into a mesh; consumers run 7B-70B LLMs at zero recurring cloud cost. Everyone earns credits for what they give.

| Field | Value |
|-------|-------|
| **Product Name** | Swarm-OS |
| **Tagline** | Your idle GPU earns. Your AI runs free. |
| **Origin** | Bangladesh |
| **Category** | Decentralized AI Inference Network |
| **Tech Stack** | Tauri v2 (Rust backend) + React (frontend) |
| **License** | Apache 2.0 (strictly NO GPL) |
| **Target Users** | BD freelancers, developers, students, GPU owners |

### User Roles

| Role | Description | Needs |
|------|-------------|-------|
| **Contributor** | Runs a node, earns credits | View earnings, node health, credit history |
| **Consumer** | Uses API, spends credits | Credit balance, API keys, model selection, usage |
| **Operator** | Manages the swarm | Node oversight, user management, ledger audit, BDT payments |
| **Super Admin** | Core team only | Full system access, governance controls |

---

## 2. CORE FEATURES (UI MUST COVER ALL)

### F1: Heterogeneous Resource Pooling
- Auto-detect CPU, RAM, VRAM, GPU model
- Support: NVIDIA CUDA, AMD ROCm, Apple Metal, CPU-only
- Two-phase scheduling: hard-gate pre-filter then weighted scoring
- **UI Need:** Dashboard showing node capabilities, hardware profiles, scoring breakdown

### F2: P2P Mesh Networking
- WireGuard overlay, zero-config join
- NAT traversal, DERP relay fallback
- **UI Need:** Network topology map, connection quality indicators, node status

### F3: Distributed State (Blackboard)
- etcd v3 key-space: nodes, jobs, ledger, config
- TTL-based heartbeat (10s expiry, 5s write interval)
- Node dropout = automatic re-assignment
- **UI Need:** Real-time node status, job queue visualization

### F4: AI Inference Engine
- llama.cpp backend (GGUF models)
- Model sharding across nodes (pipeline ring topology)
- Supported: Llama 3, Mistral, Qwen, Gemma, Phi
- **UI Need:** Model selection, inference status, shard assignment visualization

### F5: OpenAI-Compatible API Gateway
- LiteLLM proxy facade
- Per-key auth, rate limiting, usage tracking
- **UI Need:** API key management, usage analytics, rate limit configuration

### F6: Contribution Ledger
- Append-only, timestamped, node-signed (Ed25519)
- Credit formula: earn 0.008/token, spend 0.006 input + 0.01 output
- ~20% platform spread
- **UI Need:** Ledger audit trail, credit history, balance display, BDT payment integration

### F7: Observability Stack
- Prometheus metrics, Grafana dashboards
- Custom metrics: tokens/s, VRAM%, job queue depth
- Alertmanager integration
- **UI Need:** Metrics visualization, alert management, system health overview

### F8: Tauri Desktop Agent
- System tray app: Join Swarm, Pause, View Ledger
- Resource throttle slider
- Live stats
- **UI Need:** Minimal tray UI, settings panel

### F9: Admin Governance Portal
- Node whitelist/blacklist, capability overrides
- API key management, rate limit tiers
- Ledger audit export (CSV/JSON)
- BD-specific: BDT display, Bangla UI option
- **UI Need:** Full admin dashboard with all management screens

---

## 3. BANGLADESH-SPECIFIC REQUIREMENTS (NON-NEGOTIABLE)

| Constraint | UI Impact |
|------------|-----------|
| **CGNAT / ISP NAT** | Show connection quality, DERP relay status |
| **Low bandwidth** | Lazy-load, compressed images, minimal animations |
| **BDT payments** | SSLCommerz/bKash/Nagad integration, BDT currency display (৳) |
| **Bangla speakers** | i18n support, Bangla UI toggle, RTL-ready layout |
| **Power cuts / node dropout** | Offline indicators, reconnection status, TTL countdown |
| **Trust/reputation** | Node reputation scores, uptime history |
| **Local latency** | Same-ASN/same-city preference indicators |

### Credit Economy Display
- Credit balance always visible (top-right or sidebar)
- BDT-to-credit conversion rates (Standard 2.5, Plus 2.67, Pro 2.89)
- Top-up flow: bKash, Nagad, Visa/Mastercard via SSLCommerz
- Transaction history with status indicators

---

## 4. DESIGN SYSTEM (CURRENT TOKENS — REDESIGN IF NEEDED)

### Color Palette
| Token | Hex | Usage |
|-------|-----|-------|
| `bg-deep-slate` | `#0F0F11` | Main Background |
| `accent-cyber-amber` | `#FFB800` | Active / GPU Load |
| `accent-warning-rose` | `#FF4D4D` | Latency Spike / Danger |
| `accent-success-teal` | `#00C4B4` | Ledger Finalized / Success |
| `surface-obsidian` | `#1A1A1F` | Component Backgrounds |
| `surface-card` | `#1C2536` | Card Backgrounds |
| `surface-hover` | `#242E40` | Hover States |
| `text-primary` | `#E2E2E9` | Primary Text |
| `text-secondary` | `#8D9099` | Secondary Text |
| `border-default` | `#44474E` | Borders |

### Typography
- Primary: Inter or system sans-serif
- Monospace: JetBrains Mono (for hashes, node IDs)
- Scale: 11px labels, 14px body, 16px subheadings, 20px headings, 24px page titles, 36px stat values

### Spacing & Layout
- Border radius: 28px (cards), 16px (large), 12px (medium), 8px (small)
- Padding: 16px-24px for cards, 12px-16px for table rows
- Grid: 24px gaps for multi-column layouts
- Sidebar: 240px width, collapsible
- Top bar: 64px height, fixed

---

## 5. EXISTING SCREENS (AUDIT THESE)

### Screen 01: Orchestration Dashboard (`01-dashboard.html`)
**Content:**
- Top bar with system status, alerts count, language toggle, user menu
- Sidebar with role-based navigation
- Stats grid: Active Nodes, Daily Tokens, P95 Latency, Credits Issued
- Swarm Map: interactive force-directed graph (placeholder)
- Recent Activity: live event feed with status icons

**Issues to check:**
- Is the information hierarchy clear?
- Are stats scannable at a glance?
- Is the swarm map actually useful or just decorative?
- Mobile responsiveness of the grid layout

### Screen 02: Node Detail View (`02-node-detail.html`)
**Content:**
- Node header with status, score, uptime
- Hardware profile: GPU, VRAM, RAM, CPU, Backend
- Network metrics: RTT, Jitter, Bandwidth, ASN, City
- Confidence Score visualization (0-100%)
- Activity log with recent jobs
- Throughput chart (tokens/s over time)

**Issues to check:**
- Is the confidence score meaningful and actionable?
- Are hardware specs readable and properly prioritized?
- Is the activity log scannable?
- Chart visualization best practices

### Screen 03: Consumer Dashboard (`03-consumer-dashboard.html`)
**Content:**
- Credit balance with BDT equivalent
- API key management (create, copy, revoke)
- Usage analytics: tokens, requests, cost breakdown
- Model selection with VRAM requirements
- Quick Start code snippet

**Issues to check:**
- Is the credit balance prominent enough?
- Is the API key flow intuitive?
- Is the code snippet copy-paste friendly?
- Model selection clarity for non-technical users

### Screen 04: Admin Governance Portal (`04-admin-portal.html`)
**Content:**
- Dashboard overview: nodes, users, revenue, credits
- Active alerts with severity levels
- Node management table with actions
- BDT payment history with status
- Ledger audit trail with block details

**Issues to check:**
- Is the admin workflow efficient?
- Are critical actions (blacklist, ban) safe from accidental clicks?
- Is the ledger audit trail readable?
- Payment status clear at a glance?

---

## 6. REVIEW & REDESIGN RULES

### UI/UX Standards (NON-NEGOTIABLE)

1. **shadcn/ui Component Library** — Use shadcn/ui primitives as the foundation. If a component doesn't exist, extend it properly. No custom reinventions.

2. **Device Agnostic** — Every screen must work at:
   - Desktop: 1920px, 1440px, 1280px
   - Tablet: 1024px, 768px
   - Mobile: 375px ( iPhone SE), 390px (iPhone 14), 414px (iPhone Plus)
   - Never fixed widths. Use responsive grid, fluid typography, container queries.

3. **Enterprise-Grade Quality**:
   - Zero decorative elements — every pixel serves a purpose
   - Consistent spacing system (4px base unit)
   - Accessible color contrast (WCAG 2.1 AA minimum)
   - Keyboard navigation for all interactive elements
   - Focus indicators on all focusable elements
   - Screen reader labels for icons and status indicators

4. **Data Density** — Enterprise users need information density. Avoid excessive white space. Use compact layouts for data-heavy views (tables, lists, metrics).

5. **Status & Feedback**:
   - Every async action must have loading state
   - Every error must have clear recovery path
   - Every success must have confirmation
   - Use toast notifications, not alerts
   - Inline validation for forms

6. **Color Usage**:
   - Never use color alone to convey meaning (add icons or text)
   - Semantic colors: success (green/teal), warning (amber/yellow), error (red), info (blue)
   - High contrast for critical data (latency, errors, scores)

7. **Typography Hierarchy**:
   - Maximum 3 font sizes per screen
   - Clear heading hierarchy (h1 > h2 > h3)
   - Monospace for technical data (hashes, IDs, code)
   - Consistent line heights (1.4 for body, 1.2 for headings)

8. **Accessibility**:
   - ARIA labels on all interactive elements
   - Role attributes for custom components
   - Alt text for all meaningful images
   - Skip navigation links
   - Reduced motion support

### Screen-by-Screen Audit Checklist

For EACH screen, verify:

- [ ] **Information Architecture**: Is content organized logically? Can users find what they need in <3 seconds?
- [ ] **Visual Hierarchy**: Is the most important element most prominent? Is scan path clear?
- [ ] **Data Density**: Is there too much white space? Too cluttered? Right balance?
- [ ] **Interaction Patterns**: Are actions discoverable? Are destructive actions confirmed?
- [ ] **Error States**: Are empty states handled? Error states? Loading states?
- [ ] **Responsive**: Does it work at all breakpoints? Is text readable on mobile?
- [ ] **Accessibility**: Can a screen reader navigate it? Is contrast sufficient?
- [ ] **Consistency**: Do similar elements look similar? Are patterns consistent?
- [ ] **Performance**: Are images optimized? Is the DOM minimal?
- [ ] **Enterprise Grade**: Would a bank/government use this? Is it trustworthy?

### Redesign Deliverables

For each screen, produce:

1. **Audit Report** — Issues found with severity (Critical/High/Medium/Low)
2. **Redesigned Screen** — Complete HTML/CSS/JS implementation
3. **Component Inventory** — shadcn components used, custom components needed
4. **Responsive Breakpoints** — How layout adapts at each breakpoint
5. **Accessibility Report** — ARIA labels, keyboard nav, contrast ratios
6. **Interaction States** — Loading, empty, error, success, edge cases

### Component Library Reference (shadcn/ui)

Use these shadcn components as building blocks:

- **Layout**: Card, Separator, Sheet, Tabs, Accordion
- **Data Display**: Table, Badge, Avatar, Calendar, Progress
- **Forms**: Input, Select, Checkbox, Radio, Switch, Textarea, Button
- **Feedback**: Alert, Toast, Dialog, AlertDialog, Tooltip
- **Navigation**: Sidebar, Breadcrumb, Pagination, Command
- **Charts**: Bar, Line, Area (using Recharts or Victory)

---

## 7. SCREENS TO GENERATE

After auditing the existing 4 screens, generate redesigned versions plus these additional screens:

### Additional Screens Needed

| Screen | Purpose | Key Data |
|--------|---------|----------|
| **05: Onboarding Flow** | First-time setup wizard | Account creation, node registration, first top-up |
| **06: Model Browser** | Browse and select models | Model cards, VRAM requirements, pricing |
| **07: Credit Purchase** | Top-up flow | BDT amount, payment method, confirmation |
| **08: Node Health Monitor** | Detailed node metrics | Real-time charts, alerts,历史 performance |
| **09: Settings** | User preferences | API keys, notifications, language, theme |
| **10: Mobile Dashboard** | Mobile-optimized main view | Simplified stats, quick actions, swipe nav |

### Screen Output Format

Each screen must be:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Swarm-OS — [Screen Name]</title>
  <style>
    /* Inline CSS only — zero external dependencies */
    /* Use CSS custom properties for design tokens */
    /* Mobile-first responsive design */
  </style>
</head>
<body>
  <!-- Semantic HTML5 structure -->
  <!-- ARIA labels on all interactive elements -->
  <!-- All states: loading, empty, error, success, edge -->
</body>
</html>
```

**Constraints:**
- Max 30KB per HTML file
- Zero external dependencies (no CDN, no npm)
- All CSS inline (no external stylesheets)
- Responsive down to 375px
- All interactive states included
- Accessible (WCAG 2.1 AA)

---

## 8. QUALITY GATES

Before approving any screen, verify:

### Visual Quality
- [ ] Consistent color usage across all screens
- [ ] Typography hierarchy is clear
- [ ] Spacing is consistent (4px grid)
- [ ] Icons are meaningful and labeled
- [ ] No visual clutter or noise

### Functional Quality
- [ ] All data fields have proper formatting
- [ ] Numbers are locale-formatted (BD: 1,234 not 1234)
- [ ] Currency shows BDT symbol (৳)
- [ ] Timestamps are relative ("2 min ago") with absolute on hover
- [ ] Status indicators are clear and semantic

### UX Quality
- [ ] Primary action is obvious
- [ ] Destructive actions require confirmation
- [ ] Back navigation is always available
- [ ] Loading states are shown for async operations
- [ ] Error messages are actionable

### Technical Quality
- [ ] HTML is semantic (header, nav, main, section, article)
- [ ] CSS uses custom properties for theming
- [ ] No inline styles for repeated patterns
- [ ] Responsive at all breakpoints
- [ ] Keyboard navigable

---

## 9. OUTPUT FORMAT

Return your review as:

```
# UI/UX Review Report — Swarm-OS

## Executive Summary
[2-3 sentences on overall quality and critical issues]

## Screen-by-Screen Audit

### Screen 01: [Name]
**Rating:** [1-10]
**Critical Issues:**
- [Issue 1]
- [Issue 2]

**Redesigned Screen:**
[Complete HTML]

**Component Inventory:**
- shadcn: [components used]
- Custom: [components needed]

**Responsive Behavior:**
[Description of breakpoint behavior]

**Accessibility:**
[ARIA labels, keyboard nav, contrast]

---

[Repeat for each screen]

## Additional Screens Generated

[Same format as above]

## Global Recommendations
[Cross-cutting improvements]

## Component Library Extensions
[New shadcn components needed for this project]
```

---

## 10. STRICT RULES FOR THE REVIEWER

1. **DO NOT** add external dependencies. All CSS must be inline.
2. **DO NOT** use placeholder text like "Lorem ipsum". Use real data.
3. **DO NOT** skip responsive design. Every screen must work on mobile.
4. **DO NOT** ignore accessibility. WCAG 2.1 AA is mandatory.
5. **DO NOT** use color alone to convey meaning.
6. **DO NOT** create screens without loading/empty/error states.
7. **DO NOT** use fixed pixel widths. Use responsive units.
8. **DO** use shadcn/ui patterns as reference.
9. **DO** test at 375px, 768px, 1024px, 1440px, 1920px.
10. **DO** include all interactive states (hover, focus, active, disabled).
11. **DO** use semantic HTML5 elements.
12. **DO** format numbers and currencies for BD locale.

---

*Prompt Version: 1.0 | Last Updated: 2026-06-27*
*For questions, refer to AGENTS.md and UI_SYSTEM.md*
