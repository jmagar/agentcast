---
title: "API Spec"
doc_type: "spec"
status: "active"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs:
  - "docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md"
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
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# API Spec

This spec describes the initial HTTP API implementation shape.

## Owning Crates

- `agent-api`: Axum router, handlers, extractors, and HTTP error mapping.
- `agent-server`: runtime assembly and process lifecycle for serving API and MCP surfaces.
- `agent-protocol`: shared DTOs.
- `agent-auth`: auth and local bind policy.

## Module Shape

Initial `agent-api` modules:

- `lib.rs`: router construction exports.
- `error.rs`: HTTP mapping for normalized AgentCast errors.
- `state.rs`: shared application state.
- `routes/health.rs`: health checks.
- `routes/servers.rs`: server listing and detail routes.
- `routes/mcp.rs`: MCP server, tool, resource, and direct call routes.
- `routes/actions.rs`: action listing and detail routes.
- `routes/call.rs`: invocation route.
- `routes/registry.rs`: registry search route.
- `routes/install.rs`: install preview and apply routes.

## State Boundary

`agent-api` state should hold trait-backed handles for:

- gateway action catalog and invocation.
- runtime server status.
- registry search.
- install-plan preview/apply.
- auth context.

Handlers should not parse `config.toml`, spawn MCP processes directly, or normalize registry DTOs directly.

## Route Behavior

Collection routes accept pagination and filtering query parameters:

```txt
limit=<n>
cursor=<cursor>
filter=<expr>
server_id=<server-id>
```

Handlers pass these through to runtime/search APIs. Filtering applies before pagination.

`GET /v1/actions/{action_id}` decodes the path segment as a percent-encoded `LauncherActionId`.

`GET /v1/mcp/servers/{server_id}/resources/{uri}` decodes `uri` as a percent-encoded MCP resource URI before delegating to the runtime. This route must not treat `/` inside a resource URI as additional API path structure after decoding.

`POST /v1/call` accepts:

```json
{
  "action_id": "mcp:filesystem:tool:read_file",
  "arguments": {}
}
```

The route forwards the request to `agent-gateway` and returns the shared invocation response DTO.

MCP direct routes forward to the same runtime path:

```txt
GET /v1/mcp/servers
GET /v1/mcp/servers/{server_id}/tools
GET /v1/mcp/servers/{server_id}/resources
GET /v1/mcp/servers/{server_id}/resources/{uri}
GET /v1/mcp/servers/{server_id}/prompts
POST /v1/mcp/servers/{server_id}/tools/{tool_name}/call
```

## Auth

`agent-auth` decides whether a request is allowed. `agent-api` only enforces the decision and maps denial to normalized errors.

Loopback development mode is explicit config. Any non-loopback bind without auth should fail server startup.

## Verification

Add source-side tests for:

- router route registration.
- percent-encoded action ID handling.
- percent-encoded MCP resource URI handling.
- pagination/filtering query behavior.
- MCP server/tool/resource route delegation.
- handler state delegation.
- error envelope serialization.
- auth guard behavior for loopback and non-loopback binds.

Run:

```bash
cargo test -p agent-api
```

## Upstream References

- `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`
- `docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md`
- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`
- `docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md`
- `docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md`
- `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`
