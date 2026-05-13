---
title: "Protocol Integration Contract"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "end-state"
source_of_truth: false
upstream_refs:
  - "docs/references/acp/docs/markdown/0011-agentclientprotocol-com-protocol-initialization.md"
  - "docs/references/acp/docs/markdown/0019-agentclientprotocol-com-announcements-acp-agent-registry-stabilized.md"
  - "docs/references/acp/docs/markdown/0045-agentclientprotocol-com-protocol-session-setup.md"
  - "docs/references/acp/docs/markdown/0061-agentclientprotocol-com-protocol-session-config-options.md"
  - "docs/references/acp/docs/markdown/0063-agentclientprotocol-com-protocol-session-list.md"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Protocol Integration Contract

AgentCast integrates with two protocol families:

- ACP: agent/client session protocol.
- MCP: tool/resource/prompt protocol.

Both protocols are adapters into AgentCast’s internal model.

Protocol status matters. MCP launcher behavior is v0. ACP support is target architecture unless a specific feature is promoted. ACP RFDs are design input until their corresponding protocol docs or stabilization announcements say otherwise.

## Delivery Order

MCP is the first protocol implemented for the launcher MVP. ACP remains part of the target architecture, but it should not block MCP server discovery, launcher action generation, or deterministic tool invocation.

The first shipped path is:

```txt
configured MCP server -> agent-mcp -> runtime/gateway -> CLI/API/UI surface
```

ACP paths are post-v0 unless explicitly promoted.

Stabilized ACP features relevant to future work include `session/resume`, `session/list`, `session/close`, Session Config Options, and the ACP Agent Registry. Draft/RFD-only features such as `session/fork`, proxy chains, MCP-over-ACP, and custom LLM endpoint/provider methods must be labeled as non-v0 design input.

## ACP Ownership

`agent-acp` owns ACP-specific adapter code, not product policy.

It translates:

```txt
ACP initialize/session/prompt/update events
  -> AgentCast provider/session/runtime events
```

It should expose low-level operations like:

```rust
connect_agent(...)
create_acp_session(...)
send_acp_prompt(...)
list_acp_sessions(...)
resume_acp_session(...)
```

These operations should be wrappers. They should not decide which provider is selected, which process is spawned, or how runtime state is persisted.

ACP initialization uses `protocolVersion: 1` and capability-advertised features. Clients must check the relevant `sessionCapabilities` or `mcpCapabilities` before invoking optional methods.

## MCP Ownership

`agent-mcp` owns MCP-specific adapter code, not product policy.

It translates:

```txt
MCP tools/resources/prompts/server metadata
  -> AgentCast tool/capability models
```

It should expose low-level operations like:

```rust
connect_mcp_server(...)
list_tools(...)
call_tool(...)
list_resources(...)
read_resource(...)
```

These operations should be intentionally boring wrappers. They should not decide whether a tool is exposed, renamed, filtered, trusted, installed, or routed. Those decisions belong in `agent-gateway`, `agent-marketplace`, or `agent-runtime` depending on the concern.

## AgentCast MCP Surface

When AgentCast exposes its own MCP server surface, it is an agent-facing adapter over the same runtime behavior as CLI and API.

List/search-style AgentCast MCP tools must support pagination and filtering. This applies to tools that expose:

- configured MCP servers.
- discovered MCP tools.
- discovered MCP resources.
- discovered MCP prompts.
- launcher actions.
- registry candidates.
- install plans.
- runtime health/events.

Required input fields should use stable names where possible:

```json
{
  "limit": 50,
  "cursor": null,
  "filter": null
}
```

Rules:

- filtering applies before pagination.
- responses include a continuation cursor or explicit end-of-results marker.
- responses should be concise by default and require explicit follow-up calls for large details.
- MCP tools must not return unbounded collections by default.
- MCP output should use the same domain DTOs as CLI `--json` and API responses where practical.

The MCP surface may provide tool names such as `servers_list`, `tools_list`, `resources_list`, `prompts_list`, `resource_read`, `tool_call`, `registry_search`, and `actions_search`, but naming must remain subordinate to the shared runtime contract.

Those names are AgentCast MCP-server tool names, not upstream MCP protocol method names. The upstream methods remain JSON-RPC methods such as `tools/list`, `tools/call`, `resources/list`, `resources/read`, `prompts/list`, and `prompts/get`.

## No Protocol Leakage Rule

Surface crates should not expose raw ACP/MCP SDK types in public API responses.

Allowed:

- storing raw protocol payload in `_meta` or debug fields.
- protocol-specific adapters in ACP/MCP crates.

Not allowed:

- API route returns raw ACP SDK event enum directly.
- CLI command branches on raw MCP SDK response shape.
- runtime session store persists raw protocol objects as the primary schema.

## Bridging

AgentCast may bridge protocols:

- ACP-backed agent capabilities exposed as MCP tools.
- MCP tools attached to ACP agent sessions.
- MCP servers launched as child processes for agents.

Bridge ownership:

```txt
agent-gateway: exposure/routing/filtering/bridge policy
agent-runtime: process/session/workspace ownership
agent-acp: ACP transport and type-conversion shim
agent-mcp: MCP transport and type-conversion shim
```

## Upstream References Checked

- `docs/references/acp/docs/markdown/0011-agentclientprotocol-com-protocol-initialization.md`: ACP initialization and capability advertisement.
- `docs/references/acp/docs/markdown/0045-agentclientprotocol-com-protocol-session-setup.md`: stabilized `session/resume` and `session/close`, plus MCP capability checks.
- `docs/references/acp/docs/markdown/0063-agentclientprotocol-com-protocol-session-list.md`: stabilized session list behavior.
- `docs/references/acp/docs/markdown/0061-agentclientprotocol-com-protocol-session-config-options.md`: stabilized Session Config Options and mode replacement.
- `docs/references/acp/docs/markdown/0019-agentclientprotocol-com-announcements-acp-agent-registry-stabilized.md`: ACP Registry stabilization.
- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`: upstream MCP tool method names.
- `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`: upstream MCP resource method names.
