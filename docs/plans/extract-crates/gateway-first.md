---
title: "Gateway-First Extraction Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
  - "docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md"
  - "docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md"
related:
  - "docs/MVP.md"
  - "docs/CRATE_BOUNDARIES.md"
  - "docs/contracts/crates-and-dependencies.md"
  - "docs/plans/extract-crates/README.md"
  - "docs/plans/extract-crates/agent-api.md"
  - "docs/plans/extract-crates/agent-auth.md"
  - "docs/plans/extract-crates/agent-cli.md"
  - "docs/plans/extract-crates/agent-config.md"
  - "docs/plans/extract-crates/agent-gateway.md"
  - "docs/plans/extract-crates/agent-mcp.md"
  - "docs/plans/extract-crates/agent-protocol.md"
  - "docs/plans/extract-crates/agent-runtime.md"
  - "docs/plans/extract-crates/agent-search.md"
  - "docs/plans/extract-crates/agent-store.md"
last_reviewed: "2026-05-15"
last_modified: "2026-05-15"
modified_on_branch: "gateway-first-skeleton"
modified_at_version: "0.1.0"
modified_at_commit: "d327495"
review_basis: "live ../lab gateway, MCP, upstream source review, and gateway-first implementation audit"
---

# Gateway-First Extraction Plan

This plan adapts the existing Lab extraction sequence for the case where the gateway is the first user-visible AgentCast capability to extract.

The goal is **not** to port Lab's `GatewayManager` wholesale. In Lab, gateway behavior is tightly coupled to Lab config, built-in homelab services, virtual servers, and the upstream MCP runtime pool. AgentCast should extract a **complete generic gateway** for v0: no final v0 gateway milestone may omit protected public MCP routes or upstream OAuth lifecycle.

```txt
config discovery -> MCP upstream runtime -> normalized catalog -> gateway policy/routing -> protected MCP routes + OAuth -> thin CLI/API/MCP surfaces
```

This updates the earlier gateway-first framing: AgentCast still leaves behind Lab virtual servers and homelab service wiring, but the rest of the generic gateway feature set is in v0 scope.

The first mergeable gateway skeleton is smaller than the v0 completion bar: config discovery, stdio runtime, normalized catalog/list/call, and a CLI path over the shared services. Protected routes and OAuth are v0 requirements, but they should land as later tested slices over that skeleton instead of blocking the first usable runtime.

## Current Implementation Audit

As of 2026-05-15 on `gateway-first-skeleton`, the first usable gateway is no longer only a skeleton. The implemented path includes MCP config discovery for Claude Code, Claude Desktop, Codex, Gemini, Cursor, VS Code, VS Code Insiders, Antigravity, Windsurf, and OpenCode; MCP JSON/JSONC import parsing with line and block comments; RMCP stdio upstreams; streamable HTTP upstream clients; runtime catalog snapshots; tool/resource/prompt listing; gateway exposure allow/deny filtering; action search and invocation; protected public MCP route indexing/auth/metadata/basic JSON-RPC dispatch; protected route CRUD/status/test surfaces; fixture/static bearer token verification boundaries; fixture-backed OAuth probe/begin/callback/status/refresh/clear/register; gateway API routes for actions, servers, resources, prompts, and status; CLI gateway handlers/views and real `agentcast gateway` list/search/call/read-resource/get-prompt/protected-route/oauth subcommands; AgentCast MCP stdio tools for listing servers/actions/resources/prompts, action search/call, resource read, prompt get, and gateway status; and an `agentcast` HTTP server that composes these routes.

When executing this plan now, first verify the corresponding code path and skip historical steps that are already landed. The post-dispatch audit gaps identified on 2026-05-15 have been folded back into this implementation slice; remaining work should come from downstream crate plans, production hardening called out below, or newly discovered defects rather than from this historical skeleton sequence.

## Reviewed Lab Sources

Primary live Lab sources reviewed:

- `../lab/crates/lab/src/dispatch/gateway/discovery.rs`
- `../lab/crates/lab/src/dispatch/gateway/discovery/claude_code.rs`
- `../lab/crates/lab/src/dispatch/gateway/discovery/codex.rs`
- `../lab/crates/lab/src/dispatch/gateway/discovery/gemini.rs`
- `../lab/crates/lab/src/dispatch/gateway/config.rs`
- `../lab/crates/lab/src/dispatch/gateway/manager.rs`
- `../lab/crates/lab/src/dispatch/gateway/index.rs`
- `../lab/crates/lab/src/dispatch/gateway/oauth.rs`
- `../lab/crates/lab/src/dispatch/gateway/oauth_lifecycle.rs`
- `../lab/crates/lab/src/dispatch/gateway/protected_routes.rs`
- `../lab/crates/lab/src/dispatch/gateway/types.rs`
- `../lab/crates/lab/src/dispatch/upstream/types.rs`
- `../lab/crates/lab/src/dispatch/upstream/pool.rs`
- `../lab/crates/lab/src/mcp/server.rs`
- `../lab/crates/lab/src/mcp/upstream.rs`

Evidence note:

- These are live sibling-repo paths, not all checked-in reference snapshot paths. Before implementation, verify each live path exists and map any stale or missing path to `docs/references/jmagar/jmagar-lab.xml`; for example, neighboring AgentCast plans record GatewayManager runtime evidence in `gateway/runtime.rs` when the packed snapshot does not expose `manager.rs`.
- Treat every Lab source as behavior evidence only. Do not copy Lab module layout, singleton manager shape, route names, env var names, service exceptions, or web-admin wording into AgentCast.

Relevant existing AgentCast plans:

- `docs/plans/extract-crates/README.md`
- `docs/plans/extract-crates/agent-gateway.md`
- `docs/plans/extract-crates/agent-config.md`
- `docs/plans/extract-crates/agent-mcp.md`
- `docs/plans/extract-crates/agent-runtime.md`
- `docs/plans/extract-crates/agent-protocol.md`
- `docs/plans/extract-crates/agent-search.md`
- `docs/plans/extract-crates/agent-auth.md`
- `docs/plans/extract-crates/agent-store.md`
- `docs/plans/extract-crates/agent-api.md`

## Key Finding

Lab's gateway is not one separable crate-shaped component. It is a product feature built from several layers:

1. external MCP config discovery.
2. gateway config validation and mutation.
3. MCP upstream process/connection pool.
4. tool/resource/prompt discovery cache.
5. exposure policy and health tracking.
6. tool search and catalog diff notifications.
7. protected public MCP route mapping and OAuth metadata/auth lifecycle.
8. Lab-specific service and virtual-server behavior.
9. API, CLI, web admin, and MCP server surfaces.

For AgentCast, the generic gateway extraction must include layers 1-7 for v0. Layer 8 stays out of v0 because it is Lab/homelab product behavior, not generic gateway behavior.

## Extract First

### MCP config discovery

Source:

- `../lab/crates/lab/src/dispatch/gateway/discovery.rs`
- `../lab/crates/lab/src/dispatch/gateway/discovery/claude_code.rs`
- `../lab/crates/lab/src/dispatch/gateway/discovery/codex.rs`
- `../lab/crates/lab/src/dispatch/gateway/discovery/gemini.rs`

Keep and generalize:

- stable scanner order.
- name-based dedupe with first-seen wins.
- JSONC stripping that preserves URL strings and comment-like text inside strings.
- extraction from `mcpServers`, `servers`, `mcp`, and root fallback where appropriate.
- command-array normalization where the first element is command and remaining elements are args.
- URL key aliases: `url`, `baseUrl`, `base_url`, `serverUrl`, `server_url`.
- imported-source provenance.
- env key counting without copying raw env values.
- imported servers default disabled until explicitly adopted.

Implementation requirements:

- Implement AgentCast config loading first, then one external client scanner at a time, then project config discovery.
- Imported external MCP servers default disabled. Adoption is an explicit CLI/API action; imports must not silently enable a newly discovered command.
- Config previews and imports preserve env var names only. They must not copy raw env values, bearer token values, or token-looking argv/env fragments into stored config, logs, CLI JSON, or API responses.

AgentCast placement:

- `agent-config`: scanners, import provenance, dedupe, AgentCast config load/write.
- `agent-protocol`: stable server/config model names if shared across runtime/gateway/API.

### Upstream config validation and persistence patterns

Source:

- `../lab/crates/lab/src/dispatch/gateway/config.rs`

Keep and generalize:

- atomic write through temp file and persist.
- lock around mutation.
- validation that an upstream has a supported target.
- empty allowlist means clear filter.
- bearer-token env var indirection and validation where auth support is enabled.

Caution:

- Lab rewrites the entire TOML struct and drops unknown keys/comments. AgentCast should decide whether v0 accepts that or uses `toml_edit` for comment-preserving config mutation.
- Phase 1 must make this decision before mutation code lands. Preferred v0 behavior is comment/unknown-key preservation for AgentCast-owned config via `toml_edit`; if whole-file rewrite is chosen, it must be documented as an explicit exception and must never rewrite third-party imported config files.

AgentCast placement:

- `agent-config`: config file and import mutation.
- `agent-gateway`: exposure policy validation that is not config-file-specific.
- `agent-auth`: reusable auth primitives later.

### Upstream runtime and health model

Source:

- `../lab/crates/lab/src/dispatch/upstream/types.rs`
- `../lab/crates/lab/src/dispatch/upstream/pool.rs`
- `../lab/crates/lab/src/dispatch/gateway/runtime.rs`

Keep and generalize:

- process-backed upstream runtime metadata.
- discovered tool/resource/prompt counts.
- per-capability health for tools, resources, and prompts.
- circuit-breaker shape and reprobe interval.
- discovery and request timeouts.
- response size caps.
- stdio process cleanup patterns.
- runtime pool snapshot/caching pattern.

Implementation order:

1. stdio connect/list/call/read with timeout handling and child cleanup.
2. failed-upstream health/status.
3. response size caps.
4. cached discovery/runtime snapshots.
5. circuit breaker and live reprobe semantics only after repeated-failure behavior is defined.

Runtime dependency contract:

- `agent-runtime` exposes upstream snapshots and call/read primitives over AgentCast models.
- `agent-gateway` maps `LauncherActionId` and route policy to those runtime primitives.
- `agent-runtime` must not depend on `agent-gateway`; invocation must not introduce a cycle.

AgentCast placement:

- `agent-mcp`: raw MCP SDK connection, list, call, read, and error normalization wrappers.
- `agent-runtime`: process lifecycle, upstream supervision, catalog cache, and health state.
- `agent-gateway`: only policy/routing decisions over runtime snapshots.

### Gateway catalog policy and routing

Source:

- `../lab/crates/lab/src/dispatch/gateway/manager.rs`
- `../lab/crates/lab/src/dispatch/gateway/types.rs`
- `../lab/crates/lab/src/mcp/server.rs`

Keep and generalize:

- deterministic merge of multiple upstream catalogs.
- route lookup from action/tool id to upstream/tool owner.
- allowlist/wildcard exposure policy.
- user-facing status summaries.

Implementation requirements:

- Stable action IDs must be defined before Phase 3 starts. Default rule: derive from stable server id plus MCP tool name; duplicate human names are allowed only if action IDs remain distinct and collision reports explain the conflicting display names/routes.
- Exposure policy precedence must be explicit: denylist beats allowlist, exact rules beat wildcard rules, and disabled upstreams expose nothing.
- Collision reports must include action id, display name, upstream/server id, original tool name, and the resolution outcome.
- Do not add a generic catalog-diff notification abstraction until an MCP/API surface actually consumes it.

AgentCast placement:

- `agent-gateway`: catalog merge, collisions, exposure policy, routing, gateway health summary.
- `agent-protocol`: `LauncherAction`, invocation request/result, server status DTOs.
- `agent-mcp`: MCP server notification glue later.

### Search indexing

Source:

- `../lab/crates/lab/src/dispatch/gateway/index.rs`

Keep and generalize:

- build index from already-discovered healthy tools/actions.
- bounded max indexed tools.
- simple deterministic scoring by exact name, name substring, token-in-name, and only redacted/bounded description/schema text.
- catalog hash metadata.

Search document contract:

- `agent-gateway` exports bounded `SearchDocument`s containing action id, display name, server id, tool name, short description, schema summary, trust/safety flags, and catalog hash.
- `agent-search` owns query normalization, ranking, and truncation metadata. It must not invoke tools, inspect process state, or depend on runtime internals.
- First implementation should rank exact/name/substring matches before schema-text ranking. Schema text indexing requires explicit truncation and redaction tests.

AgentCast placement:

- `agent-search`: indexing and ranking.
- `agent-gateway`: export search documents, not ranking policy.

### Protected public MCP routes

Source:

- `../lab/crates/lab/src/dispatch/gateway/protected_routes.rs`
- `../lab/crates/lab/src/dispatch/gateway/config.rs`
- `../lab/crates/lab/src/dispatch/gateway/manager.rs`
- `../lab/crates/lab-auth/src/metadata.rs`
- `../lab/crates/lab-auth/src/routes.rs`
- `../lab/crates/lab-auth/src/authorize.rs`
- `../lab/crates/lab-auth/src/token.rs`

Keep and generalize:

- enabled-route index keyed by normalized host and public path.
- first-path-segment route resolution for mounted MCP routes.
- exact `/.well-known/oauth-protected-resource{path}` metadata resolution.
- route validation for public host/path, backend URL/path, scopes, and optional health path.
- public protected-resource metadata for MCP clients.
- route add/update/remove/list/status behavior through gateway APIs.

Implementation requirements:

- Route index, validation, and resolution must be pure `agent-gateway` behavior with no Axum dependency. `agent-api` mounts HTTP routes over that tested policy.
- Protected routes may dispatch to AgentCast-managed upstream MCP servers in v0. Generic backend URL proxying is a separate target shape and must be decided before implementation if promoted.
- Host matching uses `Host` by default. `Forwarded` and `X-Forwarded-Host` are trusted only when a configured trusted-proxy context is present.
- Public paths must reject `/`, `/.well-known`, `/v1`, empty segments, `.`/`..`, duplicate slashes, encoded slash/backslash/dot traversal, query, and fragment.
- Every protected route must enforce auth before proxy/runtime dispatch. Missing or invalid bearer credentials return `401` with `WWW-Authenticate: Bearer ... resource_metadata="..."`; insufficient scope returns `403` with `error="insufficient_scope"` and authoritative `scope`.
- The canonical protected-resource URI for each route must be defined before auth enforcement lands and must be used consistently in metadata, `WWW-Authenticate`, OAuth `resource` parameters, and audience checks.

AgentCast placement:

- `agent-gateway`: protected route config model, route index, route validation, route-to-upstream/backend policy.
- `agent-auth`: generic protected-resource metadata, scope parsing, bearer/OAuth primitives.
- `agent-api`: HTTP route mounting and metadata endpoints.
- `agent-mcp`: no route policy; only MCP transport glue where needed.

### Upstream OAuth lifecycle

Source:

- `../lab/crates/lab/src/dispatch/gateway/oauth.rs`
- `../lab/crates/lab/src/dispatch/gateway/oauth_lifecycle.rs`
- `../lab/crates/lab/src/dispatch/gateway/manager.rs`
- `../lab/crates/lab-auth/src/routes.rs`
- `../lab/crates/lab-auth/src/authorize.rs`
- `../lab/crates/lab-auth/src/token.rs`
- `../lab/crates/lab-auth/src/sqlite.rs`
- `../lab/crates/lab/src/oauth/upstream/**` when present in live Lab.

Keep and generalize:

- upstream OAuth capability probing and metadata discovery.
- begin authorization, complete callback, status, refresh, and clear credential operations.
- per-upstream OAuth manager/cache lifecycle.
- subject-scoped upstream credentials.
- token storage through `agent-store` or an auth-owned store abstraction, not in `agent-gateway` internals.
- status shape that distinguishes connected, expiring, expired, refresh failed, discovery failed, and disconnected.
- reload reconciliation for removed or changed OAuth upstreams.

Security and lifecycle requirements:

- OAuth probes are external-only HTTPS by default and reject userinfo, query, fragment, localhost, loopback, private, link-local, ULA, IPv4-mapped private, CGNAT, and metadata-service addresses after DNS resolution. Tests must cover all resolved addresses and DNS rebinding policy.
- Operator-configured protected-route backend URLs may allow private/self-hosted addresses only behind an explicit policy; they still reject localhost, link-local, metadata IPs, userinfo, query, fragment, and ambiguous paths.
- Authorization uses PKCE S256 and refuses providers whose metadata cannot prove supported code challenge methods.
- OAuth state is subject/upstream scoped, TTL-bound, single-use, and atomically consumed on callback. Invalid or replayed state must not leak code or token details.
- Redirect URI, issuer, resource/audience, and expected authorization server metadata are validated exactly. Inbound client bearer tokens are never passed through to upstream services.
- Persistent OAuth credentials are encrypted at rest if refresh/status/clear are implemented durably: AEAD encryption, fresh nonce per write, AAD bound to upstream + subject + client id, key-required startup validation, restrictive store file permissions, and decryption failure mapped to reauth.
- Refresh uses per-upstream/subject locking to avoid thundering herds and records refresh-failed state without logging raw token material.
- Redaction applies to logs, status views, errors, CLI JSON, API envelopes, test output, and config previews: no access tokens, refresh tokens, auth codes, CSRF state, client secrets, bearer values, URL userinfo, or secret query params.

AgentCast placement:

- `agent-auth`: OAuth metadata, token/credential primitives, callback validation, protected-resource metadata.
- `agent-store`: local encrypted/permissioned credential persistence when persistence is required.
- `agent-gateway`: lifecycle orchestration and upstream association policy.
- `agent-runtime`: applies subject-scoped credentials when connecting to/calling upstream MCP servers.
- `agent-api`: callback and OAuth metadata HTTP routes.

## Leave Behind For v0

Do not extract these into the first AgentCast gateway slice:

- Lab built-in service catalog and service SDK wiring:
  - `../lab/crates/lab/src/dispatch/gateway/service_catalog.rs`
  - `../lab/crates/lab/src/dispatch/gateway/config_mutation.rs`
  - `lab_apis` service clients.
- Lab virtual servers:
  - `../lab/crates/lab/src/dispatch/gateway/virtual_servers.rs`.
- node/fleet/device runtime policy.
- homelab service credentials and `.env` mutation.
- Lab web admin UI flows and Lab-specific copy.
- Lab OAuth env var names, hard-coded upstream names, route/admin naming, or service-specific OAuth exceptions.
- global `GatewayManager` singleton pattern unless AgentCast explicitly chooses it later.

## Gateway-First Implementation Sequence

### Phase -1: Boundary preflight

Before implementation work starts, reconcile the plan against current crate dependencies and open blockers:

- remove or document illegal direct SDK/storage edges under `docs/contracts/crates-and-dependencies.md`.
- `rmcp` belongs only in `agent-mcp`.
- `axum` and `tower` belong only in `agent-api` or top-level server composition.
- `clap` belongs only in `agent-cli`.
- SQLite dependencies belong only in `agent-store`.
- decide stable action ID format, AgentCast config mutation behavior, public route target shape, canonical protected-resource URI, minimal subject identity, and credential-store ownership.

Verification:

```bash
cargo tree -p agent-gateway
cargo tree -p agent-auth
rg -n "rmcp|agent_client_protocol|axum|tower|clap|rusqlite|sqlx" crates/agent-gateway crates/agent-auth crates/agent-runtime crates/agent-config
```

### Phase 0: Minimal shared models

Land only the models needed to make gateway-first work:

- `McpServerId`.
- `McpToolId`.
- `McpResourceUri`.
- `McpResourceTemplate`.
- `McpPromptName`.
- `LauncherActionId`.
- `McpServerConfig`.
- `McpTransportConfig`.
- `DiscoveredMcpServer`.
- `ImportSource`.
- protocol version and negotiated capability models.
- paginated list cursor/page result models.
- resource metadata and prompt metadata models.
- transport kind and streamable HTTP session/header state models.
- minimal subject identity for local CLI/API/MCP flows.
- `LauncherAction`.
- `ToolInvocation`.
- `ToolInvocationResult`.
- `ServerStatus`.

Crates:

- `agent-protocol`
- `agent-core` only for shared errors/IDs if needed.

### Phase 1: Config discovery/import

Implement `agent-config` discovery from Lab patterns for the MVP clients first:

- AgentCast config.
- Claude Code.
- Codex.
- Gemini CLI.
- project MCP config files.

Implement one scanner at a time and keep each scanner fixture-backed before adding the next one.

Acceptance tests:

- JSONC comments are stripped correctly.
- URL strings containing `//` survive stripping.
- `mcpServers` wins over `servers`/`mcp` when present.
- command arrays normalize deterministically.
- raw env values are not copied during import.
- raw bearer-looking values are rejected where an env var name is expected.
- dedupe is stable and first-seen wins.
- imported servers are persisted disabled until adopted.

### Phase 2: MCP adapter + runtime scaffold

Implement the smallest real runtime path before gateway policy:

- `agent-mcp`: stdio connect, initialize, send `notifications/initialized`, list tools/resources/templates/prompts with pagination, call tool, read resource, get prompt.
- `agent-runtime`: start configured stdio upstreams, cache discovery snapshot, expose health/status, stop processes.

Use Lab `UpstreamPool` as behavior reference, but split responsibilities per AgentCast crate boundaries.

Acceptance tests:

- unsupported protocol version disconnects or fails before normal operations.
- negotiated capabilities gate tool/resource/prompt calls.
- paginated `tools/list`, `resources/list`, and `prompts/list` aggregate deterministically with bounded pages.
- fixture stdio MCP server lists tools.
- fixture tool call returns structured output.
- resource read preserves MIME and normalizes text versus binary/blob content with size caps.
- prompt get validates arguments and preserves prompt messages.
- failed upstream updates health.
- timeouts produce actionable errors and cancellation/progress behavior is recorded.
- stdout must contain only newline-delimited JSON-RPC; stderr is captured/logged but is not failure by itself.
- runtime snapshots include negotiated protocol version, server info, declared capabilities, and initialization failure reason.
- resource subscriptions and `listChanged` are not declared or consumed in v0 unless explicitly promoted; refresh/polling is the v0 behavior.

### Phase 3: Gateway catalog/routing

Implement `agent-gateway` over runtime snapshots:

- merge runtime catalog snapshots into `LauncherAction`s.
- preserve MCP tool metadata such as title, icons, annotations, output schema, and task-support hints where available.
- reject or report collisions deterministically.
- apply allowlists/denylists.
- classify untrusted annotations and high-risk tools for later confirmation policy.
- resolve `LauncherActionId` to upstream/tool route.
- return gateway health/status summaries.

Acceptance tests:

- unique tools keep stable action IDs.
- duplicate action IDs with different routes produce a collision report.
- exposure wildcard patterns match as expected.
- denylist/exact/wildcard precedence is deterministic.
- unknown action returns structured `UnknownAction`.

Early CLI slice:

- `agent-cli` can list servers, tools, resources, prompts, and actions, read resources, get prompts, and call actions through the shared runtime/gateway path.
- CLI handlers must not issue raw JSON-RPC methods or duplicate routing/invocation policy.

### Phase 4: Search split

Implement `agent-search` from Lab's lightweight index pattern:

- index gateway-exported search documents.
- rank exact name over substring over token matches.
- include metadata for truncation and catalog hash.

Acceptance tests:

- exact action name ranks first.
- empty query returns no results.
- top-k and truncation are deterministic.
- schema/description redaction and maximum indexed bytes are enforced.

### Phase 5a: Protected route policy

Implement protected route policy without HTTP mounting:

- protected MCP route config, validation, index, and route resolution.
- host extraction policy.
- canonical route resource URI.

Acceptance tests:

- host normalization handles ports, trailing dots, and forwarded host lists.
- route resolution uses host plus first public path segment.
- forwarded-host values are ignored unless trusted proxy mode is configured.
- reserved and ambiguous public paths are rejected.
- route add/update rejects duplicate enabled host/path pairs.

### Phase 5b: Protected-resource metadata and auth enforcement

Implement protected-resource metadata DTOs and exact metadata matching:

- protected-resource metadata resolves exact `/.well-known/oauth-protected-resource{path}` paths.
- `WWW-Authenticate` uses the route resource metadata URI.
- required scopes and insufficient-scope responses are modeled in `agent-auth`.

Acceptance tests:

- missing bearer/JWT returns `401` before route dispatch.
- invalid audience/resource is rejected.
- missing required scope returns `403` with `insufficient_scope`.
- metadata discovery works via `WWW-Authenticate` `resource_metadata` and well-known fallback paths.

### Phase 5c: Public HTTP MCP route mounting

Status: implemented for v0 protected Streamable HTTP behavior. Basic protected route matching, bearer/scope auth, protected-resource metadata, JSON-RPC dispatch, subject-scoped upstream OAuth credential lookup, `initialize` handling, Origin validation, `Accept` handling, protocol-version validation, `MCP-Session-Id` issuance/tracking/deletion, unknown-session rejection, `202` notification handling, authenticated GET `text/event-stream` setup, and `Last-Event-ID` based event replay numbering work through `agent-api`.

Mount protected route behavior in `agent-api` over in-memory gateway/auth fixtures:

- route matching and auth decisions delegate to `agent-gateway` and `agent-auth`.
- streamable HTTP obligations are handled at the API/MCP transport layer: Origin validation, `Accept` handling for `application/json` and `text/event-stream`, `MCP-Protocol-Version` after initialize, optional `MCP-Session-Id`, `202` for notifications/responses where appropriate, and `400`/`404`/`405` for invalid session or method.

### Phase 5d: OAuth probe/status fixtures

Status: implemented for fixture metadata, safe probe validation, authorization begin, callback completion, and status through `agent-api`; live provider token exchange remains out of this completed slice.

Implement fixture-backed upstream OAuth probe and status without durable credentials:

- metadata discovery and SSRF-safe URL validation.
- PKCE/S256 support validation.
- scope selection: use `WWW-Authenticate` scope when present, then protected-resource metadata `scopes_supported`, then omit scope if unavailable.
- disconnected, discovery-failed, and unsupported-provider status states.

Acceptance tests:

- OAuth probes reject unsafe schemes, userinfo, query, fragment, private/loopback/link-local/metadata addresses, and DNS rebinding cases.
- default tests use fixture metadata and no live OAuth provider or network.

### Phase 5e: Credential persistence and callback completion

Status: partially implemented. Callback completion now works against either the in-memory OAuth store or the SQLite OAuth store with AES-GCM encrypted access/refresh tokens and one-time pending-state consumption. Dynamic client registration persistence remains.

Implement durable OAuth state and credential persistence:

- `agent-store` owns SQLite migrations, restrictive file permissions, credential rows, pending auth state, and dynamic client registration rows if used.
- `agent-auth` owns token/credential domain types, callback validation, PKCE verifier/state validation, and encryption helpers or an auth-facing repository trait.
- `agent-gateway` orchestrates upstream association and status.

Acceptance tests:

- callback consumes state atomically once.
- replayed/expired/wrong-subject/wrong-upstream state fails without leaking secrets.
- encrypted credential round trip uses fresh nonce and AAD.
- clear deletes credentials, pending auth state, and dynamic registrations for the upstream/subject scope.

### Phase 5f: Refresh, clear, and runtime credential injection

Status: implemented for the v0 generic lifecycle. Begin, callback, status, refresh, clear, refresh-failed state transitions, subject-scoped credential lookup for protected MCP routes, runtime credential injection primitives, metadata discovery, dynamic client registration with persisted client credentials, authorization-code token exchange, refresh-token exchange against an OAuth token endpoint, API-level per-subject/upstream refresh locking, and reload reconciliation for removed upstream OAuth state exist over the shared gateway/store/runtime/API path.

Complete the OAuth lifecycle before declaring the gateway complete:

- begin authorization, callback-complete, status, refresh, and clear operations.
- subject-scoped credential lookup for upstream runtime calls.
- per-upstream/subject refresh locks and refresh-failed status.
- reload reconciliation for changed/removed OAuth upstreams and protected routes.

Acceptance tests:

- OAuth status distinguishes disconnected, connected, expiring, expired, refresh failed, and discovery failed.
- near-expiry credentials refresh once under concurrent callers.
- removing an OAuth upstream evicts or disables its manager/cache state.
- inbound client bearer tokens are not passed through to upstream services.
- status/errors/logs are redacted.

### Phase 6: Thin surfaces

Status: partially implemented. API routes, server composition, gateway CLI handler/view helpers, full Clap subcommands, and AgentCast MCP server tools exist for the v0 gateway surfaces.

After the shared path works, expose it through surfaces:

- `agent-cli`: list/import servers, list/read resources, list/get prompts, list actions, call action, protected route CRUD/status/test, OAuth probe/authorize/status/refresh/clear.
- AgentCast MCP server entrypoint: expose action list/search/call tools, server/resource/prompt tools, protected route status tools, and gateway status tools.
- `agent-api`: gateway, protected route, OAuth metadata, callback, and upstream OAuth lifecycle routes.

Surfaces must call runtime/gateway/auth services and must not reimplement discovery, routing, invocation, protected-route matching, OAuth lifecycle, or raw JSON-RPC method logic.

Registry and marketplace work stays out of this gateway-first slice until the MVP build order reaches `agent-registry` and `agent-marketplace`.

## Existing Plan Adjustments

The current `agent-gateway.md` plan is still useful for catalog merge, projection, routing, and search-document export. It is not sufficient as the first extraction step because it assumes protocol/runtime/MCP/config models already exist.

Use this gateway-first plan as an overlay:

- Do **not** begin by porting `GatewayManager`.
- Begin with gateway-adjacent config discovery because it is the least coupled and immediately supports the gateway workflow.
- Pull just enough `agent-protocol`, `agent-mcp`, and `agent-runtime` forward to avoid putting lifecycle or SDK code inside `agent-gateway`.
- Then execute the focused `agent-gateway.md` catalog/routing tasks and extend them with protected-route and OAuth lifecycle tasks; those are v0 gateway requirements, not optional post-v0 extras.
- Update `agent-gateway.md`, `agent-auth.md`, and `agent-api.md` before implementation if they still say OAuth/protected routes are deferred. This plan supersedes that older scope statement for v0.

## Blocking Decisions Before Code

- Confirm the stable action ID format and collision report schema.
- Confirm `toml_edit` comment/unknown-key preservation or document a whole-file rewrite exception.
- Confirm protected route target shape for v0: AgentCast-managed upstreams only, generic backend URLs, or both.
- Confirm canonical protected-resource URI derivation for public routes.
- Confirm minimal subject identity for CLI/API/MCP OAuth flows.
- Confirm whether credential encryption helpers live in `agent-auth` over `agent-store`, or in an auth-facing store repository trait.

## Open Decisions

- Which extra client scanners beyond Claude Code, Codex, Gemini, and project MCP config belong in v0.
- Whether gateway catalog changes should expose a generic notification trait later when the MCP/API surfaces need it.
- Whether resource subscriptions/listChanged move into v0 after the refresh/polling path works.

## Verification Expectations

For every phase:

- record the Lab source paths used as evidence.
- state what was kept, generalized, renamed, or left behind.
- use AgentCast-owned tests and fixtures.
- keep default tests offline.
- run focused crate tests first, then `cargo xtask verify` before declaring a substantial extraction complete.
- run dependency-boundary checks for forbidden SDK/storage imports outside owner crates.
- verify `agent-runtime` and `agent-gateway` dependency direction remains one-way.
- verify CLI/API/MCP surfaces do not duplicate discovery, routing, invocation, protected-route matching, OAuth lifecycle, or JSON-RPC method logic.
- include negative security fixture tests for OAuth URL validation, DNS/private IP rejection, callback replay, token redaction, duplicate route rejection, invalid audience/resource, missing/insufficient scope, encrypted credential storage, and offline-only OAuth fixtures.
