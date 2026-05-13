---
title: "apps/desktop"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
  - "implementers"
scope: "v0"
source_of_truth: false
upstream_refs: []
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# apps/desktop

Future AgentCast desktop app surface.

Desktop-specific shell code belongs here. Shared client DTOs come from `crates/agent-ui-contracts`.

Reusable native behavior should move into a Rust crate only when it becomes substantial enough to test independently, such as tray integration, global hotkeys, deep links, updater integration, or OS keychain bridging.
