---
title: "API Contract"
doc_type: "contract"
status: "active"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs:
  - "docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md"
  - "docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
  - "docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md"
  - "docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# API Contract

This contract defines the initial HTTP API surface for AgentCast.

## Scope

The API is a thin surface over validated AgentCast runtime, gateway, registry, and install-plan behavior. It must not own independent MCP protocol logic or alternate DTOs.

## Versioning

Requirements:

- v0 routes live under `/v1`.
- response bodies use shared DTOs from AgentCast contract crates.
- route paths that include action IDs must accept percent-encoded IDs.
- route paths that include MCP resource URIs must accept percent-encoded URIs.
- breaking response changes require a contract update.

## Initial Routes

Required routes:

- `GET /health`
- `GET /v1/servers`
- `GET /v1/servers/{server_id}`
- `GET /v1/mcp/servers`
- `GET /v1/mcp/servers/{server_id}/tools`
- `GET /v1/mcp/servers/{server_id}/resources`
- `GET /v1/mcp/servers/{server_id}/resources/{uri}`
- `GET /v1/mcp/servers/{server_id}/prompts`
- `POST /v1/mcp/servers/{server_id}/tools/{tool_name}/call`
- `GET /v1/actions`
- `GET /v1/actions/{action_id}`
- `POST /v1/call`
- `GET /v1/registry/search`
- `POST /v1/install/preview`
- `POST /v1/install/apply`

`POST /v1/call` must accept the action ID in the JSON body so clients do not need to rely on path encoding for invocation.

Collection routes must support pagination and filtering. Filtering applies before pagination. Collection responses must not be unbounded by default and must expose a stable continuation field when more results exist.

## Errors

Requirements:

- error responses use the normalized error envelope from `docs/contracts/errors.md`.
- HTTP status codes reflect transport-level outcome, while `error.kind` reflects AgentCast behavior.
- secret-bearing fields are redacted before serialization.

## Security

Requirements:

- local-only unauthenticated API is allowed only for explicitly configured loopback development mode.
- non-loopback binds require auth configuration.
- destructive calls require the same confirmation semantics as CLI.
- API must not expose `.env` values.

## Acceptance Tests

Implementations must test:

- route DTO compatibility with shared contract structs.
- percent-encoded action ID lookup.
- percent-encoded MCP resource URI lookup.
- pagination and filtering on collection routes.
- JSON error envelope shape.
- redaction before response serialization.
- non-loopback auth guard.

## Upstream References

- `docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md`
- `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`
- `docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md`
- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`
- `docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md`
- `docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md`
- `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`
