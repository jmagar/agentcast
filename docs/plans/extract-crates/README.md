---
title: "Lab Extraction Bootstrap Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/acp/docs/markdown/0011-agentclientprotocol-com-protocol-initialization.md"
  - "docs/references/acp/docs/markdown/0018-agentclientprotocol-com-libraries-rust.md"
  - "docs/references/acp/docs/markdown/0022-agentclientprotocol-com-protocol-overview.md"
  - "docs/references/acp/docs/markdown/0028-agentclientprotocol-com-protocol-content.md"
  - "docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md"
  - "docs/references/acp/docs/markdown/0032-agentclientprotocol-com-protocol-file-system.md"
  - "docs/references/acp/docs/markdown/0043-agentclientprotocol-com-get-started-registry.md"
  - "docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md"
  - "docs/references/acp/docs/markdown/0062-agentclientprotocol-com-protocol-prompt-turn.md"
  - "docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md"
  - "docs/references/claude-code/markdown/0093-code-claude-com-docs-en-plugins.md"
  - "docs/references/claude-code/markdown/0102-code-claude-com-docs-en-plugins-reference.md"
  - "docs/references/claude-code/markdown/0108-code-claude-com-docs-en-plugin-marketplaces.md"
  - "docs/references/fastmcp/docs/auth.mdx"
  - "docs/references/fastmcp/docs/client.mdx"
  - "docs/references/fastmcp/docs/generate-cli.mdx"
  - "docs/references/fastmcp/docs/inspecting.mdx"
  - "docs/references/fastmcp/docs/overview.mdx"
  - "docs/references/fastmcp/docs/running.mdx"
  - "docs/references/jmagar/jmagar-aurora-design-system.xml"
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
  - "docs/references/mcporter/docs/agent-skills.md"
  - "docs/references/mcporter/docs/cli-reference.md"
  - "docs/references/mcporter/docs/livetests.md"
  - "docs/references/mcporter/docs/mcp.md"
  - "docs/references/mcporter/docs/spec.md"
  - "docs/references/mcporter/docs/tool-calling.md"
related:
  - "docs/plans/extract-crates/agent-acp.md"
  - "docs/plans/extract-crates/agent-api.md"
  - "docs/plans/extract-crates/agent-auth.md"
  - "docs/plans/extract-crates/agent-cli.md"
  - "docs/plans/extract-crates/agent-config.md"
  - "docs/plans/extract-crates/agent-core.md"
  - "docs/plans/extract-crates/agent-fleet.md"
  - "docs/plans/extract-crates/agent-gateway.md"
  - "docs/plans/extract-crates/agent-marketplace.md"
  - "docs/plans/extract-crates/agent-mcp.md"
  - "docs/plans/extract-crates/agent-observability.md"
  - "docs/plans/extract-crates/agent-protocol.md"
  - "docs/plans/extract-crates/agent-registry.md"
  - "docs/plans/extract-crates/agent-runtime.md"
  - "docs/plans/extract-crates/agent-schema.md"
  - "docs/plans/extract-crates/agent-search.md"
  - "docs/plans/extract-crates/agent-server.md"
  - "docs/plans/extract-crates/agent-stash.md"
  - "docs/plans/extract-crates/agent-store.md"
  - "docs/plans/extract-crates/agent-ui-contracts.md"
last_reviewed: "2026-05-15"
last_modified: "2026-05-15"
modified_on_branch: "gateway-first-skeleton"
modified_at_version: "0.1.0"
modified_at_commit: "d327495"
review_basis: "cross-referenced against gateway-first implementation audit and local docs/references snapshot"
---

# Lab Extraction Bootstrap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bootstrap AgentCast by extracting and generalizing proven Lab runtime, gateway, registry, marketplace, API, CLI, and UI patterns into AgentCast crate boundaries.

**Architecture:** Treat Lab as source material, not as a dependency. Extract only the generic behavior that fits AgentCast's MVP and crate map, then prove each slice with AgentCast-owned tests and fixtures.

**Tech Stack:** Rust 2024 workspace, Lab local repo, Lab Repomix snapshots, MCP, ACP, RMCP, agent-client-protocol, Axum, Clap, rusqlite, Tokio.

---

## Source Of Truth

Use these sources when planning extraction:

- `../lab`: live local Lab repo when available.
- `docs/references/jmagar/jmagar-lab.xml`: Repomix source-of-truth snapshot for Lab at the captured revision.
- `docs/references/jmagar/jmagar-aurora-design-system.xml`: Repomix source-of-truth snapshot for Aurora design system at the captured revision.
- `docs/reports/lab-extraction-source-map.md`: first-party source map from the live `../lab` exploration pass.

When `../lab` and the repopack disagree, inspect the live repo and decide whether the repopack is stale. Do not assume folder names or seed transcript claims are accurate without checking source.

Do not modify `docs/references/` while executing these plans. Reference snapshots, repopacks, and seed transcripts preserve source/provenance text.

## Upstream Reference Checks

- Lab source claims must be checked against `docs/references/jmagar/jmagar-lab.xml`; Aurora UI claims must be checked against `docs/references/jmagar/jmagar-aurora-design-system.xml`.
- MCP launcher claims must be checked against `docs/references/mcp/docs/markdown/0185-modelcontextprotocol-io-docs-learn-architecture.md`, `docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md`, `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`, `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`, `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`, `docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md`, and `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`.
- MCP Registry and install-package claims must be checked against `docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md` and `docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md`; the registry is preview, so plans must not promise GA stability.
- ACP claims must be checked against `docs/references/acp/docs/markdown/0022-agentclientprotocol-com-protocol-overview.md`, `docs/references/acp/docs/markdown/0011-agentclientprotocol-com-protocol-initialization.md`, `docs/references/acp/docs/markdown/0062-agentclientprotocol-com-protocol-prompt-turn.md`, `docs/references/acp/docs/markdown/0028-agentclientprotocol-com-protocol-content.md`, `docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md`, `docs/references/acp/docs/markdown/0032-agentclientprotocol-com-protocol-file-system.md`, `docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md`, `docs/references/acp/docs/markdown/0018-agentclientprotocol-com-libraries-rust.md`, and `docs/references/acp/docs/markdown/0043-agentclientprotocol-com-get-started-registry.md`.
- Claude Code plugin and MCP compatibility claims must be checked against `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`, `docs/references/claude-code/markdown/0093-code-claude-com-docs-en-plugins.md`, `docs/references/claude-code/markdown/0102-code-claude-com-docs-en-plugins-reference.md`, and `docs/references/claude-code/markdown/0108-code-claude-com-docs-en-plugin-marketplaces.md`.
- FastMCP and mcporter are comparison references, not AgentCast dependencies in these plans: use `docs/references/fastmcp/docs/overview.mdx`, `docs/references/fastmcp/docs/client.mdx`, `docs/references/fastmcp/docs/running.mdx`, `docs/references/fastmcp/docs/inspecting.mdx`, `docs/references/fastmcp/docs/generate-cli.mdx`, `docs/references/fastmcp/docs/auth.mdx`, `docs/references/mcporter/docs/spec.md`, `docs/references/mcporter/docs/tool-calling.md`, `docs/references/mcporter/docs/cli-reference.md`, `docs/references/mcporter/docs/mcp.md`, `docs/references/mcporter/docs/agent-skills.md`, and `docs/references/mcporter/docs/livetests.md` when implementing comparable client, CLI, auth, inspection, or live-test behavior.

## Extraction Rule

Extract code and patterns only when they can be generalized without carrying Lab-specific service assumptions.

Allowed to extract/generalize:

- surface-neutral runtime/dispatch patterns.
- MCP gateway upstream routing, filtering, health, and tool search patterns.
- registry and metadata normalization patterns.
- marketplace/install-plan patterns.
- auth primitives when not tied to Lab service policy.
- API/CLI thin-surface patterns.
- static web/admin composition patterns.
- Aurora UI components that fit AgentCast surfaces.
- persistence, logging, and observability helpers when generic.

Do not extract into AgentCast core:

- Plex/Sonarr/Radarr/Unraid/Gotify workflows.
- homelab service SDKs.
- service credential extraction from appdata.
- Lab-only node/fleet policy unless promoted into a post-v0 AgentCast spec.
- UI copy or flows that assume Lab service administration.

## MVP Extraction Priority

The gateway-first pass has already landed a working MCP launcher/gateway path: config discovery, stdio and streamable HTTP upstream clients, catalog/list/search/call behavior, protected public MCP JSON-RPC routes, OAuth probe/begin/callback/status/refresh/clear/register fixtures, API routes, CLI handler surfaces, protected-route/OAuth CLI commands, and a runnable `agentcast` HTTP server.

Continue extraction in this order:

1. execute `agent-registry.md` for registry metadata sync once the gateway's local runtime path remains stable.
2. execute `agent-marketplace.md` for install preview/apply after registry models and store support exist.
3. extend `agent-schema.md` and `agent-core.md` only where a concrete registry, marketplace, config, or API slice needs additional shared validation or cross-crate primitives.
4. extend `agent-observability.md`, `agent-server.md`, and `agent-ui-contracts.md` incrementally as real API/server/UI surfaces require them.

ACP runtime behavior, loadouts, stash workflows, fleet workflows, desktop UI, and remote execution remain post-v0 unless explicitly promoted in `docs/MVP.md`; their contract crates now exist so future work has a clean home.

## Crate Plans

- [agent-acp.md](./agent-acp.md)
- [agent-api.md](./agent-api.md)
- [agent-auth.md](./agent-auth.md)
- [agent-cli.md](./agent-cli.md)
- [agent-config.md](./agent-config.md)
- [agent-core.md](./agent-core.md)
- [agent-gateway.md](./agent-gateway.md)
- [agent-marketplace.md](./agent-marketplace.md)
- [agent-mcp.md](./agent-mcp.md)
- [agent-observability.md](./agent-observability.md)
- [agent-protocol.md](./agent-protocol.md)
- [agent-registry.md](./agent-registry.md)
- [agent-runtime.md](./agent-runtime.md)
- [agent-schema.md](./agent-schema.md)
- [agent-search.md](./agent-search.md)
- [agent-server.md](./agent-server.md)
- [agent-store.md](./agent-store.md)
- [agent-stash.md](./agent-stash.md)
- [agent-fleet.md](./agent-fleet.md)
- [agent-ui-contracts.md](./agent-ui-contracts.md)

## Current Audit

Use this table before executing any individual plan. For partially implemented plans, first verify the listed current state in code, then skip completed historical tasks and continue at the remaining gaps.

| Plan | Current state on `gateway-first-skeleton` | Continue with |
|---|---|---|
| `gateway-first.md` | Operational through the local gateway path, including broad MCP config discovery, durable OAuth storage, metadata discovery, dynamic client registration, authorization-code and refresh-token endpoint exchange, refresh locking/state transitions, reload reconciliation, protected-route credential lookup, runtime credential injection, protected MCP Streamable HTTP session/SSE behavior, AgentCast MCP stdio tools for servers/actions/resources/prompts/search/call/read/get/status, core `agentcast gateway` subcommands, protected-route/OAuth CLI commands, runtime operation timeouts, runtime response/catalog caps, circuit breaker/reprobe behavior, persisted process-state stale-record helpers, runtime shutdown hook, and graceful server shutdown. | Process-group termination remains future runtime hardening unless the MCP SDK exposes stable child process handles. |
| `agent-protocol.md` | Partially complete for gateway action IDs, action models, and MCP-facing DTOs. | Add schema, registry, marketplace, or ACP DTOs only when the owning plan needs a shared contract. |
| `agent-config.md` | Partially complete with MCP JSON/JSONC parsing including line/block JSONC comments, native AgentCast MCP TOML load/validation, discovery for Claude Code, Claude Desktop, Codex, Gemini, Cursor, VS Code, VS Code Insiders, Antigravity, Windsurf, and OpenCode, explicit upstream mutation helpers, and restricted token/secret/key/url/runtime `.env` merge behavior. | Richer import/adoption CLI workflows. |
| `agent-mcp.md` | Partially complete with RMCP stdio and streamable HTTP clients, gateway-facing MCP server tools for servers/actions/resources/prompts/search/call/read/get/status, and tools/resources/prompts/result models. | HTTP fixture tests, response bounds, and transport-specific error mapping. |
| `agent-runtime.md` | Partially complete with upstream startup, catalog snapshots that preserve tool input/output schemas and annotations, call/read/get-prompt routing, streamable HTTP upstreams, subject-scoped bearer injection for HTTP MCP calls, configurable operation timeouts, catalog/response caps, circuit breaker/reprobe behavior, persisted process-state stale-record detection/cleanup helpers, and runtime shutdown. | Process-group termination and direct child reaping remain future hardening until the MCP SDK exposes stable process handles. |
| `agent-gateway.md` | Partially complete with catalog/routing/search documents, exposure allow/deny policy, protected route config/index/CRUD/status/test policy, OAuth orchestration, metadata discovery, dynamic client registration, authorization-code/refresh-token endpoint exchange, refresh lock integration through the API surface, and removed-upstream OAuth reconciliation. | Catalog diff/reload behavior and broader health aggregation. |
| `agent-search.md` | Partially complete with deterministic action indexing and search. | Redaction/truncation, schema summaries, result bounds, and explanation metadata. |
| `agent-auth.md` | Partially complete with bearer/scope helpers, fixture, static, and HS256 JWT/JWKS bearer verifier boundaries, protected-resource metadata, OAuth primitives, and refresh DTOs. | Add asymmetric JWKS algorithms only when a deployment surface requires them. |
| `agent-store.md` | Partially complete with in-memory and SQLite OAuth stores, pending-state persistence, encrypted credential persistence, dynamic OAuth client registration persistence, generic SQLite open/migration harness, owner-only DB permissions, and catalog snapshot persistence. | Install-plan/audit records only when CLI/API surfaces require durable history. |
| `agent-api.md` | Partially complete with gateway action/search/call/server/resource/prompt/status routes, protected MCP, OAuth HTTP routes, protected-route admin CRUD/status/test routes, protected-route credential lookup, protected MCP transport/session validation, authenticated SSE setup, registry search routes that can be backed by cache, marketplace MCP plan/apply routes with env validation, and typed JSON error envelopes. | OpenAPI generation remains surface polish. |
| `agent-cli.md` | Partially complete with gateway handler/view helpers plus binary-level `agentcast gateway` actions/search/call/read-resource/get-prompt/protected-route/oauth, protected-route JSON file list/add/remove/status/test, `registry search`, `marketplace plan-mcp/apply-mcp` subcommands with optional `.env` merge, and human table output for gateway action/search views. | Config import/adoption command polish and broader human output coverage. |
| `agent-server.md` | Partially complete with binary composition, gateway HTTP serving, MCP stdio mode, registry/marketplace command dispatch, and graceful shutdown. Current composition is accepted in `main.rs`; split into `lib.rs`/`startup.rs` only when reuse requires it. | Config path strategy, logging flags, and production serve ergonomics. |
| `agent-core.md` | Implemented with action metadata/category/risk DTOs, protocol-neutral error kinds, stable ID helpers, timestamp helpers, and small JSON helpers. | Add only primitives needed by multiple crates. |
| `agent-schema.md` | Implemented with JSON Schema normalization and payload validation for action/tool schemas. | Extend only for OpenAPI/UI form needs or broader schema coverage. |
| `agent-registry.md` | Partially complete with MCP Registry DTO normalization, status/freshness/provenance metadata, paginated HTTP list client, in-memory fetched-at cache, and deterministic local search. | Durable cache/audit persistence only if marketplace or store execution needs it. |
| `agent-marketplace.md` | Partially complete with install plan vocabulary, stdio and remote HTTP MCP package-to-upstream preview planning, object-level apply through `agent-config`, file-level CLI apply, required/default env resolution, restricted `.env` merge, runtime/env/argv/URL safety checks, and clean v0 MCP-only scope. | MCPB integrity verification and non-MCP package installation remain future work. |
| `agent-observability.md` | Implemented with tracing setup, redaction helpers, activity events, and health DTOs. | Add audit/metrics/log storage only when a real surface requires it. |
| `agent-ui-contracts.md` | Implemented with gateway, registry, marketplace, and invocation view DTOs. | Extend only for stable external/UI contracts. |
| `agent-acp.md` | Implemented with post-v0 adapter contract DTOs for errors, events, permissions, and session commands. | ACP runtime launch, provider supervision, persistence, registry, marketplace, and chat UI remain future work. |
| `agent-stash.md` | Implemented with safe relative paths, item metadata, drift status, revisions, and import/export manifests. | Saved-artifact workflows, persistence wiring, API/CLI/UI surfaces remain future work. |
| `agent-fleet.md` | Implemented with generic node identity, heartbeat/status, capabilities, execution targets, and remote execution request DTOs. | Enrollment, queues, remote execution runtime, API/CLI routes, and Lab fleet policy remain future work. |

## Verification Expectations

Every extracted slice should include:

- source path or repopack evidence from Lab.
- what was kept.
- what was renamed/generalized.
- what Lab-specific behavior was intentionally left behind.
- tests or fixtures proving the AgentCast behavior independently of Lab.

Extraction is not copying blindly. The output must match AgentCast crate boundaries and MVP scope.

## Test Placement Rule

Use source-side unit tests for these extraction plans. Implementation files may declare `#[cfg(test)] mod tests;` when needed, but test bodies belong in source-side sidecar files such as `src/foo/tests.rs`, `src/foo_test.rs`, or `src/foo/tests/*.rs`. Do not create `crates/*/tests/` integration-test files for crate-internal behavior; reserve integration tests only for explicit cross-crate public contracts.

## Work Plan

### Task 1: Confirm Target Scope Before Each Extraction

**Files:**
- Read: `docs/MVP.md`
- Read: `docs/CRATE_BOUNDARIES.md`
- Read: `docs/plans/extract-crates/README.md`
- Read: the relevant `docs/plans/extract-crates/agent-*.md`

- [ ] **Step 1: Read the MVP and crate boundary docs.**

Run:

```bash
sed -n '1,220p' docs/MVP.md
sed -n '1,260p' docs/CRATE_BOUNDARIES.md
```

Expected: the target slice is confirmed as MCP launcher v0 unless the crate plan explicitly marks post-v0 work.

- [ ] **Step 2: List the crate plans and read the one being executed.**

Run:

```bash
find docs/plans/extract-crates -maxdepth 1 -type f -name 'agent-*.md' -print | sort
sed -n '1,260p' docs/plans/extract-crates/agent-acp.md
```

Expected: the implementation boundary, Lab sources, and verification commands are known before editing code. The second command is the concrete ACP example; use the matching listed file for a different crate.

### Task 2: Collect Evidence From Lab

**Files:**
- Read: `../lab`
- Read: `docs/references/jmagar/jmagar-lab.xml`

- [ ] **Step 1: Locate candidate Lab files.**

Run:

```bash
rg --files ../lab | rg 'acp|gateway|marketplace|mcpregistry|config|auth|cli|api|runtime'
```

Expected: candidate source files are identified from the live repo first.

- [ ] **Step 2: Compare against the Lab repopack when live source is unclear.**

Run:

```bash
rg -n 'AcpSessionRegistry|GatewayManager|mcpregistry|AcpEvent|install plan' docs/references/jmagar/jmagar-lab.xml
```

Expected: stale or missing live source is resolved against the captured repopack.

### Task 3: Implement In AgentCast Boundaries

**Files:**
- Modify: `crates/agent-acp/src/lib.rs` for the concrete ACP example.
- Create: `crates/agent-acp/src/events.rs` for the concrete ACP example.
- Test sidecars: `crates/agent-acp/src/{content,events,permissions,provider,session}.rs` for the concrete ACP example.

- [ ] **Step 1: Write the AgentCast-owned tests first.**

Run:

```bash
cargo nextest run -p agent-acp
```

Expected: the new behavior fails before implementation or the existing placeholder crate confirms no matching tests exist yet.

- [ ] **Step 2: Extract the smallest generic implementation.**

Expected: implementation uses AgentCast names, error types, and crate dependencies; Lab-specific service names and homelab behavior do not cross the boundary.

- [ ] **Step 3: Re-run focused verification.**

Run:

```bash
cargo nextest run -p agent-acp
```

Expected: focused tests pass.

### Task 4: Record The Extraction Decision

**Files:**
- Modify: the relevant `docs/plans/extract-crates/agent-*.md`
- Create or modify when behavior becomes stable: `docs/contracts/*.md`
- Create or modify when implementation detail matters: `docs/specs/*.md`

- [ ] **Step 1: Update the crate plan with evidence.**

Expected: the plan lists Lab source paths, kept behavior, renamed/generalized behavior, left-behind behavior, and verification evidence.

- [ ] **Step 2: Promote stable behavior into contracts or specs.**

Expected: contracts contain testable invariants; specs contain implementation details and edge cases.
