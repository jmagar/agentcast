---
title: "Schema UX Spec"
doc_type: "spec"
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
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Schema UX Spec

## Owning Crates

- `agent-protocol`: stores raw and normalized schema metadata.
- `agent-schema`: normalizes JSON Schema and validates invocation payloads.
- `agent-cli`: maps simple schemas to CLI args.
- `agent-api`: validates JSON requests.
- `apps/web`: future form rendering.

## Normalization

Schema normalization should produce:

- root type.
- required fields.
- property list.
- scalar type for simple fields.
- enum values.
- defaults.
- descriptions.
- unsupported feature flags.

The raw schema remains available in metadata.

## CLI Coercion

`--arg key=value` coercion:

- string: unchanged.
- boolean: `true`, `false`, `1`, `0`.
- integer: base-10 integer.
- number: JSON number parse.
- enum: string value must match.
- array: comma-separated only for scalar arrays when schema is simple.

Complex schemas require `--json-args`.

This coercion subset is an AgentCast local CLI UX decision. Raw MCP `inputSchema` metadata remains available so unsupported JSON Schema features are not erased.

## Validation

Validation order:

1. parse JSON.
2. ensure object root.
3. reject unknown fields unless allowed.
4. check required fields.
5. coerce CLI scalar values.
6. validate enum/type constraints.
7. pass validated JSON object to runtime.

## Verification

Run:

```bash
cargo test -p agent-cli schema
cargo test -p agent-schema
cargo test -p agent-protocol schema
```

## Upstream References

- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`
- `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`
- `docs/references/fastmcp/docs/client.mdx`
- `docs/references/fastmcp/docs/generate-cli.mdx`
