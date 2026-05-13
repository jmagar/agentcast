---
title: "AgentCast Apps"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
  - "implementers"
scope: "v0"
source_of_truth: false
upstream_refs: []
related:
  - "apps/api/README.md"
  - "apps/cli/README.md"
  - "apps/desktop/README.md"
  - "apps/mcp/README.md"
  - "apps/web/README.md"
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# AgentCast Apps

`apps/` is for deployable entrypoints. Reusable logic belongs in `crates/`.

Current planned app surfaces:

- [cli](./cli/README.md) - operator CLI packaging/wrapper surface.
- [mcp](./mcp/README.md) - AgentCast MCP server entrypoint.
- [api](./api/README.md) - HTTP API app entrypoint.
- [web](./web/README.md) - future web UI.
- [desktop](./desktop/README.md) - future desktop app surface.

Do not move domain behavior into apps. Apps compose crates.
