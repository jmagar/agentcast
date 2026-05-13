---
title: "apps/mcp"
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

# apps/mcp

Deployable AgentCast MCP server surface.

Reusable MCP protocol code lives in `crates/agent-mcp`; gateway policy lives in `crates/agent-gateway`; runtime ownership lives in `crates/agent-runtime`. This directory is reserved for the app entrypoint and smoke/deployment assets.
