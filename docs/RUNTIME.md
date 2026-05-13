---
title: "Runtime Contract"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/acp/docs/markdown/0045-agentclientprotocol-com-protocol-session-setup.md"
  - "docs/references/acp/docs/markdown/0063-agentclientprotocol-com-protocol-session-list.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
related:
  - "docs/MVP.md"
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Runtime Contract

`agent-runtime` is the primary business/orchestration layer for AgentCast.

This contract describes the target runtime. The v0 slice is the MCP launcher runtime described in [MVP.md](./MVP.md).

## Responsibilities

The runtime owns:

- configured MCP server lifecycle for v0.
- launcher action discovery and invocation for v0.
- session creation.
- session resume/list/close.
- prompt sending.
- process spawning.
- process supervision.
- provider selection.
- workspace allocation.
- event normalization.
- local state persistence.
- install plan application.
- runtime logs.

The runtime should preserve the "one binary, one local port when serving, fully self-hosted" shape. Local workflows must not depend on a hosted control plane.

For v0, session-oriented responsibilities may remain unimplemented as long as crate boundaries preserve them.

The v0 runtime is therefore a local MCP launcher runtime, not a full ACP session host. ACP session lifecycle text in this document is target architecture.

## Runtime API Shape

The runtime should expose high-level operations, not protocol-shaped functions.

Examples:

```rust
runtime.list_providers().await?;
runtime.create_session(request).await?;
runtime.resume_session(session_id).await?;
runtime.send_prompt(session_id, prompt).await?;
runtime.close_session(session_id).await?;
runtime.stream_events(session_id).await?;
runtime.apply_install_plan(plan).await?;
```

MVP operations should include:

```rust
runtime.start_configured_mcp_servers().await?;
runtime.discover_launcher_actions().await?;
runtime.list_launcher_actions(query).await?;
runtime.invoke_launcher_action(action_id, input).await?;
runtime.stop_mcp_server(server_id).await?;
```

## Runtime Inputs

Runtime inputs should use AgentCast internal types from `agent-protocol`, not raw ACP/MCP SDK types.

Bad:

```rust
fn create_session(req: acp::SessionNewRequest) -> ...
```

Good:

```rust
fn create_session(req: AgentcastCreateSessionRequest) -> ...
```

The ACP/MCP crates translate external protocol types into AgentCast internal types.

## Session Lifecycle

A session is an AgentCast-owned concept. It may map to:

- an ACP session.
- an MCP-backed workflow.
- a local process session.
- a future remote runtime session.

Session state must track:

- session id.
- provider id.
- workspace root.
- created/updated timestamps.
- status.
- current model/config when known.
- event stream cursor or log location.

## Event Normalization

Runtime events are protocol-neutral.

Examples:

- `SessionStarted`
- `SessionUpdated`
- `PromptStarted`
- `TextDelta`
- `ReasoningDelta`
- `ToolCallStarted`
- `ToolCallCompleted`
- `TerminalStarted`
- `TerminalOutputDelta`
- `FileChanged`
- `PermissionRequested`
- `PromptCompleted`
- `SessionClosed`
- `RuntimeError`

ACP/MCP-specific fields may be preserved under metadata, but UI/API consumers should not be forced to understand raw protocol payloads.

If future ACP support maps stabilized methods such as `session/resume`, `session/list`, or `session/close`, the runtime must keep those capability checks in the ACP adapter and expose protocol-neutral runtime operations above them.

## Process Supervision

The runtime owns child process lifecycle for local agents.

Rules:

- spawned processes must be associated with a runtime-owned handle.
- shutdown must be explicit and observable.
- process output must be captured or intentionally discarded.
- crashes must produce structured runtime events.
- orphaned processes are bugs.

## Persistence

Runtime persistence should be behind traits first.

Initial implementation may be in-memory. Add SQLite when session/cache/index behavior requires durable local state. PostgreSQL may be added later as an optional backing store if SQLite is no longer sufficient.

Suggested traits:

```rust
trait SessionStore {}
trait EventStore {}
trait ProviderStore {}
trait InstallStore {}
```

Do not spread direct SQLite calls across runtime modules.

## Upstream References Checked

- `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`: local process ownership for stdio MCP servers and Streamable HTTP later.
- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`: model-controlled tool invocations and human-in-the-loop guidance.
- `docs/references/acp/docs/markdown/0045-agentclientprotocol-com-protocol-session-setup.md`: ACP session setup/resume/close target behavior.
- `docs/references/acp/docs/markdown/0063-agentclientprotocol-com-protocol-session-list.md`: future ACP session listing target behavior.
- `docs/reports/lab-extraction-source-map.md`: local Lab extraction evidence for runtime/process/store boundaries.
