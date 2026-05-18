# Phase 1: Code Quality and Architecture

## Findings

- High - `crates/agent-server/src/main.rs:1`
  The binary composition file is 1,457 lines and contains CLI definitions, command dispatch, config loading, OAuth store wiring, gateway MCP backend projection, router composition, and inline tests. This violates the hard review threshold in `docs/CODE_ORGANIZATION.md` and makes the top-level binary own too many independent reasons to change.
  Fix by splitting into narrow modules such as `cli.rs`, `commands/gateway.rs`, `commands/oauth.rs`, `commands/marketplace.rs`, `server.rs`, `oauth_store.rs`, and moving inline tests into source-side sidecars.

- High - `crates/agent-api/src/http.rs:1`
  The HTTP route module is 1,339 lines and mixes gateway routes, OAuth routes, marketplace routes, protected-route admin, protected MCP transport handling, DTOs, transport helpers, refresh locking, and error rendering. It exceeds the hard review threshold and has multiple ownership concerns in one file.
  Fix by splitting into route families (`gateway_http`, `oauth_http`, `marketplace_http`, `protected_routes_http`, `protected_mcp_http`) with shared response/error helpers.

- High - `crates/agent-api/Cargo.toml:15`
  `agent-api` depends directly on `agent-mcp` even though `docs/contracts/crates-and-dependencies.md` says surface crates must not invoke protocol SDKs directly and should call runtime/domain crates for decisions. Even if the dependency is currently incidental, it weakens the documented surface -> runtime -> adapter direction.
  Fix by removing `agent-mcp` from `agent-api` or documenting a narrow exception if a DTO truly requires it.

- Medium - `crates/agent-runtime/src/mcp_runtime.rs:74`
  `McpRuntime::start_with_options` starts upstreams sequentially. A single slow or timed-out MCP server delays all later servers during startup, which conflicts with the v0 target of fast local setup and makes one bad config affect unrelated configured servers.
  Fix by starting independent upstreams concurrently with a bounded join set, preserving deterministic final ordering when inserting snapshots.

- Medium - `crates/agent-runtime/src/mcp_runtime.rs:95`
  The runtime owns connection startup, catalog discovery, circuit breaker state, per-request ephemeral HTTP auth, response-size enforcement, reprobe, and shutdown in one module. At 624 lines, it crosses the documented-exception threshold and hides several policies that will evolve independently.
  Fix by extracting connection startup/discovery, operation dispatch, circuit breaker state, and response bounding into focused modules.

- Medium - `crates/agent-store/src/oauth.rs:187`
  `SqliteOAuthStore` is a synchronous `rusqlite::Connection` implementation used behind async HTTP handlers that hold a `tokio::sync::Mutex`. This is workable for a prototype, but the store abstraction does not make the blocking boundary explicit.
  Fix by either moving store calls behind `spawn_blocking` at the HTTP boundary or documenting that OAuth store calls are intentionally small and local-only for v0.

- Medium - `crates/agent-marketplace/src/mcp.rs:172`
  `apply_install_plan_to_config` reconstructs behavior from JSON `preview` fields inside install steps. That makes a display/preview representation part of the mutation contract, so display-shape drift can break installs.
  Fix by adding typed payloads to install steps or passing an internal install action enum through planning and application.

- Low - `crates/agent-server/src/main.rs:1105`
  The binary file contains substantial inline test bodies, contrary to the repo's source-side sidecar testing contract. This increases noise in the most central composition file.
  Fix by moving tests to `crates/agent-server/src/main/tests.rs` or equivalent sidecar modules.

## Positive Notes

- The workspace has clear crate boundaries documented in `docs/CRATE_BOUNDARIES.md` and most crates use source-side `#[cfg(test)] mod tests;` sidecars.
- OAuth credential persistence uses explicit AES-GCM encryption and owner-only SQLite file permissions in `agent-store`.
- Runtime operations already have timeouts, response-size bounding, and a basic circuit breaker.

## Critical Issues for Phase 2 Context

- `agent-api` exposes several route families without visible top-level authentication middleware; Phase 2 should review which routes are intended to be public and whether defaults are safe.
- Protected MCP auth uses a fixture verifier by default; Phase 2 should assess whether production composition can accidentally use fixture tokens.
- The runtime keeps MCP clients in-process and shutdown currently clears the map; Phase 2 should inspect whether child processes and HTTP clients are actually terminated.
