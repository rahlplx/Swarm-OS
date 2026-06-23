---
type: index
title: Swarm-OS Planning Bundle
description: Decentralized P2P AI inference network — planning documentation index
tags: [architecture, planning]
timestamp: "2026-06-22"
---

# Swarm-OS Planning Bundle

Swarm-OS is a decentralized P2P AI inference network where contributors pool idle GPU/CPU compute into a WireGuard mesh and consumers run 7B-405B models via an OpenAI-compatible API. A tamper-evident credit ledger tracks earn/spend. Apache 2.0.

## Document Map

| File | Type | Authority For | Phase | ~Tokens | Description |
|------|------|---------------|-------|---------|-------------|
| [CLAUDE.md](/CLAUDE) | context | ai_onboarding | 0-4 | 1,900 | AI assistant context for Claude Code (canonical onboarding doc) |
| [AGENTS.md](/AGENTS) | context | ai_onboarding_cross_tool | 0-4 | 1,200 | Cross-tool agent onboarding (Cursor, Aider, etc.); defers to CLAUDE.md for Claude Code |
| [.claude/memory/PROJECT.md](/.claude/memory/PROJECT) | memory | project_state, decisions, invariants, progress | 0-4 | 1,500 | Persistent cross-session memory — read FIRST every session |
| [.claude/rules/INVARIANTS.md](/.claude/rules/INVARIANTS) | rules | security_invariants, ledger_invariants, license_invariants | 0-4 | 1,800 | Hard rules extracted from governance + architecture §7 + tech_stack — never violate |
| [.claude/rules/SESSION_PROTOCOL.md](/.claude/rules/SESSION_PROTOCOL) | rules | session_bootstrap, operating_protocol, auto_trigger | 0-4 | 1,200 | Session startup checklist + cross-session auto-trigger config |
| [project.md](/project) | spec | product_identity, feature_list, bd_market_context, success_metrics | 0-4 | 3,000 | Product identity, features F1-F9, phase roadmap, BD market |
| [architecture.md](/architecture) | spec | system_layers, scheduler_algorithm, pipeline_ring_topology, activation_tensor_sizes, security_model, failure_modes | 0-4 | 5,600 | System diagram, Blackboard, scheduler/router, sharding, security |
| [tech_stack.md](/tech_stack) | reference | oss_dependencies, dependency_map, model_support_matrix, build_toolchain, license_constraints | 0-4 | 4,300 | OSS deps with licenses, dependency map, build toolchain |
| [governance.md](/governance) | spec | role_hierarchy, config_schemas, credit_formula, ledger_format, ledger_audit_protocol, abuse_prevention, hardware_fingerprint | 1-4 | 4,900 | Roles, admin portal, config schemas, ledger audit, abuse prevention |
| [ui_ux.md](/ui_ux) | spec | design_system, web_portal_screens, tauri_agent_screens, grafana_dashboards, accessibility_rules | 0-4 | 7,500 | Design system, screen wireframes, accessibility |
| [critique.md](/critique) | review | critique_issues, bd_network_reality, competitive_analysis | 0-4 | 5,700 | 28 issues severity-ranked with action plan |
| [guide.md](/guide) | guide | phase_0_day_by_day, non_technical_explanations | 0 | 10,000 | Non-technical ebook-style pre-Phase 0 + Phase 0 guide |
| [research.md](/research) | reference | benchmark_data, api_shapes, code_samples, oss_verdicts | 0-4 | 17,000 | Component research: benchmarks, code samples, verdicts |
| [verify-prompt.md](/verify-prompt) | prompt | verification_prompt | pre-0 | 4,200 | Community verification prompt for architecture decisions |
| [SECURITY.md](/SECURITY) | security | security_policy, vulnerability_disposition | 0-4 | 800 | Security policy + accepted-risk dispositions for dependabot alerts |

## Reading Paths

**AI Agent — Full Context** (dependency-sorted):
architecture.md → governance.md → project.md → tech_stack.md → critique.md → ui_ux.md → guide.md → research.md

**Phase 0 Only:**
project.md → architecture.md (§1-5, §7) → tech_stack.md (Tier 1-2) → ui_ux.md (Surface 1-2) → guide.md (Part 2)

**Security Review:**
architecture.md (§7) → governance.md (§1, §5-6) → critique.md (Domain 2)

## Authority Map

When information appears in multiple files, this table identifies the single canonical source.

| Concept | Authoritative Source | Also Referenced In |
|---------|---------------------|--------------------|
| Scheduler scoring formula | architecture.md §3 | project.md F1, governance.md §3.1, CLAUDE.md |
| Credit earn/spend formula | governance.md §3.4 | project.md F6, architecture.md §5, CLAUDE.md, README.md |
| API key hashing (Argon2id) | architecture.md §7 | governance.md §1, critique.md Domain 2, CLAUDE.md |
| Ledger entry format | governance.md §4.1 | architecture.md §2, critique.md Domain 1 |
| Key prefixes (swrm_*) | governance.md §1 | architecture.md §7, CLAUDE.md |
| Hardware fingerprint | governance.md §5.3 | CLAUDE.md |
| Activation tensor sizes | architecture.md §4 | tech_stack.md Tier 0, critique.md Domain 1 |
| Pipeline ring topology | architecture.md §4 | tech_stack.md Tier 1, project.md F4 |
| Node churn handling | architecture.md §2 | critique.md Domain 1 |
| OSS dependency verdicts | tech_stack.md | research.md, CLAUDE.md |
| Phase 0 day-by-day roadmap | guide.md Ch.9 | project.md Phase 0 checklist |
| Design system (colors, fonts) | ui_ux.md §1 | CLAUDE.md |
| BD market context | project.md | critique.md Domain 4 |
| Failure mode matrix | architecture.md §8 | — |
