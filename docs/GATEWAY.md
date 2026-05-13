---
title: "Gateway Contract"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/acp/docs/markdown/0051-agentclientprotocol-com-rfds-mcp-over-acp.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
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

# Gateway Contract

`agent-gateway` owns capability exposure and routing.

For v0, gateway means the MCP launcher catalog/router. The broader bridge and public gateway behavior is post-v0 unless explicitly promoted.

## Responsibilities

Gateway owns:

- merged tool catalogs.
- upstream health.
- exposure filters.
- name collision policy.
- routing decisions.
- bridge policy between ACP and MCP for post-v0 work.
- public gateway metadata for post-v0 work.

## MVP Gateway Mode

The first gateway implementation should:

- merge tools from configured MCP servers into launcher actions.
- preserve source server id and original tool name.
- apply deterministic exposed ids/names.
- detect collisions.
- support explicit aliases later without silently rewriting names.
- route an invocation request back to the owning MCP server/tool.

It should not initially expose ACP-backed capabilities, remote device routing, or public gateway policy.

## Upstream Types

Gateway upstreams may be:

- MCP stdio server.
- MCP Streamable HTTP server.
- ACP agent.
- AgentCast runtime capability.
- generated extension.

## Tool Exposure Policy

Every exposed tool must be traceable to:

- source upstream.
- original name.
- exposed name.
- description.
- input schema.
- risk level if known.
- required confirmation if destructive.

MCP `ToolAnnotations` can inform display and risk hints only when the source is trusted. They are upstream hints, not security guarantees.

## Collision Rules

Tool name collisions must not be resolved silently.

Allowed strategies:

- prefix by upstream.
- hide duplicate.
- require explicit alias.
- fail gateway config validation.

## Gateway Is Not Runtime

Gateway routes. Runtime owns processes and sessions.

## Upstream References Checked

- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`: MCP tool metadata, tool annotations, and trust guidance.
- `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`: `Tool`, `ToolAnnotations`, `inputSchema`, `outputSchema`, and `title` fields.
- `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`: stdio and Streamable HTTP upstream types.
- `docs/references/acp/docs/markdown/0051-agentclientprotocol-com-rfds-mcp-over-acp.md`: MCP-over-ACP is RFD material, not v0 required behavior.
- `docs/references/seed/chatgpt-windows-raycast-extensions.md`: product-origin distinction between MCP capabilities and ACP agents.
