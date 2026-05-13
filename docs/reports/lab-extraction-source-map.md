---
title: "Lab Extraction Source Map"
doc_type: "report"
status: "active"
owner: "agentcast"
audience:
  - "maintainers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/acp/docs/markdown/0011-agentclientprotocol-com-protocol-initialization.md"
  - "docs/references/acp/docs/markdown/0022-agentclientprotocol-com-protocol-overview.md"
  - "docs/references/acp/docs/markdown/0028-agentclientprotocol-com-protocol-content.md"
  - "docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md"
  - "docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md"
  - "docs/references/acp/docs/markdown/0062-agentclientprotocol-com-protocol-prompt-turn.md"
  - "docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md"
  - "docs/references/claude-code/markdown/0093-code-claude-com-docs-en-plugins.md"
  - "docs/references/claude-code/markdown/0108-code-claude-com-docs-en-plugin-marketplaces.md"
  - "docs/references/fastmcp/docs/client.mdx"
  - "docs/references/fastmcp/docs/inspecting.mdx"
  - "docs/references/fastmcp/docs/overview.mdx"
  - "docs/references/fastmcp/docs/running.mdx"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
  - "docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
  - "docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md"
  - "docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md"
  - "docs/references/mcp/docs/markdown/0185-modelcontextprotocol-io-docs-learn-architecture.md"
  - "docs/references/mcporter/docs/cli-reference.md"
  - "docs/references/mcporter/docs/spec.md"
  - "docs/references/mcporter/docs/tool-calling.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Lab Extraction Source Map

This report records the first systematic live-source pass over `../lab` for AgentCast crate extraction planning.

`docs/references/` is not edited during this pass. Live `../lab` is preferred when available; `docs/references/jmagar/jmagar-lab.xml` remains the fallback source-of-truth snapshot when live source and repopack need comparison. Every source mapping below was rechecked against the Lab snapshot path inventory in `docs/references/jmagar/jmagar-lab.xml`; protocol/API assumptions were cross-checked against the local upstream docs listed in the reference checks section.

## Upstream Reference Checks

- Lab source evidence: `docs/references/jmagar/jmagar-lab.xml`.
- MCP protocol/API evidence: `docs/references/mcp/docs/markdown/0185-modelcontextprotocol-io-docs-learn-architecture.md`, `docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md`, `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`, `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`, `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`, `docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md`, `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`, `docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md`, and `docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md`.
- ACP protocol/API evidence: `docs/references/acp/docs/markdown/0022-agentclientprotocol-com-protocol-overview.md`, `docs/references/acp/docs/markdown/0011-agentclientprotocol-com-protocol-initialization.md`, `docs/references/acp/docs/markdown/0062-agentclientprotocol-com-protocol-prompt-turn.md`, `docs/references/acp/docs/markdown/0028-agentclientprotocol-com-protocol-content.md`, `docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md`, and `docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md`.
- Claude Code, FastMCP, and mcporter comparison evidence: `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`, `docs/references/claude-code/markdown/0093-code-claude-com-docs-en-plugins.md`, `docs/references/claude-code/markdown/0108-code-claude-com-docs-en-plugin-marketplaces.md`, `docs/references/fastmcp/docs/overview.mdx`, `docs/references/fastmcp/docs/client.mdx`, `docs/references/fastmcp/docs/running.mdx`, `docs/references/fastmcp/docs/inspecting.mdx`, `docs/references/mcporter/docs/spec.md`, `docs/references/mcporter/docs/tool-calling.md`, and `docs/references/mcporter/docs/cli-reference.md`.

## High-Value Source Clusters

| AgentCast crate | Primary Lab evidence | Extractable patterns | Leave behind |
|---|---|---|---|
| `agent-core` | `../lab/crates/lab-apis/src/core.rs`, `../lab/crates/lab/src/dispatch/error.rs`, `../lab/crates/lab/src/dispatch/helpers.rs` | action metadata, stable error kinds, JSON param helpers, path/redaction helpers | service-specific plugin metadata and client pools |
| `agent-protocol` | `../lab/crates/lab-apis/src/acp.rs`, `../lab/crates/lab-apis/src/acp/types.rs`, `../lab/crates/lab-apis/src/mcpregistry.rs`, `../lab/crates/lab-apis/src/acp_registry.rs`, `../lab/crates/lab/src/dispatch/gateway/types.rs` | protocol-neutral session, tool, registry, gateway, marketplace DTOs | SDK clients, HTTP envelopes, CLI formatting |
| `agent-config` | `../lab/crates/lab/src/config.rs`, `../lab/crates/lab/src/config/env_merge.rs`, `../lab/crates/lab/src/dispatch/gateway/config.rs`, `../lab/crates/lab/src/dispatch/gateway/config_mutation.rs` | TOML loading, env merge, upstream config, config mutation, validation | `~/.lab`, service credential scanning, Lab env names |
| `agent-mcp` | `../lab/crates/lab/src/mcp/server.rs`, `../lab/crates/lab/src/mcp/catalog.rs`, `../lab/crates/lab/src/mcp/error.rs`, `../lab/crates/lab/src/mcp/upstream.rs`, `../lab/crates/lab/src/mcp/registry.rs` | RMCP server shape, MCP error normalization, catalog/resource/prompt handling | Lab service dispatch and built-in service registry |
| `agent-runtime` | `../lab/crates/lab/src/dispatch/gateway/runtime.rs`, `../lab/crates/lab/src/process.rs`, `../lab/crates/lab/src/process/unix.rs`, `../lab/crates/lab/src/acp/runtime.rs`, `../lab/crates/lab/src/acp/registry.rs` | process groups, cleanup matching, runtime state, lifecycle snapshots, ACP post-v0 session runtime | deploy pipeline, node policy, Lab-specific process matchers |
| `agent-gateway` | `../lab/crates/lab/src/dispatch/gateway/index.rs`, `../lab/crates/lab/src/dispatch/gateway/dispatch.rs`, `../lab/crates/lab/src/dispatch/gateway/projection.rs`, `../lab/crates/lab/src/dispatch/gateway/runtime.rs`, `../lab/crates/lab/src/dispatch/gateway/service_catalog.rs`, `../lab/crates/lab/src/dispatch/gateway/types.rs`, `../lab/crates/lab/src/dispatch/gateway/virtual_servers.rs` | tool index, search/ranking, catalog diff, exposure policy, view model redaction, routing, and GatewayManager runtime impls exposed in the snapshot | virtual Lab service catalog, OAuth lifecycle unless promoted |
| `agent-registry` | `../lab/crates/lab-apis/src/mcpregistry.rs`, `../lab/crates/lab-apis/src/acp_registry.rs`, `../lab/crates/lab/src/api/services/registry_v01.rs`, `../lab/crates/lab/src/dispatch/marketplace/store.rs` | registry DTOs, local cache, list/search/version filters, cursor handling | ACP install behavior until post-v0, UI cards |
| `agent-marketplace` | `../lab/crates/lab/src/dispatch/marketplace/mcp_dispatch.rs`, `../lab/crates/lab/src/dispatch/marketplace/mcp_params.rs`, `../lab/crates/lab/src/dispatch/marketplace/mcp_catalog.rs`, `../lab/crates/lab/src/dispatch/marketplace/store.rs`, `../lab/crates/lab/src/dispatch/marketplace/service.rs`, `../lab/crates/lab/src/dispatch/marketplace/package.rs`, `../lab/crates/lab/src/dispatch/marketplace/diff.rs` | MCP install plan generation, runtime/env validation, metadata normalization, dry-run/conflict handling | Claude/Codex plugin write policy for v0, Lab artifact forks |
| `agent-auth` | `../lab/crates/lab-auth/src/auth_context.rs`, `../lab/crates/lab-auth/src/middleware.rs`, `../lab/crates/lab-auth/src/config.rs`, `../lab/crates/lab-auth/src/error.rs`, `../lab/crates/lab-auth/src/session.rs`, `../lab/crates/lab-auth/src/state.rs`, `../lab/crates/lab-auth/src/token.rs`, `../lab/crates/lab-auth/src/types.rs` | auth context, bearer/session extraction, OAuth metadata DTOs, stable error kinds | Google OAuth policy and Lab admin allowlists by default |
| `agent-api` | `../lab/crates/lab/src/api/router.rs`, `../lab/crates/lab/src/api/state.rs`, `../lab/crates/lab/src/api/error.rs`, `../lab/crates/lab/src/api/health.rs`, `../lab/crates/lab/src/api/host_validation.rs`, `../lab/crates/lab/src/api/openapi.rs`, `../lab/crates/lab/src/api/services/helpers.rs` | Axum route composition, state injection, auth layers, error mapping, OpenAPI generation | service-specific routes, browser admin fallback until app exists |
| `agent-cli` | `../lab/crates/lab/src/cli.rs`, `../lab/crates/lab/src/cli/helpers.rs`, `../lab/crates/lab/src/cli/gateway.rs`, `../lab/crates/lab/src/cli/marketplace.rs`, `../lab/crates/lab/src/cli/install.rs`, `../lab/crates/lab/src/output/render.rs`, `../lab/crates/lab/src/output/theme.rs` | Clap layout, JSON/human output, confirmation patterns, render theming | homelab service commands |
| `agent-server` | `../lab/crates/lab/src/main.rs`, `../lab/crates/lab/src/cli/serve.rs`, `../lab/crates/lab/src/api/router.rs`, `../lab/crates/lab/src/api/state.rs`, `../lab/crates/lab/src/observability.rs` | binary composition, logging startup, serve/shutdown wiring | Lab service registry assembly and node boot policy |
| `agent-observability` | `../lab/crates/lab/src/observability.rs`, `../lab/crates/lab/src/observability/activity.rs`, `../lab/crates/lab/src/observability/activity_event.rs`, `../lab/crates/lab/src/audit.rs`, `../lab/crates/lab/src/log_fmt/formatter.rs`, `../lab/crates/lab/src/dispatch/logs/store.rs`, `../lab/crates/lab/src/dispatch/logs/stream.rs`, `../lab/crates/lab/src/dispatch/logs/types.rs` | activity/audit event shapes, log formatting, local log stream/store patterns | peer/node log ingestion for v0 unless promoted |
| `agent-store` | `../lab/crates/lab/src/dispatch/marketplace/store.rs`, `../lab/crates/lab/src/dispatch/stash/store.rs`, `../lab/crates/lab/src/node/store.rs`, `../lab/crates/lab/src/node/log_store.rs`, `../lab/crates/lab-auth/src/sqlite.rs` | SQLite migrations, restrictive file permissions, cache/store traits, local metadata | Lab registry schema as-is, node auth DB coupling |
| `agent-stash` | `../lab/crates/lab/src/dispatch/stash/catalog.rs`, `../lab/crates/lab/src/dispatch/stash/service.rs`, `../lab/crates/lab/src/dispatch/stash/store.rs`, `../lab/crates/lab/src/dispatch/stash/import.rs`, `../lab/crates/lab/src/dispatch/stash/export.rs`, `../lab/crates/lab/src/dispatch/stash/revision.rs`, `../lab/crates/lab/src/dispatch/stash/providers/filesystem.rs`, `../lab/crates/lab/src/dispatch/marketplace/stash_meta.rs`, `../lab/crates/lab/src/mcp/services/stash.rs`, `../lab/crates/lab-apis/src/stash.rs` | stash metadata, provider abstraction, import/export/revision flow, path safety | Lab plugin artifact assumptions |
| `agent-fleet` | `../lab/crates/lab/src/node/identity.rs`, `../lab/crates/lab/src/node/enrollment.rs`, `../lab/crates/lab/src/node/runtime.rs`, `../lab/crates/lab/src/node/checkin.rs`, `../lab/crates/lab/src/node/queue.rs`, `../lab/crates/lab/src/node/store.rs`, `../lab/crates/lab/src/node/health.rs`, `../lab/crates/lab/src/api/nodes/fleet.rs`, `../lab/crates/lab/src/api/nodes/status.rs`, `../lab/crates/lab/src/dispatch/node/send.rs`, `../lab/crates/lab/tests/node_queue.rs`, `../lab/crates/lab/tests/device_runtime.rs` | identity, enrollment, heartbeat, queue, runtime, log collection, status DTOs | Lab master/worker policy unless promoted |
| `agent-search` | `../lab/crates/lab/src/dispatch/gateway/index.rs`, `../lab/crates/lab/src/dispatch/gateway/projection.rs`, `../lab/crates/lab/src/dispatch/gateway/runtime.rs`, `../lab/crates/lab/src/config.rs` (`ToolSearchConfig`) | indexed tool document, search scoring, catalog hashing, stale-index refresh | gateway-owned invocation and runtime lifecycle |
| `agent-schema` | `../lab/crates/lab-apis/src/core.rs`, `../lab/crates/lab/src/mcp/server.rs` (`action_schema`), `../lab/crates/lab/src/api/openapi.rs`, `../lab/apps/gateway-admin/components/ai/schema-display.tsx` | parameter schema metadata, JSON Schema generation, OpenAPI type mapping, schema display states | UI component implementation and service params |
| `agent-ui-contracts` | `../lab/crates/lab/src/dispatch/gateway/types.rs`, `../lab/apps/gateway-admin/lib/api/gateway-client.ts`, `../lab/apps/gateway-admin/lib/api/marketplace-client.ts`, `../lab/apps/gateway-admin/lib/api/mcpregistry-client.ts`, `../lab/apps/gateway-admin/components/gateway/gateway-detail-content.tsx`, `../lab/apps/gateway-admin/components/marketplace/mcp-server-card.tsx`, `../lab/apps/gateway-admin/components/registry/server-detail-panel.tsx`, `../lab/apps/gateway-admin/components/chat/types.ts`, `../lab/apps/gateway-admin/lib/acp/types.ts` | client-facing DTOs and view models for gateway, registry, marketplace, ACP/chat | React components and Lab copy |

## Planning Rules From The Source Pass

- Use live source path evidence in every extraction plan.
- Treat ACP runtime/session behavior as post-v0 unless `docs/MVP.md` changes.
- Use Lab tests as behavior evidence, but port tests into AgentCast-owned fixtures and names.
- Do not copy Lab service modules into AgentCast core.
- Do not rewrite `docs/references/`; update first-party plans, specs, contracts, and reports instead.
