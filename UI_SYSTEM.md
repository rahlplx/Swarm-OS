# UI_SYSTEM.md — Swarm-OS Design Tokens

## 🎨 Visual Identity
**Theme:** Cyber-Industrial / High-Confidence Utility.
**Vibe:** "Professional tool for the edge."

## 🌈 Color Palette
| Token | Hex | Usage | Vibe |
|---|---|---|---|
| `bg-deep-slate` | `#0F0F11` | Main Background | Depth/Stability |
| `accent-cyber-amber` | `#FFB800` | Active Computation / GPU Load | Energy/Action |
| `accent-warning-rose` | `#FF4D4D` | Latency Spike / Churn Risk | Alert/Danger |
| `accent-success-teal` | `#00C4B4` | Ledger Finalized / Sync OK | Trust/Success |
| `surface-obsidian` | `#1A1A1F` | Component Backgrounds | Contrast |

## 📐 Design Components
- **Confidence Metrics:** Every node action must show a "Confidence Score" (0-100%) based on latency and reliability.
- **The Swarm Map:** An interactive force-directed graph of nodes.
- **Credit Co-op Dashboard:** Vertical timeline of Merkle-DAG hashes with BDT balance.

## ⚡ UX Principles
- **Latency-Awareness:** UI should visually pulse when network latency increases.
- **Confidence-First:** Don't just show "Connected"; show "Low-Jitter Connection."
- **Minimalist Edge:** No bloated animations; prioritize raw telemetry.