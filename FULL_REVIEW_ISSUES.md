# Consolidated Full Review Issues

Source artifacts:

- `.full-review/00-scope.md`
- `.full-review/01-quality-architecture.md`
- `.full-review/02-security-performance.md`
- `.full-review/03-testing-documentation.md`
- `.full-review/04-best-practices.md`
- `.full-review/05-final-report.md`

This document consolidates every distinct issue surfaced across the full review artifacts. Duplicate mentions across phase reports and the final report are merged under the same root issue.

## Critical

### 1. Protected MCP router can default to fixture bearer auth

- Source: `.full-review/02-security-performance.md`, `.full-review/03-testing-documentation.md`, `.full-review/04-best-practices.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-api/src/protected_mcp.rs:21`, `crates/agent-api/src/protected_mcp/tests.rs:9`
- Issue: `ProtectedMcpRouteApi::new` defaults to `FixtureBearerTokenVerifier`, and production router helpers can reach that constructor. Tests also exercise this fixture verifier as the normal path.
- Impact: A deployed protected MCP router that does not explicitly inject a real verifier can accept fixture-form headers such as `Bearer sub=...;aud=...;scope=...`.
- Fix: Require explicit verifier injection in production constructors, gate fixture verifiers behind `#[cfg(test)]` or a fixture-only feature, and add tests proving fixture-shaped bearer headers fail unless a test-only fixture verifier is selected.

## High

### 2. JWT bearer verification lacks freshness and issuer validation

- Source: `.full-review/02-security-performance.md`, `.full-review/03-testing-documentation.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-auth/src/bearer.rs:255`, `crates/agent-auth/src/bearer/tests.rs:117`
- Issue: JWT verification checks HS256 signature and audience, but does not validate `exp`, `nbf`, or `iss`. Tests do not cover expired tokens, missing expiration, not-yet-valid tokens, or issuer mismatch.
- Impact: Validly signed stale or revoked credentials can continue authorizing protected MCP routes indefinitely.
- Fix: Add issuer and time-claim validation with configurable clock skew, fail closed when `exp` is missing for protected-route JWT mode, and add negative tests for expired, missing, not-yet-valid, and wrong-issuer claims.

### 3. Origin validation is derived from untrusted request headers

- Source: `.full-review/02-security-performance.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-api/src/http.rs:903`
- Issue: Origin validation compares `Origin` to `public_origin(headers)`, which is derived from request `Host` and `x-forwarded-proto`.
- Impact: Without trusted-proxy configuration, a client can forge headers so the computed public origin matches a forged `Origin`.
- Fix: Configure allowed public origins or trusted proxy behavior explicitly, and validate `Origin` against that configured set.

### 4. Mutating/admin route families are unauthenticated by construction

- Source: `.full-review/02-security-performance.md`, `.full-review/03-testing-documentation.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-api/src/http.rs:60`, `crates/agent-api/src/http/tests.rs:223`
- Issue: `oauth_router`, `marketplace_router`, `registry_router`, and protected-route admin router are exposed as unauthenticated routers by construction. Tests cover CRUD behavior but not authorization or local-only composition.
- Impact: Accidental network exposure could allow unauthorized OAuth state manipulation, marketplace install operations against supplied config bodies, or protected-route configuration changes.
- Fix: Require an admin/auth layer or local-bind guard for mutating route families, and add tests or docs-backed composition checks proving these routers are only mounted in local/admin contexts.

### 5. `agent-server` composition file exceeds code organization limits

- Source: `.full-review/01-quality-architecture.md`, `.full-review/03-testing-documentation.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-server/src/main.rs:1`, `crates/agent-server/src/main.rs:1105`
- Issue: The binary composition file is 1,457 lines and mixes CLI definitions, command dispatch, config loading, OAuth store wiring, gateway MCP backend projection, router composition, and inline tests.
- Impact: The file violates the hard review threshold in `docs/CODE_ORGANIZATION.md`, has too many reasons to change, and makes command/router work harder to review.
- Fix: Split into narrow modules such as `cli.rs`, `commands/gateway.rs`, `commands/oauth.rs`, `commands/marketplace.rs`, `server.rs`, and `oauth_store.rs`; move test bodies into source-side sidecars.

### 6. `agent-api` HTTP module exceeds code organization limits

- Source: `.full-review/01-quality-architecture.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-api/src/http.rs:1`
- Issue: The HTTP route module is 1,339 lines and combines gateway routes, OAuth routes, marketplace routes, protected-route admin, protected MCP transport handling, DTOs, transport helpers, refresh locking, and error rendering.
- Impact: The file exceeds the hard review threshold and mixes several independent ownership concerns.
- Fix: Split by route family, for example `gateway_http`, `oauth_http`, `marketplace_http`, `protected_routes_http`, and `protected_mcp_http`, with shared response/error helpers.

### 7. `agent-api` depends directly on `agent-mcp`

- Source: `.full-review/01-quality-architecture.md`, `.full-review/04-best-practices.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-api/Cargo.toml:15`
- Issue: `agent-api` has a direct dependency on `agent-mcp`, despite the documented surface -> runtime -> adapter dependency direction.
- Impact: Surface code can drift toward protocol SDK ownership and weaken crate-boundary enforcement.
- Fix: Remove the dependency or document a narrow exception if a DTO truly requires it; add an automated boundary check so this class of drift is not manual-only.

### 8. Documented verification gates are not reproducible in the current environment

- Source: `.full-review/03-testing-documentation.md`, `.full-review/04-best-practices.md`, `.full-review/05-final-report.md`, `.full-review/00-scope.md`
- Location: `xtask/src/task.rs:72`
- Issue: `cargo xtask ci` expects `nextest-ci` and `deny`, and docs expect `cargo-nextest`, `cargo-deny`, and `taplo`, but the visible direct cargo path used during review lacks those tools. Normal cargo also hit Snap/sccache wrapper failures in this environment.
- Impact: The documented pre-push and completion gates cannot be reproduced as written.
- Fix: Repair local toolchain wiring or document the direct cargo workaround; make `cargo xtask doctor` fail clearly for missing subcommands before verification.

## Medium

### 9. Runtime upstream startup is sequential

- Source: `.full-review/01-quality-architecture.md`, `.full-review/02-security-performance.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-runtime/src/mcp_runtime.rs:74`
- Issue: `McpRuntime::start_with_options` starts independent upstreams sequentially.
- Impact: A single slow or timed-out MCP server delays all later servers and makes cold start scale linearly.
- Fix: Start independent upstreams concurrently with a bounded join set while preserving deterministic final snapshot ordering.

### 10. Runtime module owns too many policies

- Source: `.full-review/01-quality-architecture.md`
- Location: `crates/agent-runtime/src/mcp_runtime.rs:95`
- Issue: `mcp_runtime.rs` owns connection startup, catalog discovery, circuit breaker state, per-request ephemeral HTTP auth, response-size enforcement, reprobe, and shutdown in one 624-line module.
- Impact: The module crosses the documented-exception threshold and hides policies that will evolve independently.
- Fix: Extract connection startup/discovery, operation dispatch, circuit breaker state, and response bounding into focused modules.

### 11. Synchronous SQLite store is used behind async HTTP handlers without an explicit blocking boundary

- Source: `.full-review/01-quality-architecture.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-store/src/oauth.rs:187`
- Issue: `SqliteOAuthStore` uses synchronous `rusqlite::Connection` behind async HTTP handlers guarded by `tokio::sync::Mutex`.
- Impact: Blocking database work can occupy async executor threads, even if the current local-only workload is small.
- Fix: Move store calls behind `spawn_blocking` at the HTTP boundary or document the local-only v0 assumption explicitly.

### 12. Marketplace install apply reconstructs behavior from JSON preview fields

- Source: `.full-review/01-quality-architecture.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-marketplace/src/mcp.rs:172`
- Issue: `apply_install_plan_to_config` reconstructs mutation behavior from display-oriented JSON `preview` fields.
- Impact: Display-shape drift can break install behavior because preview data becomes part of the mutation contract.
- Fix: Add typed payloads to install steps or pass an internal install action enum through planning and application.

### 13. Runtime shutdown does not retain explicit cancellation or process handles

- Source: `.full-review/02-security-performance.md`, `.full-review/03-testing-documentation.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-runtime/src/mcp_runtime.rs:208`, `crates/agent-runtime/src/mcp_runtime/tests.rs:1`
- Issue: `McpRuntime::shutdown` clears the upstream map and relies on transport drop semantics. Client options create cancellation tokens that are not retained. Tests do not prove stdio child process shutdown.
- Impact: Stale MCP child processes or HTTP sessions can survive runtime shutdown if drop behavior is insufficient.
- Fix: Store cancellation tokens or explicit process/session handles per upstream, cancel them during shutdown, and add process-lifecycle tests.

### 14. Protected streamable HTTP calls create a new MCP client per operation

- Source: `.full-review/02-security-performance.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-runtime/src/mcp_runtime.rs:323`
- Issue: Per-request upstream OAuth forwarding creates a fresh MCP HTTP client for every tool/resource/prompt call.
- Impact: Repeated protected calls pay repeated connection/session setup cost and can amplify load on remote MCP servers.
- Fix: Introduce a short-lived keyed client/session cache with strict credential scoping and expiry, or document the v0 isolation tradeoff.

### 15. Catalog discovery is sequential per server and capability kind

- Source: `.full-review/02-security-performance.md`
- Location: `crates/agent-runtime/src/mcp_runtime.rs:458`
- Issue: Catalog discovery probes tools, resources, resource templates, and prompts sequentially for each server.
- Impact: One slow capability endpoint can delay all catalog visibility.
- Fix: Probe independent capability lists concurrently per server and degrade only the failed capability where unsupported methods can be distinguished from hard failures.

### 16. Protected MCP sessions are unbounded and have no TTL

- Source: `.full-review/02-security-performance.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-api/src/http.rs:954`
- Issue: Protected MCP session IDs are stored in an in-memory `BTreeSet` with no expiration or maximum.
- Impact: Any successful initialize or SSE request can accumulate session IDs until explicit DELETE or process exit.
- Fix: Store session metadata with TTL and maximum capacity per route/subject, evicting expired sessions on insert/read.

### 17. API docs and implementation disagree on route and response contracts

- Source: `.full-review/03-testing-documentation.md`, `.full-review/05-final-report.md`
- Location: `docs/API.md:31`
- Issue: API docs recommend stable success/error envelopes, pagination, filtering, and route families such as `/v1/mcp/servers`, while implementation returns raw arrays/values under `/v1/gateway/*`, `/v1/oauth/*`, `/v1/marketplace/*`, and `/v1/protected-routes/*`.
- Impact: Client compatibility and generated UI contracts are hard to stabilize.
- Fix: Either update docs to mark current route families as the v0 implemented surface or wrap responses in the documented envelopes.

### 18. Security docs do not describe current protected HTTP behavior

- Source: `.full-review/03-testing-documentation.md`, `.full-review/05-final-report.md`
- Location: `docs/SECURITY.md:70`
- Issue: Security docs discuss future Streamable HTTP requirements, but implementation already includes protected Streamable HTTP routes and origin checks. Docs do not describe the current auth model, fixture verifier risk, allowed-origin policy, trusted-proxy expectations, or local/admin-only route boundaries.
- Impact: Operators cannot tell which HTTP surfaces are safe to expose.
- Fix: Document current HTTP route families, auth expectations, trusted proxy behavior, and local-only/admin-only boundaries.

### 19. Dependency boundary audit is documented but not implemented

- Source: `.full-review/04-best-practices.md`
- Location: `docs/specs/crates-and-dependencies.md:33`, `docs/QUALITY_GATES.md:58`
- Issue: Docs mention a future `cargo xtask audit-deps`, while implemented tasks include `check-deps` and `deny`; there is no automated layer audit.
- Impact: Dependency-layer rules rely on manual review and can drift, as shown by the `agent-api` -> `agent-mcp` edge.
- Fix: Implement the promised manifest/layer audit or add an explicit `cargo deny`/custom check for forbidden crate edges; update docs to distinguish dependency update checks, `cargo deny`, and boundary auditing.

### 20. Collection routes lack consistent hard pagination limits

- Source: `.full-review/04-best-practices.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-api/src/http.rs:157`
- Issue: Collection routes such as list actions, list servers, list resources, and registry search return unpaginated arrays or caller-controlled limits with no hard maximum at the HTTP boundary.
- Impact: Large catalogs can produce oversized responses and poor client behavior, contrary to the API doc's "no unbounded collections by default" requirement.
- Fix: Add default and maximum pagination limits consistently at API boundaries.

### 21. Marketplace install env resolution includes unselected package variants

- Source: `.full-review/04-best-practices.md`
- Location: `crates/agent-marketplace/src/mcp.rs:263`
- Issue: `resolve_install_env` gathers allowed environment variables from all packages on a registry server, not only the package selected by `plan_mcp_server_install`.
- Impact: A caller can supply env values for an unselected package variant and receive them back in `env_values`.
- Fix: Resolve env values against the selected package from the install plan rather than every package on the server.

### 22. Marketplace apply API echoes secret environment values

- Source: `.full-review/04-best-practices.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-api/src/http/tests.rs:215`
- Issue: Marketplace apply responses include supplied secret env values, and tests assert that behavior.
- Impact: API output can echo secrets back to clients and conflict with `docs/SECURITY.md`, which says CLI and API output must redact common secret fields.
- Fix: Return only env key names, redacted values, or a write summary from apply endpoints.

## Low

### 23. Protected MCP `Accept` header behavior and error contract disagree

- Source: `.full-review/02-security-performance.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-api/src/http.rs:876`
- Issue: `accepts_mcp_response` allows missing `Accept` headers, but the rejection message says the route requires `Accept: application/json or text/event-stream`.
- Impact: Implementation and error contract drift can hide client compatibility bugs.
- Fix: Either enforce the header or update the contract/tests to state that it is optional.

### 24. Redaction literals are inconsistent across crates

- Source: `.full-review/02-security-performance.md`, `.full-review/04-best-practices.md`, `.full-review/05-final-report.md`
- Location: `crates/agent-observability/src/redaction.rs:4`, `crates/agent-search/src/document.rs:61`
- Issue: Observability redaction emits `[redacted]`, while search sanitization and docs use `[REDACTED]`.
- Impact: Downstream assertions, logs, search behavior, and operator expectations can diverge.
- Fix: Centralize one canonical redaction literal and helper in an appropriate low-level crate or observability facade.

### 25. Pre-existing untracked session note blocks docs audit

- Source: `.full-review/03-testing-documentation.md`, `.full-review/05-final-report.md`
- Location: `docs/sessions/2026-05-16-gateway-extraction-merge-cleanup.md:1`
- Issue: `cargo xtask audit-docs` fails on a pre-existing untracked session note that lacks required frontmatter keys.
- Impact: Pre-push docs audit remains blocked if this note is intended to be committed.
- Fix: Add required frontmatter to the session note or keep it out of the authored-docs audit path.

### 26. Inline tests remain in the central binary file

- Source: `.full-review/01-quality-architecture.md`, `.full-review/03-testing-documentation.md`
- Location: `crates/agent-server/src/main.rs:1105`
- Issue: `agent-server` keeps substantial inline test bodies inside the central 1,457-line composition file.
- Impact: This conflicts with the test placement contract and increases noise in the file most likely to be touched by command/router work.
- Fix: Move tests into `crates/agent-server/src/main/tests.rs` or equivalent source-side sidecar modules as part of the module split.

## Verification And Environment Issues

### 27. Normal cargo path failed before Cargo execution

- Source: `.full-review/00-scope.md`, `.full-review/02-security-performance.md`, `.full-review/05-final-report.md`
- Issue: `cargo metadata --no-deps --format-version 1` and `cargo tree -e normal --workspace` failed through `/home/jmagar/.local/bin/cargo` before Cargo could run because the local wrapper resolved through a Snap-backed `rustc` with a `snap-confine has elevated permissions and is not confined` error.
- Impact: The default toolchain path cannot be trusted for verification in this environment until repaired.
- Fix: Repair the local cargo/rustc wrapper path or document the direct rustup cargo invocation needed for this repo.

### 28. `sccache` wrapper failed under the direct cargo route

- Source: `.full-review/00-scope.md`, `.full-review/02-security-performance.md`, `.full-review/05-final-report.md`
- Issue: Direct rustup cargo needed `RUSTC_WRAPPER=` because `sccache` failed with `Operation not permitted`.
- Impact: Verification commands require a local workaround and may fail for contributors with the same wrapper setup.
- Fix: Repair `sccache` permissions/configuration or document when to clear `RUSTC_WRAPPER` for local verification.

### 29. `cargo xtask nextest` could not run

- Source: `.full-review/03-testing-documentation.md`, `.full-review/04-best-practices.md`, `.full-review/05-final-report.md`
- Issue: `cargo xtask nextest` failed because `cargo-nextest` is not installed for the direct cargo path.
- Impact: The repo's preferred test suite did not run during the review.
- Fix: Install `cargo-nextest` for the active toolchain path or make `cargo xtask doctor` explain the missing dependency before running verification.

### 30. `cargo-deny` and `taplo` were also missing from the visible tool path

- Source: `.full-review/03-testing-documentation.md`, `.full-review/04-best-practices.md`, `.full-review/05-final-report.md`
- Issue: Tool lookup found `gitleaks`, `lefthook`, and `just`, but not `cargo-nextest`, `cargo-deny`, or `taplo`.
- Impact: Dependency and TOML validation gates documented for the project are not fully runnable.
- Fix: Install the missing tools or update the documented gate commands/tool discovery to match the intended environment.

## Current Passing Evidence From Review

- `/home/jmagar/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo metadata --no-deps --format-version 1` passed.
- `RUSTC_WRAPPER= /home/jmagar/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo tree -e normal --workspace` passed.
- `RUSTC_WRAPPER= /home/jmagar/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo xtask check` passed.

## Recommended Fix Order

1. Remove production access to fixture bearer verification and add negative production-auth tests.
2. Harden JWT issuer and time-claim validation.
3. Add explicit admin/local-only boundaries for mutating HTTP route families.
4. Stop echoing secret env values from marketplace apply APIs.
5. Fix local verification tooling so documented gates are reproducible.
6. Split `agent-api/src/http.rs` and `agent-server/src/main.rs`.
7. Add runtime startup/catalog concurrency and shutdown lifecycle tests.
8. Align API/security docs with the implemented v0 HTTP surfaces.
9. Add automated dependency boundary enforcement.
