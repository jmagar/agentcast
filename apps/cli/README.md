---
title: "apps/cli"
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

# apps/cli

Deployable CLI surface for AgentCast.

Reusable command parsing and command handlers live in `crates/agent-cli`. This directory is reserved for packaging, wrapper binaries, completion assets, and distribution-specific files if the workspace moves app entrypoints out of `crates/agent-server`.
