# Swarm-OS Design System

## Visual Identity

**Theme:** Cyber-Industrial / High-Confidence Utility
**Vibe:** Professional tool for the edge. Dark, data-dense, enterprise-grade. Every pixel serves a purpose.

---

## Color Palette

### Background
- **Deep Slate** `#0B1120` — Primary background. Deep, stable, reduces eye strain during long sessions.
- **Surface** `#111827` — Elevated surfaces, cards, panels.
- **Surface Hover** `#1F2937` — Hover states, interactive surface highlights.
- **Surface Muted** `#374151` — Disabled states, secondary surfaces.

### Primary
- **Cyber Blue** `#3B82F6` — Primary actions, links, active states. Trust and clarity.
- **Cyber Blue Light** `#60A5FA` — Hover states, secondary emphasis.
- **Cyber Blue Dark** `#2563EB` — Pressed states.

### Accent
- **Amber** `#F59E0B` — Active computation, GPU load, energy, action. The signature Swarm-OS color.
- **Amber Light** `#FBBF24` — Hover on amber elements.

### Success
- **Teal** `#10B981` — Ledger finalized, sync OK, positive outcomes, trust.

### Warning
- **Orange** `#F97316` — Caution, degradation, non-critical alerts.

### Error / Danger
- **Rose** `#EF4444` — Latency spikes, node failure, critical alerts, destructive actions.

### Info
- **Sky** `#0EA5E9` — Informational messages, tips, neutral indicators.

### Text
- **Primary** `#F9FAFB` — Headlines, primary data. Maximum readability.
- **Secondary** `#9CA3AF` — Labels, descriptions, secondary data.
- **Tertiary** `#6B7280` — Placeholders, hints, muted text.
- **Inverse** `#111827` — Text on light backgrounds (badges, pills).

### Borders
- **Default** `#374151` — Card borders, separators.
- **Strong** `#4B5563` — Emphasized borders, focus rings.
- **Focus** `#3B82F6` — Keyboard focus indicators.

---

## Typography

### Font Family
- **Primary:** Inter, system-ui, -apple-system, sans-serif
- **Monospace:** JetBrains Mono, Fira Code, monospace (for hashes, node IDs, code, technical data)

### Scale
| Level | Size | Weight | Line Height | Use |
|-------|------|--------|-------------|-----|
| Display | 36px | 700 | 1.1 | Hero stats, large numbers |
| H1 | 24px | 600 | 1.2 | Page titles |
| H2 | 20px | 600 | 1.3 | Section headings |
| H3 | 16px | 600 | 1.4 | Card titles, subsections |
| Body | 14px | 400 | 1.5 | Default text, descriptions |
| Small | 12px | 400 | 1.4 | Labels, captions, timestamps |
| Micro | 11px | 500 | 1.3 | Badges, tags, fine print |

### Monospace Usage
- Node IDs: `swrm_node_9f2a...`
- Ledger hashes: `0x7a3f...e91b`
- Code snippets, API keys, technical metrics

---

## Spacing & Layout

### Spacing Scale (4px base)
| Token | Value | Use |
|-------|-------|-----|
| xs | 4px | Tight gaps, icon padding |
| sm | 8px | Compact gaps, inline elements |
| md | 12px | Default gaps |
| lg | 16px | Card padding, section gaps |
| xl | 24px | Page padding, major sections |
| 2xl | 32px | Page margins |

### Border Radius
| Token | Value | Use |
|-------|-------|-----|
| none | 0 | Sharp edges, data tables |
| sm | 6px | Buttons, inputs, small cards |
| md | 8px | Cards, panels |
| lg | 12px | Modals, large cards |
| xl | 16px | Feature cards |
| full | 9999px | Pills, badges, avatars |

### Grid
- **Sidebar:** 240px fixed (collapsible to 64px icon-only)
- **Top Bar:** 64px height, fixed
- **Content:** Fluid, max-width 1400px, centered
- **Card Grid:** 24px gaps, responsive columns
- **Table Row Height:** 48px (compact), 56px (default)

---

## Component Patterns

### Card
- Background: Surface `#111827`
- Border: 1px solid `#374151`
- Border Radius: `md` (8px)
- Padding: `lg` (16px)
- Shadow: none (flat, data-dense)
- Hover: border-color shifts to `#4B5563`

### Button
- **Primary:** Background `#3B82F6`, text white, radius `sm` (6px), padding 8px 16px
- **Secondary:** Background transparent, border `#3B82F6`, text `#3B82F6`
- **Ghost:** Background transparent, text `#9CA3AF`, hover text white
- **Danger:** Background `#EF4444`, text white
- **Sizes:** sm (28px), md (36px), lg (44px) height

### Badge / Pill
- Background: color-mix at 20% opacity of semantic color
- Text: semantic color
- Border-radius: `full` (pill)
- Padding: 2px 8px
- Font: 11px, weight 500

### Input
- Background: `#0B1120`
- Border: 1px solid `#374151`
- Border Radius: `sm` (6px)
- Padding: 8px 12px
- Focus: border-color `#3B82F6`, ring 2px at 20% opacity
- Error: border-color `#EF4444`

### Table
- Header: background `#0B1120`, text `#9CA3AF`, font 11px uppercase
- Row: border-bottom 1px solid `#374151`
- Row hover: background `#1F2937`
- Cell padding: 12px 16px

### Sidebar Navigation
- Background: `#0B1120`
- Width: 240px (64px collapsed)
- Item height: 40px
- Active: background `#1F2937`, text `#3B82F6`, left border 3px solid `#3B82F6`
- Hover: background `#1F2937`
- Section labels: 11px uppercase, `#6B7280`, 16px left padding

### Top Bar
- Height: 64px
- Background: `#111827`
- Border-bottom: 1px solid `#374151`
- Logo left, nav center, actions right

---

## Data Visualization

### Chart Colors
- Primary line: `#3B82F6`
- Secondary line: `#10B981`
- Warning line: `#F59E0B`
- Error line: `#EF4444`
- Fill at 10% opacity of stroke color

### Status Indicators
- Online/Active: `#10B981` (teal) with pulsing dot animation
- Idle: `#F59E0B` (amber)
- Error/Offline: `#EF4444` (rose)
- Processing: `#3B82F6` (blue) with spin animation

### Score Visualization
- 0-40: `#EF4444` (rose) — Critical
- 40-70: `#F59E0B` (amber) — Degraded
- 70-90: `#10B981` (teal) — Healthy
- 90-100: `#3B82F6` (blue) — Excellent

---

## Animation & Motion

### Transitions
- Default: 150ms ease-in-out
- Slow: 300ms ease-in-out
- Spring: 200ms cubic-bezier(0.34, 1.56, 0.64, 1)

### Micro-interactions
- Button hover: scale 1.02, shadow lift
- Card hover: border-color transition
- Status pulse: 2s infinite ease-in-out
- Loading spinner: 1s linear infinite

### Reduced Motion
- All animations respect `prefers-reduced-motion`
- Skeleton loaders use opacity fade, not slide

---

## Accessibility

### Focus
- 2px solid `#3B82F6` with 2px offset
- Visible on all interactive elements
- Skip-to-content link at top

### Contrast
- Primary text on dark: 15.4:1 ratio (AAA)
- Secondary text on dark: 5.7:1 ratio (AA)
- Semantic colors: all meet AA on dark backgrounds

### Screen Reader
- All icons have aria-labels
- Status indicators include text (not just color)
- Tables use proper th scope
- Live regions for real-time updates

---

## Responsive Breakpoints

| Breakpoint | Width | Layout |
|------------|-------|--------|
| Mobile | < 768px | Single column, bottom nav, stacked cards |
| Tablet | 768px - 1024px | 2-column grid, collapsed sidebar |
| Desktop | 1024px - 1440px | Full sidebar, 3-column grid |
| Wide | > 1440px | Max-width container, 4-column grid |

---

## BD-Specific Patterns

- Currency: BDT symbol ৳ with comma-separated thousands (৳18,420)
- Timestamps: Relative ("2 min ago") with absolute on hover
- Status text: English primary, Bangla toggle available
- Payment methods: bKash (pink), Nagad (orange), Visa/Mastercard (blue/gray)
