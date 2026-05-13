---
title: "Schema UX Contract"
doc_type: "contract"
status: "active"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs:
  - "docs/references/fastmcp/docs/client.mdx"
  - "docs/references/fastmcp/docs/generate-cli.mdx"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Schema UX Contract

This contract defines JSON Schema behavior for CLI inputs and future forms.

## Supported MVP Schema

MVP must support:

- object root schemas.
- string fields.
- number and integer fields.
- boolean fields.
- enum fields.
- arrays of scalar fields.
- required fields.
- defaults.
- descriptions.

## CLI Input Rules

Requirements:

- `--json-args` accepts a JSON object.
- `--arg key=value` is available only for simple scalar fields.
- schema metadata can generate deterministic CLI/API/MCP/UI inputs.
- required fields are validated before invocation.
- enum values must match schema.
- unknown fields fail unless schema allows additional properties.
- unsupported schemas fall back to `--json-args`.
- custom frontend extension rendering is not required for v0 schema support.

The MVP schema subset is an AgentCast local implementation limit over MCP tool `inputSchema`; MCP itself uses JSON Schema and defaults tool schemas to JSON Schema 2020-12 when `$schema` is absent.

## Error Rules

Schema and input failures use:

- `validation.schema`
- `validation.input`

## Acceptance Tests

Implementations must test:

- scalar coercion.
- missing required fields.
- enum mismatch.
- unknown fields.
- nested-object fallback to JSON input.

## Upstream References

- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`
- `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`
- `docs/references/fastmcp/docs/client.mdx`
- `docs/references/fastmcp/docs/generate-cli.mdx`
