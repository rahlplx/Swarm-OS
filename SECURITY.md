---
type: security
title: Swarm-OS Security Policy & Vulnerability Disposition
description: Security policy, vulnerability reporting, and accepted-risk dispositions
tags: [security, governance]
timestamp: "2026-06-23"
status: active
phase: "0-4"
---

# Swarm-OS Security Policy

## Reporting a Vulnerability

Report security vulnerabilities privately to `security@swarm-os.dev`. Do not open a public GitHub issue for security vulnerabilities. We will acknowledge receipt within 48 hours and provide an initial assessment within 7 days.

## Accepted-Risk Dispositions

### Dependabot Alert #1: glib unsoundness in VariantStrIter (moderate)

- **Advisory:** Unsoundness in `Iterator` and `DoubleEndedIterator` impls for `glib::VariantStrIter`
- **Vulnerable versions:** glib >= 0.15.0, < 0.20.0
- **Fixed in:** glib 0.20.0
- **Our version:** glib 0.18.5 (pinned by Tauri v2 → gtk v0.18 → glib ^0.18)
- **Status:** Accepted risk — not exploitable in Swarm-OS codebase
- **Disposition date:** 2026-06-23

**Rationale:** The vulnerable API (`glib::VariantStrIter`) is used for iterating over `GVariant` DBus values. Swarm-OS does not use `GVariant` or DBus directly — the glib dependency is transitive through Tauri's GTK-based tray icon and window management (`gtk → atk → glib`). No code in `src-tauri/src/`, `node-agent/src/`, or any other Swarm-OS crate calls `VariantStrIter` or any `GVariant` API. The vulnerability requires an attacker to control input to a `VariantStrIter` construction, which does not happen in our code paths.

**Remediation path:** Upgrade to a future Tauri version that depends on gtk v0.20+ (which uses glib 0.20+). This is a breaking change that requires testing the Tauri v2 → v3 migration path. Deferred to Phase 1 or when Tauri publishes a fix.

**Monitoring:** Re-evaluate when Tauri releases a version with glib >= 0.20.0. Check `cargo tree -i glib` after Tauri upgrades.
