---
title: "Specs"
doc_type: "spec"
status: "active"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs:
  - "docs/references/acp/docs/manifest.jsonl"
  - "docs/references/claude-code/manifest.jsonl"
  - "docs/references/fastmcp/docs/overview.mdx"
  - "docs/references/mcp/docs/manifest.jsonl"
  - "docs/references/mcporter/docs/cli-reference.md"
related:
  - "docs/specs/api.md"
  - "docs/specs/app-surfaces.md"
  - "docs/specs/cli.md"
  - "docs/specs/config.md"
  - "docs/specs/crates-and-dependencies.md"
  - "docs/specs/errors.md"
  - "docs/specs/install-plans.md"
  - "docs/specs/launcher-actions.md"
  - "docs/specs/mcp-runtime.md"
  - "docs/specs/registry.md"
  - "docs/specs/schema-ux.md"
  - "docs/specs/security.md"
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Specs

Specs describe how AgentCast should implement current contracts.

Current MVP specs:

- [launcher-actions.md](./launcher-actions.md)
- [config.md](./config.md)
- [mcp-runtime.md](./mcp-runtime.md)
- [cli.md](./cli.md)
- [registry.md](./registry.md)
- [install-plans.md](./install-plans.md)
- [security.md](./security.md)
- [schema-ux.md](./schema-ux.md)
- [errors.md](./errors.md)
- [api.md](./api.md)
- [app-surfaces.md](./app-surfaces.md)
- [crates-and-dependencies.md](./crates-and-dependencies.md)

If a spec contradicts a contract, revise the contract explicitly or treat the contract as authoritative.

## Upstream References

This spec index is grounded by the same upstream corpus as the linked spec files:

- `docs/references/mcp/docs/manifest.jsonl`
- `docs/references/claude-code/manifest.jsonl`
- `docs/references/acp/docs/manifest.jsonl`
- `docs/references/fastmcp/docs/overview.mdx`
- `docs/references/mcporter/docs/cli-reference.md`
