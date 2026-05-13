---
title: "Errors"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/claude-code/markdown/0065-code-claude-com-docs-en-errors.md"
  - "docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
  - "docs/references/mcporter/docs/livetests.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Errors

AgentCast errors must be normalized early so CLI, API, MCP, and UI surfaces behave consistently.

These error kinds are AgentCast domain errors. They do not replace upstream JSON-RPC error codes on raw MCP/ACP transports; adapters should preserve upstream error data in redacted structured details when useful.

## Error Envelope

Canonical JSON shape:

```json
{
  "error": {
    "kind": "mcp.initialize_timeout",
    "message": "MCP server `filesystem` did not initialize before timeout",
    "details": {
      "server_id": "filesystem",
      "timeout_ms": 10000
    }
  }
}
```

Rules:

- `kind` is stable and machine-readable.
- `message` is human-readable and specific enough to suggest the next useful action.
- `details` is structured and redacted.
- `details` includes relevant identifiers, attempted operation, and retry/continuation hints when available.
- raw upstream errors may be included only under a clearly named redacted field.

## Informative Error Standard

Errors are an agent-facing API. A good error should answer:

- what failed.
- which server/action/resource/config entry was involved.
- whether the operation is retryable.
- what the caller can do next.
- whether output was truncated, blocked by confirmation, blocked by missing env, or blocked by unsupported schema.

Avoid generic messages like `failed`, `bad request`, or `internal error` when more context is available.

## Error Families

Initial families:

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

## Initial Error Kinds

Config:

- `config.not_found`
- `config.invalid`
- `config.write_failed`

Runtime:

- `runtime.spawn_failed`
- `runtime.process_exited`
- `runtime.timeout`
- `runtime.cancelled`

MCP:

- `mcp.initialize_failed`
- `mcp.initialize_timeout`
- `mcp.discovery_failed`
- `mcp.invalid_cursor`
- `mcp.tool_not_found`
- `mcp.tool_failed`
- `mcp.resource_not_found`
- `mcp.resource_failed`
- `mcp.protocol_error`

Launcher:

- `launcher.action_not_found`
- `launcher.collision`
- `launcher.confirmation_required`

Registry/install:

- `registry.fetch_failed`
- `registry.decode_failed`
- `registry.entry_not_found`
- `install_plan.conflict`
- `install_plan.apply_failed`
- `install_plan.verify_failed`

Validation/security:

- `validation.input`
- `validation.schema`
- `security.confirmation_required`
- `security.confirmation_denied`

Internal:

- `internal.bug`
- `internal.unsupported`

## CLI Mapping

CLI exit codes are defined in `docs/CLI.md`. The CLI maps error families to those exit codes without changing the error envelope under `--json`.

## Upstream References Checked

- `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`: JSON-RPC request, notification, cancellation, and schema terminology.
- `docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md`: invalid cursor maps to JSON-RPC invalid params upstream.
- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`: tool result and tool error shape.
- `docs/references/claude-code/markdown/0065-code-claude-com-docs-en-errors.md`: external error UX comparison.
- `docs/references/mcporter/docs/livetests.md`: live transport checks are gated and classify hosted-server failures explicitly.
