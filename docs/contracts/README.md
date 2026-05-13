---
title: "Contracts"
doc_type: "contract"
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
  - "docs/contracts/api.md"
  - "docs/contracts/cli.md"
  - "docs/contracts/config.md"
  - "docs/contracts/errors.md"
  - "docs/contracts/install-plans.md"
  - "docs/contracts/launcher-actions.md"
  - "docs/contracts/mcp-runtime.md"
  - "docs/contracts/registry.md"
  - "docs/contracts/schema-ux.md"
  - "docs/contracts/security.md"
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Contracts

Contracts are stable behavioral requirements. They should be externally testable and concise.

Current MVP contracts:

- [launcher-actions.md](./launcher-actions.md)
- [config.md](./config.md)
- [mcp-runtime.md](./mcp-runtime.md)
- [cli.md](./cli.md)
- [registry.md](./registry.md)
- [install-plans.md](./install-plans.md)
- [api.md](./api.md)
- [security.md](./security.md)
- [schema-ux.md](./schema-ux.md)
- [errors.md](./errors.md)

When a contract changes, update related specs and top-level docs in the same patch when practical.

## Upstream References

This contract index is grounded by the same upstream corpus as the linked contract files:

- `docs/references/mcp/docs/manifest.jsonl`
- `docs/references/claude-code/manifest.jsonl`
- `docs/references/acp/docs/manifest.jsonl`
- `docs/references/fastmcp/docs/overview.mdx`
- `docs/references/mcporter/docs/cli-reference.md`
