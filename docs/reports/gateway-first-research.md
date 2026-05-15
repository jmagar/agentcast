---
title: "Gateway-First Extraction Research"
doc_type: "report"
status: "active"
owner: "agentcast"
audience:
  - "maintainers"
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
  - "docs/plans/extract-crates/gateway-first.md"
  - "docs/plans/extract-crates/agent-gateway.md"
  - "docs/plans/extract-crates/agent-auth.md"
  - "docs/plans/extract-crates/agent-api.md"
  - "docs/plans/extract-crates/agent-runtime.md"
  - "docs/plans/extract-crates/agent-mcp.md"
last_reviewed: "2026-05-14"
last_modified: "2026-05-14"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "1290747"
review_basis: "live ../lab gateway protected-route and OAuth source review"
---

# Gateway-First Extraction Research

This report records the research pass behind `docs/plans/extract-crates/gateway-first.md` after the v0 scope correction: **AgentCast v0 must implement the complete generic gateway**, including protected public MCP routes and upstream OAuth lifecycle. Only Lab virtual servers, built-in homelab service wiring, node/fleet behavior, service credential mutation, and Lab UI copy stay behind.

The v0 completion bar is not the same as the first mergeable implementation slice. The first gateway skeleton can stop at config discovery, stdio runtime, catalog/list/call, and CLI validation; protected routes and OAuth still remain mandatory before the v0 gateway can be called complete.

This report follows the requested Lavra-style research split. The AgentCast workspace currently includes `fastmcp-client-cli` and `refresh-docs` skills, but not checked-in `lavra-plan` or `lavra-research` skill definitions, so the research output is captured as an AgentCast authored report.

## Research Question

What parts of Lab's gateway are generic gateway functionality that should be extracted for AgentCast v0, and what parts are Lab/homelab product behavior that must not be carried over?

## Decision Summary

1. Protected public MCP routes are v0 gateway scope.
2. Upstream OAuth probe, authorization, callback completion, status, refresh, and credential clearing are v0 gateway scope.
3. OAuth/protected-route implementation must be split across AgentCast crate boundaries instead of copied as Lab's `GatewayManager` coupling.
4. Lab virtual servers and homelab service exposure remain out of v0.
5. No AgentCast milestone should call the gateway complete until catalog/routing, search, protected routes, OAuth lifecycle, and thin surfaces are implemented over the same runtime path.
6. Do not recreate Lab's `GatewayManager`, global singleton, web admin workflow, service catalog, route/admin naming, or OAuth env var/upstream exceptions.

## Source Map

| Capability | Lab evidence | Extract into AgentCast | Notes |
|---|---|---|---|
| MCP config discovery | `../lab/crates/lab/src/dispatch/gateway/discovery.rs`, `discovery/claude_code.rs`, `discovery/codex.rs`, `discovery/gemini.rs` | `agent-config`, shared DTOs in `agent-protocol` as needed | Keep JSONC handling, stable dedupe, import provenance, env key counting without secret values. |
| Upstream MCP runtime | `../lab/crates/lab/src/dispatch/upstream/pool.rs`, `../lab/crates/lab/src/dispatch/upstream/types.rs` | `agent-mcp`, `agent-runtime`, `agent-gateway` | Split SDK/client behavior from process lifecycle and gateway policy. |
| Catalog merge/routing | `../lab/crates/lab/src/dispatch/gateway/manager.rs`, `../lab/crates/lab/src/dispatch/gateway/types.rs`, `../lab/crates/lab/src/mcp/server.rs` | `agent-gateway`, `agent-protocol` | Do not port manager wholesale; extract route/collision/exposure behavior. |
| Search | `../lab/crates/lab/src/dispatch/gateway/index.rs` | `agent-search` | Gateway exports documents; search crate owns ranking/indexing. |
| Protected public MCP routes | `../lab/crates/lab/src/dispatch/gateway/protected_routes.rs`, `../lab/crates/lab/src/dispatch/gateway/config.rs`, `../lab/crates/lab/src/dispatch/gateway/manager.rs`, `../lab/crates/lab-auth/src/metadata.rs`, `../lab/crates/lab-auth/src/routes.rs` | `agent-gateway`, `agent-auth`, `agent-api` | In v0 scope. Keep route index, validation, protected-resource metadata, route CRUD/status. |
| Upstream OAuth lifecycle | `../lab/crates/lab/src/dispatch/gateway/oauth.rs`, `../lab/crates/lab/src/dispatch/gateway/oauth_lifecycle.rs`, `../lab/crates/lab/src/oauth/upstream/cache.rs`, `encryption.rs`, `manager.rs`, `refresh.rs`, `store.rs`, `types.rs`, `../lab/crates/lab-auth/src/authorize.rs`, `token.rs`, `sqlite.rs` | `agent-auth`, `agent-store`, `agent-gateway`, `agent-runtime`, `agent-api` | In v0 scope. Keep probe/begin/callback/status/refresh/clear and subject-scoped credential flow. |
| Virtual servers | `../lab/crates/lab/src/dispatch/gateway/virtual_servers.rs`, `service_catalog.rs`, `config_mutation.rs` | Do not extract for v0 | Homelab/Lab-specific service virtualization. |

Evidence confidence note: `../lab` paths are live sibling-repo evidence and may drift from `docs/references/jmagar/jmagar-lab.xml`. Implementation tasks should verify live paths and map any missing files back to the packed reference snapshot before citing or porting behavior.

OAuth should be tracked as separate extraction concerns rather than one broad row:

- metadata/probe and provider capability discovery.
- authorization begin/callback with PKCE/state handling.
- credential persistence and encryption.
- refresh/clear/status.
- runtime credential injection.

## MCP Spec Delta

The Lab source map identifies generic gateway behavior, but MCP conformance adds requirements that must shape the plan:

- MCP runtime must perform `initialize` before normal requests, send `notifications/initialized`, negotiate protocol version, record server info/capabilities, and gate tool/resource/prompt calls on declared capabilities.
- `tools/list`, `resources/list`, and `prompts/list` must handle pagination and bounded aggregation; resources also need read size limits, MIME preservation, and text versus binary/blob normalization.
- Prompt support should include `prompts/get` argument validation if prompts are listed in v0 surfaces.
- Stdio transport tests should require stdout to contain only newline-delimited JSON-RPC while stderr is captured for diagnostics without becoming failure by itself.
- Public HTTP MCP routes are not just path matching. Once exposed, they must satisfy streamable HTTP concerns such as Origin validation, `Accept` negotiation, `MCP-Protocol-Version`, optional `MCP-Session-Id`, and proper 202/400/404/405 handling.
- Protected routes must emit OAuth protected-resource metadata and use `WWW-Authenticate` bearer challenges with `resource_metadata`; insufficient scope should return 403 with `insufficient_scope` and authoritative scope.
- OAuth authorization must use PKCE S256, verify supported code challenge methods, use the OAuth `resource` parameter, and never pass inbound client tokens through to upstream services.

## Protected Route Findings

Lab's protected route model is generic enough to extract once stripped of Lab naming:

- `ProtectedRouteIndex` stores enabled routes by normalized host plus public path.
- host normalization handles comma-separated forwarded host values, ports, trailing dots, whitespace, and lowercasing.
- normal MCP route matching resolves by host plus the first public path segment.
- metadata matching resolves exact `/.well-known/oauth-protected-resource{path}` requests.
- manager methods cover list/get/add/update/remove/test.
- route validation rejects duplicate enabled host/path pairs and returns structured errors.
- protected routes carry scopes and backend mapping information.

AgentCast should keep this as a first-class gateway feature, not a later auth/API side quest.

Acceptance boundary:

- Unit-test route indexing, normalization, duplicate detection, and reserved-path rejection without Axum.
- Treat forwarded host values as untrusted unless a trusted-proxy configuration enables them.
- API tests should only verify HTTP extraction/response mapping over the already-tested gateway/auth policy.
- Protected route dispatch must be behind auth enforcement: missing/invalid bearer returns 401 with route-specific `WWW-Authenticate`, invalid audience/resource is rejected, and missing route scopes return 403 with `insufficient_scope`.

### Protected Route Crate Split

- `agent-gateway`: route config, validation, index, route-to-upstream/backend mapping policy.
- `agent-auth`: protected-resource metadata DTOs, scope parsing, generic auth errors.
- `agent-api`: public HTTP endpoints, metadata routes, request extraction, response mapping.
- `agent-runtime`: only if a route maps directly to an AgentCast-managed upstream; no HTTP route definitions.

## Upstream OAuth Findings

Lab's upstream OAuth lifecycle is broader than a metadata helper. It includes:

- URL validation for OAuth probing.
- upstream-name validation and deterministic transient probe manager keys.
- SSRF-style URL validation before probing remote metadata.
- OAuth metadata discovery via RMCP authorization manager behavior.
- strategy selection between dynamic registration and client metadata document where supported.
- prerequisite validation for encryption key, SQLite credential store, and redirect URI.
- begin authorization.
- complete authorization callback.
- status view with connected/expiring/expired/refresh-failed/discovery-failed/disconnected states.
- credential clearing.
- reload reconciliation that evicts OAuth client cache entries and removes managers for deleted OAuth upstreams.
- subject-scoped credentials for upstream calls.

AgentCast should extract the lifecycle semantics, not Lab's environment names or service-specific exceptions.

OAuth implementation should be staged. Fixture-backed probe/status should land before dynamic registration, durable credential storage, refresh, and runtime injection. Default tests must use fixture metadata, fake token storage, no live OAuth provider, no network, and no real secrets.

Unresolved prerequisites before credential work:

- encryption key policy and startup validation.
- redirect URI derivation.
- minimal subject identity for CLI/API/MCP flows.
- credential-store ownership between `agent-auth` and `agent-store`.
- canonical protected-resource URI and OAuth `resource` parameter derivation.

### OAuth Crate Split

- `agent-auth`: OAuth metadata models, authorization flow primitives, callback validation, token/credential domain types.
- `agent-store`: local credential persistence and migrations if persistence is needed for v0. Keep secrets out of config files.
- `agent-gateway`: user-facing lifecycle orchestration and association between upstreams/routes and OAuth state.
- `agent-runtime`: inject subject-scoped credentials into upstream MCP calls.
- `agent-api`: callback routes, metadata routes, and lifecycle HTTP handlers.
- `agent-cli`: probe/authorize/status/clear commands as thin surface wrappers.

## What Must Not Be Extracted

These are not generic gateway requirements for AgentCast v0:

- Lab virtual servers and service-backed virtual MCP servers.
- `lab_apis` service catalog exposure.
- homelab service credential discovery or `.env` mutation.
- node/fleet/device runtime policy.
- Lab gateway-admin UI copy and workflows.
- Lab-specific OAuth exceptions such as hard-coded upstream names.
- Lab OAuth env var names.
- Lab admin API/UI route naming.
- global manager singleton pattern unless later documented as a conscious AgentCast decision.

## Implementation Risks

### Boundary Risk

Protected routes and OAuth could easily pull `axum`, `rmcp`, SQLite, and gateway policy into the same crate. Avoid this by keeping:

- HTTP route definitions in `agent-api`.
- RMCP SDK types in `agent-mcp`.
- credential persistence in `agent-store` or behind auth-owned traits.
- orchestration policy in `agent-gateway`.

### Security Risk

Gateway v0 now includes public route and OAuth surfaces. Tests and implementation must cover:

- URL validation and SSRF-style restrictions for probes and backend URLs.
- trust-class-specific URL policy: OAuth probes are external-only HTTPS; configured route backends may allow private/self-hosted addresses only by explicit policy and still reject localhost/link-local/metadata/userinfo/query/fragment/ambiguous paths.
- DNS rebinding behavior for all resolved addresses.
- PKCE S256, state/CSRF single-use TTL-bound callback storage, exact redirect URI validation, and issuer/resource/audience validation.
- encrypted credential persistence with fresh nonce, AAD binding to upstream plus subject plus client id, restrictive store permissions, and reauth on decryption failure.
- refresh locking and refresh-failed status without thundering herds.
- no secrets in config imports or previews.
- token storage outside plain config files.
- redacted logs, status views, errors, CLI JSON, API envelopes, test output, and config previews.
- scope validation and protected-resource metadata correctness.
- no live OAuth provider requirement in default tests.

### Incremental Delivery Risk

Bundling protected routes, metadata, auth enforcement, API mounting, OAuth probing, credential persistence, refresh, and runtime credential injection into one phase creates a high-risk merge. Each part should land behind a tested internal contract:

- route policy without HTTP.
- metadata/auth DTOs without runtime dispatch.
- API route mounting over in-memory fixtures.
- OAuth probe/status with fixture metadata and no stored credentials.
- encrypted credential persistence and callback completion.
- refresh/clear and runtime credential injection.

### Boundary Verification Risk

The corrected v0 scope crosses several crates. Implementation must continuously verify the dependency contract:

- `rmcp` remains in `agent-mcp`.
- `axum`/`tower` remain in `agent-api` or server composition.
- `clap` remains in `agent-cli`.
- SQLite remains in `agent-store`.
- `agent-gateway` depends on runtime/auth/store-facing traits and AgentCast models, not protocol SDKs or HTTP route types.
- surfaces never duplicate JSON-RPC method handling, route matching, OAuth lifecycle, or runtime invocation policy.

### Completion Risk

A gateway that only lists/calls tools is not complete under the corrected scope. Completion requires:

- config discovery/import.
- MCP upstream runtime.
- catalog merge and action routing.
- search/indexing.
- protected route CRUD/resolution/metadata.
- upstream OAuth probe/authorize/callback/status/refresh/clear.
- CLI/API/MCP thin surfaces over shared gateway/runtime/auth paths.

## Research Output Applied

This research updates `docs/plans/extract-crates/gateway-first.md` by:

- moving protected public MCP routes and upstream OAuth lifecycle into v0 extraction scope.
- removing those features from the leave-behind list.
- adding source evidence and crate-placement guidance.
- adding a dedicated implementation phase and acceptance tests for protected routes and OAuth.
