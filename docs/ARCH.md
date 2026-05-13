---
title: "Architecture"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "end-state"
source_of_truth: true
upstream_refs: []
related:
  - "docs/MVP.md"
  - "docs/plans/extract-crates/README.md"
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Architecture

AgentCast is a Rust workspace centered on agent runtime orchestration and ACP/MCP interoperability.

This document describes the target architecture. The first implementation slice is the MCP launcher/runtime in [MVP.md](./MVP.md).

AgentCast is a separate repo, but it should be bootstrapped by extracting and generalizing proven Lab patterns where appropriate. See [plans/extract-crates/README.md](./plans/extract-crates/README.md).

## Core Shape

- One Rust workspace.
- One primary server binary.
- One local port when serving API/MCP surfaces.
- Fully self-hosted core runtime.
- SQLite-backed local state by default, with PostgreSQL only as a future optional backend when needed.
- Multiple small library crates.
- Runtime-first business logic.
- Protocol adapters for ACP and MCP.
- Registry and marketplace domain crates.
- CLI/API as thin surfaces.
- Fast, simple, composable, reusable, portable, organized, and logical modules.

Protocol support must track upstream stability. MCP stdio and Streamable HTTP are standard transports in the captured MCP references. ACP stdio is stable; ACP Streamable HTTP/WebSocket remains draft/proposal material in the captured ACP references and must stay post-v0 or experimental until upstream stabilizes it.

## MVP Slice

The end-state architecture must support ACP sessions, marketplace-compatible artifacts, loadouts, and remote execution. v0 only has to prove the MCP launcher path:

```txt
config -> MCP server runtime -> launcher action catalog -> deterministic invocation -> structured result
```

For v0:

- `agent-mcp` wraps MCP connect/list/call behavior.
- `agent-runtime` owns configured server lifecycle and action invocation.
- `agent-gateway` owns merged catalog and collision policy.
- `agent-cli` is the first usable surface.
- API, MCP gateway endpoint, desktop UI, ACP, and loadout execution come after the shared runtime path works.

## Initial Workspace

```txt
crates/
  agent-core/
  agent-config/
  agent-protocol/
  agent-schema/
  agent-acp/
  agent-mcp/
  agent-registry/
  agent-marketplace/
  agent-observability/
  agent-store/
  agent-search/
  agent-runtime/
  agent-gateway/
  agent-auth/
  agent-api/
  agent-cli/
  agent-server/
  agent-stash/
  agent-fleet/
  agent-ui-contracts/

apps/
  cli/
  mcp/
  api/
  web/
  desktop/
```

## Layer Model

```txt
Shared primitives:
  agent-core
  agent-protocol
  agent-config
  agent-schema
  agent-observability

Protocol adapters:
  agent-acp
  agent-mcp

Domain/business services:
  agent-store
  agent-runtime
  agent-gateway
  agent-registry
  agent-marketplace
  agent-auth
  agent-search
  agent-stash
  agent-fleet

Surfaces:
  agent-api
  agent-cli
  agent-ui-contracts

Composition:
  agent-server
  apps/*
```

## Dependency Direction

Allowed high-level dependency direction:

```txt
agent-core
  ↓
agent-protocol
agent-config
agent-schema
agent-observability
  ↓
agent-acp
agent-mcp
agent-registry
agent-marketplace
agent-store
agent-runtime
agent-auth
agent-search
agent-stash
agent-fleet
  ↓
agent-gateway
  ↓
agent-api
agent-cli
agent-ui-contracts
  ↓
agent-server
apps/*
```

The exact Cargo graph may differ slightly, but the conceptual direction must hold:

- lower-level crates must not depend on surface crates.
- protocol adapters must not know about HTTP routes or CLI commands.
- API and CLI must not own core business behavior.
- runtime must not import API or CLI.
- UI contract DTOs must stay reusable and must not import frontend implementation code.

## Runtime-Centered Design

AgentCast is not a generic SDK collection. The central product object is the runtime.

In the full architecture, the runtime answers:

- What agents are available?
- What sessions exist?
- How is a prompt sent?
- Which process owns this session?
- How are events normalized?
- What workspace is attached?
- What should be persisted?
- What must be supervised or cleaned up?

For v0, the runtime answers the narrower MCP launcher questions:

- What MCP servers are configured?
- Which configured servers are running?
- What tools did those servers expose?
- Which launcher actions are searchable?
- How does an action route back to a server/tool?
- What result or error came back from invocation?

## Product Surfaces

AgentCast exposes behavior through:

- CLI commands.
- HTTP API.
- future desktop/web UI.
- MCP gateway endpoint.
- ACP integration paths.

All surfaces must call shared domain/runtime APIs.

## Non-Goals

AgentCast core will not contain service-specific homelab integrations. If Labby needs those, Labby can depend on AgentCast or install AgentCast-managed capabilities as plugins.

This non-goal does not prohibit extracting generic runtime, gateway, registry, marketplace, API, CLI, auth, observability, or UI patterns from Lab. It only prohibits carrying Lab-specific service product behavior into AgentCast core.
