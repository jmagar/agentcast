---
title: "apps/api"
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

# apps/api

Deployable HTTP API surface for AgentCast.

Reusable Axum routes live in `crates/agent-api`; shared client DTOs live in `crates/agent-ui-contracts`. This directory is reserved for API app entrypoint concerns, deployment config, and generated API publication assets.
