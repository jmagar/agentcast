---
title: "apps/web"
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

# apps/web

Future AgentCast web UI.

The web app must consume API/runtime contracts and shared DTOs from `crates/agent-ui-contracts`. It must not implement MCP protocol behavior directly.

UI work should follow `../../docs/SCHEMA_UX.md`, `../../docs/LAUNCHER.md`, and the relevant contracts/specs.
