---
title: "HTTP API Contract"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/acp/docs/markdown/0011-agentclientprotocol-com-protocol-initialization.md"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
  - "docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0172-modelcontextprotocol-io-registry-faq.md"
related:
  - "docs/MVP.md"
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# HTTP API Contract

`agent-api` owns HTTP routes and response mapping.

API is not required before the CLI proves the MVP runtime path. When added, it must expose the same runtime/gateway behavior described in [MVP.md](./MVP.md).

This is an AgentCast HTTP contract, not an MCP or ACP protocol surface. MCP and ACP details must be translated by the runtime/protocol crates before they reach API responses.

## Rules

API handlers must be thin.

They may:

- extract path/query/body.
- validate HTTP-level request shape.
- call runtime/gateway/registry/marketplace.
- map domain result to HTTP response.

They must not:

- spawn agents directly.
- parse registry sources directly.
- mutate config directly.
- contain business policy that is not also available to CLI/UI.

## Response Shape

Use stable envelopes for API responses.

Suggested success envelope:

```json
{
  "data": {},
  "meta": {
    "next_cursor": null
  }
}
```

Suggested error envelope:

```json
{
  "error": {
    "kind": "...",
    "message": "...",
    "details": {}
  }
}
```

## Route Families

Initial route families:

```txt
/v1/health
/v1/mcp/servers
/v1/launcher/actions
/v1/launcher/invocations
/v1/registry
/v1/gateway
/v1/config
```

## Agent-Friendly API Rules

The API is a primary agent surface.

Requirements:

- collection endpoints support pagination.
- collection endpoints support filtering before pagination.
- response envelopes are stable and machine-readable.
- errors use informative normalized envelopes.
- endpoints expose the same server/tool/resource operations as the CLI.
- no endpoint returns unbounded collections by default.
- large payloads use explicit detail/read endpoints instead of being embedded in list responses.

Initial MCP runtime routes must cover:

```txt
GET /v1/mcp/servers
GET /v1/mcp/servers/{server_id}/tools
GET /v1/mcp/servers/{server_id}/resources
GET /v1/mcp/servers/{server_id}/resources/{uri}
GET /v1/mcp/servers/{server_id}/prompts
POST /v1/mcp/servers/{server_id}/tools/{tool_name}/call
```

Route parameters that carry upstream identifiers, such as MCP resource URIs and tool names, must be encoded safely. Do not infer MCP semantics from path shape; call the runtime with structured fields.

Future route families:

```txt
/v1/providers
/v1/sessions
/v1/sessions/:id/events
/v1/marketplace
```

## Upstream References Checked

- `docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md`: MCP cursor pagination model.
- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`: `tools/list` and `tools/call` operations.
- `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`: `resources/list` and `resources/read` behavior.
- `docs/references/mcp/docs/markdown/0172-modelcontextprotocol-io-registry-faq.md`: official MCP Registry API terminology and preview status.
- `docs/references/acp/docs/markdown/0011-agentclientprotocol-com-protocol-initialization.md`: ACP protocol fields are adapter inputs, not raw API response shapes.
