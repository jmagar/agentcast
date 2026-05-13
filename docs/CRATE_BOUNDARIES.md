---
title: "Crate Boundaries"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs: []
related:
  - "docs/MVP.md"
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Crate Boundaries

This file defines ownership boundaries. New code should be placed by ownership, not convenience.

These are target boundaries for the full architecture. The MCP launcher MVP in [MVP.md](./MVP.md) controls implementation order. Crates for post-v0 capabilities may exist as stubs or contracts before they are feature-complete.

## `agent-core`

Owns:

- shared IDs.
- shared errors.
- shared event primitives.
- shared timestamps.
- common result/envelope types.
- small cross-cutting utilities with no external runtime dependency.

May depend on:

- `serde`
- `serde_json`
- `thiserror`
- `uuid`
- `jiff`
- `url`

Must not depend on:

- `tokio`
- `axum`
- `clap`
- `rmcp`
- `agent-client-protocol`
- `reqwest`
- Tauri/frontend code

## `agent-config`

Owns:

- app directory resolution.
- config file loading.
- environment-variable loading.
- config structs.
- precedence rules.
- validation of config shape.

Must not:

- spawn processes.
- call external registries.
- start network servers.
- parse CLI arguments directly.

## `agent-protocol`

Owns AgentCast protocol-neutral models:

- sessions.
- providers.
- agents.
- tools.
- workspaces.
- registry entries.
- marketplace entries.
- install plans.
- normalized events.

This crate should avoid direct dependency on ACP/MCP SDK crates. It defines AgentCast’s stable internal vocabulary.

For v0, this crate must define the MCP launcher vocabulary first:

- `McpServerId`.
- `McpToolId`.
- `LauncherActionId`.
- `LauncherAction`.
- `ToolInvocation`.
- `ToolInvocationResult`.
- `ServerStatus`.
- `InstallCandidate`.

## `agent-schema`

Owns schema normalization and validation:

- JSON Schema normalization for launcher actions and tool inputs.
- schema capability detection.
- validation helpers.
- CLI argument coercion metadata.
- future form-generation metadata.
- unsupported-schema fallback classification.

Must not:

- render CLI output.
- render frontend forms.
- invoke MCP tools.
- own registry or marketplace fetch policy.

## `agent-acp`

Owns ACP-specific adapter behavior only.

ACP support is part of the target architecture, but not required for the MCP launcher MVP. This crate can stay skeletal until the MVP runtime path is stable.

This crate should be a thin shim around the ACP SDK and AgentCast internal protocol models.

Owns:

- ACP type conversion.
- ACP connection helpers.
- ACP session method wrappers.
- ACP stdio transport setup.
- draft ACP Streamable HTTP/WebSocket transport tracking only when gated as post-v0 or experimental.
- ACP capability interpretation.
- ACP event parsing.
- ACP error mapping into AgentCast errors.

Must not own:

- provider selection policy.
- runtime session state.
- process supervision.
- marketplace install policy.
- CLI commands.
- HTTP routes.

Policy and orchestration live in `agent-runtime` and `agent-gateway`.

## `agent-mcp`

Owns MCP-specific adapter behavior only.

This crate should be a thin shim around the MCP SDK and AgentCast internal protocol models.

Owns:

- MCP type conversion.
- MCP client connection helpers.
- MCP server handler glue.
- MCP transport setup.
- MCP tool/resource/prompt discovery wrappers.
- MCP tool invocation wrappers.
- MCP error mapping into AgentCast errors.

Must not own:

- tool exposure policy.
- tool allowlists or denylists.
- marketplace install policy.
- gateway routing decisions.
- AgentCast runtime session state.
- process supervision.
- CLI commands.
- HTTP routes.

Policy lives in `agent-gateway`. Orchestration and process/session ownership live in `agent-runtime`.

## `agent-registry`

Owns registry aggregation:

- MCP Registry fetch/search/cache.
- registry normalization.
- curated local metadata overlays.
- registry freshness metadata.

For v0, implement official MCP Registry support first. The official MCP Registry is preview upstream, so this crate must model freshness, provenance, status changes, pagination, and local cache behavior explicitly. ACP Registry support is post-v0 unless explicitly promoted.

Must not:

- install files into Claude/Codex configs.
- spawn agents.
- expose HTTP routes directly.

## `agent-marketplace`

Owns marketplace/install planning:

- Claude plugin marketplace metadata.
- Codex plugin/config metadata only after an explicit upstream reference or AgentCast decision defines the supported format.
- plugin/skill/agent/slash-command component models.
- install targets.
- install previews.
- install plans.
- uninstall plans.

Must not directly mutate the filesystem without going through runtime/apply APIs unless explicitly scoped as a pure local helper.

For v0, marketplace install planning should only cover MCP server config/install plans. Claude/Codex plugin installation, generated loadouts, and ACP providers are end-state responsibilities.

## `agent-runtime`

Owns the main business/orchestration layer:

- agent process lifecycle.
- session lifecycle.
- prompt execution orchestration.
- workspace lifecycle.
- provider selection.
- local state/cache.
- normalized event stream.
- runtime logs.
- applying install plans.

Must not:

- contain HTTP route definitions.
- contain Clap command definitions.
- contain UI-only shaping.

## `agent-observability`

Owns reusable observability primitives:

- tracing setup helpers.
- audit event shapes.
- redaction helpers for logs and audit records.
- operation spans.
- runtime health snapshots.
- metrics DTOs when they are not surface-specific.

Must not own:

- business decisions.
- HTTP routes.
- CLI rendering.
- persistence engines.
- provider-specific policy.

## `agent-store`

Owns local persistence:

- SQLite connection setup.
- migrations.
- store traits and implementations.
- local cache directories.
- persisted runtime/catalog/install state.
- transaction helpers.

Must not own:

- runtime process lifecycle.
- protocol adapter behavior.
- gateway routing policy.
- surface DTO shaping.

## `agent-search`

Owns search and ranking behavior:

- launcher action indexing.
- aliases and category matching.
- normalized token/search metadata.
- ranking, recency, and favorites signals.
- future semantic/hybrid search abstractions.

Must not:

- invoke tools.
- spawn servers.
- fetch registries directly.
- own CLI or UI presentation.

## `agent-gateway`

Owns gateway-specific business rules:

- tool exposure policy.
- upstream routing.
- MCP tool merging.
- ACP-backed capability exposure.
- name collision resolution.
- allowlists/denylists.
- gateway health state.

Must call protocol/runtime crates rather than duplicating process/session behavior.

## `agent-auth`

Owns:

- bearer/session auth primitives.
- optional OAuth/server auth helpers.
- auth context extraction.
- auth errors.

Must not own product authorization policy beyond generic primitives unless documented.

## `agent-api`

Owns:

- Axum routers.
- HTTP DTOs.
- request extraction.
- response mapping.
- OpenAPI wiring.
- server state wrapper.

Must not own business logic. Handlers call runtime, gateway, registry, marketplace, or auth.

## `agent-ui-contracts`

Owns shared view/API contract DTOs for clients:

- launcher action list/detail view models.
- install-plan preview view models.
- server health/status view models.
- schema form metadata DTOs.
- shared request/response envelopes for web, desktop, Raycast-style clients, and external clients.

Must not:

- contain frontend components.
- contain Axum route handlers.
- duplicate protocol adapter models when `agent-protocol` is sufficient.
- own runtime policy.

## `agent-cli`

Owns:

- Clap command definitions.
- CLI rendering.
- terminal-friendly output.
- CLI argument validation.

Must not own business logic. Commands call runtime, gateway, registry, marketplace, or config.

## `agent-server`

Owns:

- binary entrypoint.
- tracing setup.
- top-level config load.
- API/CLI/server composition.
- graceful shutdown.

Must stay thin.

## `agent-stash`

Owns saved user artifacts and reusable context collections:

- saved launcher actions.
- saved prompts or invocation templates.
- collected references.
- local history bundles.
- future import/export formats for user-owned context.

This crate is post-v0 unless promoted by `docs/MVP.md`.

Must not:

- replace runtime persistence owned by `agent-store`.
- own gateway routing or invocation.
- own desktop/web UI state.

## `agent-fleet`

Owns future multi-node/remote-agent coordination:

- node identity metadata.
- remote runtime registration.
- heartbeat and capability summaries.
- remote execution coordination contracts.

This crate is post-v0 unless promoted by `docs/MVP.md`.

Must not:

- own local MCP stdio lifecycle.
- own local launcher routing.
- assume Lab-specific node or homelab policy.

## `apps/`

`apps/` contains deployable entrypoint directories. Reusable logic stays in `crates/`.

Initial app directories:

- `apps/cli`: CLI packaging/wrapper surface.
- `apps/mcp`: AgentCast MCP server entrypoint surface.
- `apps/api`: HTTP API app entrypoint surface.
- `apps/web`: future web UI.
- `apps/desktop`: future desktop app surface.

Apps may compose crates, package binaries, hold deployment assets, and contain app-specific tests. Apps must not become the only owner of runtime, gateway, registry, marketplace, config, auth, protocol, or schema behavior.
