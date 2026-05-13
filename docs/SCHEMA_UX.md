---
title: "Schema UX"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/fastmcp/docs/generate-cli.mdx"
  - "docs/references/mcp/docs/markdown/0021-modelcontextprotocol-io-seps-1330-elicitation-enum-schema-improvements-and-stand.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
  - "docs/references/seed/chatgpt-windows-raycast-extensions.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Schema UX

AgentCast uses MCP tool JSON Schema to drive CLI arguments now and future UI forms later.

Schema-driven UX is the extension model. AgentCast should generate deterministic interfaces from schemas instead of requiring users to write custom extension UI code.

## Inputs

The source schema is the MCP tool `inputSchema`.

In MCP 2025-11-25, tool `inputSchema` is a JSON Schema object with an object root. If `$schema` is absent, upstream defaults embedded schemas to JSON Schema 2020-12.

Rules:

- schema is untrusted input.
- validation happens before invocation.
- raw schema is preserved in launcher action metadata.
- unsupported schema shapes must fall back to JSON input rather than guessing.

## Supported MVP Shapes

MVP supports:

- object root schemas.
- string fields.
- number/integer fields.
- boolean fields.
- enum fields.
- arrays of scalar values.
- required fields.
- defaults.
- descriptions.

MVP may also display simple titled enum forms when they are easy to detect, but complex `oneOf`/`anyOf` forms must still have a JSON fallback.

MVP may display but does not need ergonomic controls for:

- deeply nested objects.
- `oneOf`, `anyOf`, `allOf`.
- conditional schemas.
- recursive schemas.
- binary/file upload schemas.

## CLI Mapping

For simple object schemas:

```txt
agentcast call <action-id> --arg path=/tmp --arg recursive=true
```

For complex schemas:

```txt
agentcast call <action-id> --json-args '{"path":"/tmp","recursive":true}'
```

Rules:

- scalar `--arg` values are coerced according to schema.
- enum values must match one allowed value.
- missing required fields fail before MCP invocation.
- unknown fields fail unless schema allows additional properties.

## Future Form Mapping

Future UI forms should map:

- string -> text input.
- boolean -> checkbox/toggle.
- enum -> select/segmented control.
- number/integer -> numeric input.
- arrays -> repeatable field.
- object -> nested section when shallow.

Unsupported forms must show a JSON editor fallback.

## Extension Boundary

AgentCast should avoid arbitrary custom frontend extension APIs in v0 and early post-v0 work.

Rules:

- MCP metadata and JSON Schema drive generated CLI/API/MCP/UI surfaces.
- custom rendering is not required for an MCP server to feel usable.
- unsupported schema or result shapes fall back to JSON editor/tree/markdown/code views.
- arbitrary frontend code is deferred unless a concrete security and consistency model exists.

## Validation Errors

Validation errors use `validation.schema` or `validation.input` error kinds from `docs/ERRORS.md`.

## Upstream References Checked

- `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`: `Tool.inputSchema`, `Tool.outputSchema`, object-root schema, and JSON Schema 2020-12 default.
- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`: tool schema and no-parameter schema guidance.
- `docs/references/mcp/docs/markdown/0021-modelcontextprotocol-io-seps-1330-elicitation-enum-schema-improvements-and-stand.md`: enum schema design context; SEP material only.
- `docs/references/fastmcp/docs/generate-cli.mdx`: schema-generated CLI precedent.
- `docs/references/seed/chatgpt-windows-raycast-extensions.md`: product-origin decision to generate extension UX from MCP schemas instead of custom extension UI code.
