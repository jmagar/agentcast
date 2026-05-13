---
title: "Error Contract"
doc_type: "contract"
status: "active"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs:
  - "docs/references/claude-code/markdown/0083-code-claude-com-docs-en-permissions.md"
  - "docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
  - "docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md"
  - "docs/references/mcporter/docs/cli-reference.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Error Contract

This contract defines normalized AgentCast errors.

## Envelope

Machine-facing errors must use:

```json
{
  "error": {
    "kind": "family.code",
    "message": "Human readable message",
    "details": {}
  }
}
```

## Requirements

- `kind` is stable and machine-readable.
- `message` is human-readable and specific.
- `details` is structured.
- `details` must be redacted.
- redacted values use the literal string `"[REDACTED]"`.
- surfaces may add transport metadata outside `error`, but must not change `kind`.
- details should include relevant IDs, retryability, and next-action hints when available.

## Required Families

- `config.*`
- `runtime.*`
- `mcp.*`
- `launcher.*`
- `registry.*`
- `install_plan.*`
- `validation.*`
- `security.*`
- `api.*`
- `internal.*`

## Acceptance Tests

Implementations must test:

- each crate maps its public errors to a normalized kind.
- CLI `--json` errors match the envelope.
- API errors match the envelope.
- secret fields are redacted in details.
- messages contain enough context to identify the failed operation.

## Upstream References

- `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`
- `docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md`
- `docs/references/claude-code/markdown/0083-code-claude-com-docs-en-permissions.md`
- `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`
- `docs/references/mcporter/docs/cli-reference.md`
