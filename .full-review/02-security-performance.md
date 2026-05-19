# Phase 2: Security and Performance

## Findings

- Critical - `crates/agent-api/src/protected_mcp.rs:21`
  `ProtectedMcpRouteApi::new` defaults to `FixtureBearerTokenVerifier`, and `protected_mcp_router` uses that constructor through `protected_mcp_router_with_oauth_service`. That means any deployed protected MCP router that does not explicitly inject a real verifier accepts fixture-form headers such as `Bearer sub=...;aud=...;scope=...`.
  Impact: protected routes can be bypassed if production composition uses the default helper.
  Fix by removing the public fixture-default constructor from production paths, requiring an explicit verifier for all protected MCP router construction, and keeping fixture parsing behind `#[cfg(test)]` or a test-only feature.

- High - `crates/agent-auth/src/bearer.rs:255`
  JWT verification checks HS256 signature and audience, but the deserialized payload has no `exp`, `nbf`, or `iss` fields and the verifier has no clock/issuer validation. A validly signed expired token is accepted indefinitely.
  Impact: revoked or stale bearer credentials can continue authorizing protected MCP routes.
  Fix by adding issuer and time-claim validation with configurable clock skew, and failing closed when `exp` is missing for protected-route JWT mode.

- High - `crates/agent-api/src/http.rs:903`
  Origin validation compares `Origin` to `public_origin(headers)`, and `public_origin` is derived from untrusted `Host` and `x-forwarded-proto` headers. Without trusted-proxy configuration, a client can make the computed public origin match a forged `Origin`.
  Impact: the origin check does not reliably protect browser-accessible protected MCP endpoints when the service is exposed through a proxy or directly reachable.
  Fix by configuring allowed public origins or trusted proxy headers explicitly, and validate `Origin` against that configured set rather than raw request headers.

- High - `crates/agent-api/src/http.rs:60`
  `oauth_router`, `marketplace_router`, `registry_router`, and protected-route admin router are exposed as unauthenticated routers by construction. Some endpoints can mutate route collections, store OAuth credentials, register clients, or apply marketplace install previews to supplied config bodies. These may be intended as local-only APIs, but the code does not enforce locality or admin auth at the router boundary.
  Impact: accidental network exposure could allow unauthorized OAuth state manipulation or protected-route configuration changes.
  Fix by requiring an admin/auth layer or local-bind guard for mutating API route families, and make unauthenticated helpers test-only or clearly local-only.

- Medium - `crates/agent-runtime/src/mcp_runtime.rs:208`
  `McpRuntime::shutdown` only clears the upstream map. The runtime relies on RMCP/transport drop semantics to terminate stdio children and HTTP sessions, and `McpClientOptions` are created with fresh cancellation tokens that are not retained for shutdown.
  Impact: stale MCP child processes or HTTP sessions can survive runtime shutdown if transport drop behavior is insufficient.
  Fix by storing cancellation tokens or explicit process/session handles per upstream and cancelling them during shutdown; verify with a process-lifecycle test.

- Medium - `crates/agent-runtime/src/mcp_runtime.rs:323`
  Per-request upstream OAuth forwarding for streamable HTTP creates a new MCP HTTP client for every tool/resource/prompt call. That is safe for credential isolation, but it repeats connection/session setup for every protected call.
  Impact: protected-route calls can become expensive under normal repeated use and can amplify load on remote MCP servers.
  Fix by introducing a short-lived keyed client/session cache with strict credential scoping and expiry, or document that v0 intentionally trades performance for isolation.

- Medium - `crates/agent-runtime/src/mcp_runtime.rs:458`
  Catalog discovery performs tools, resources, resource templates, and prompts sequentially for each server. Combined with sequential server startup, cold start cost scales linearly by server and capability kind.
  Impact: one slow capability endpoint can delay all other catalog visibility, hurting the MVP setup and command-palette loop.
  Fix by probing independent capability lists concurrently per server and degrading only the failed capability where the MCP SDK can distinguish unsupported methods from hard connection failure.

- Medium - `crates/agent-api/src/http.rs:954`
  Protected MCP session IDs are stored in an in-memory `BTreeSet` with no expiration or maximum. Any successful initialize or SSE request inserts a UUID and it remains until explicit DELETE or process exit.
  Impact: a client can accumulate unbounded session IDs over time.
  Fix by storing session metadata with TTL and a maximum capacity per route/subject, evicting expired sessions on insert/read.

- Low - `crates/agent-api/src/http.rs:876`
  `accepts_mcp_response` allows missing `Accept` headers. That may be convenient for clients, but the rejection message says the protected MCP route requires `Accept: application/json or text/event-stream`.
  Impact: the implementation and error contract are inconsistent, which can hide client compatibility bugs.
  Fix by either enforcing the header or correcting the contract/tests to say it is optional.

- Low - `crates/agent-observability/src/redaction.rs:4`
  The observability redaction helper emits `[redacted]`, while search sanitization emits `[REDACTED]` in `crates/agent-search/src/document.rs:61` and docs call for `[REDACTED]`.
  Impact: downstream assertions and operator expectations can drift across surfaces.
  Fix by centralizing the redaction literal or aligning all redaction helpers to the documented canonical value.

## Verification Notes

- `cargo metadata --no-deps --format-version 1` failed before Cargo could run because `/home/jmagar/.local/bin/cargo` resolved through rustup to a Snap-backed `rustc`, which refused to execute with `snap-confine has elevated permissions and is not confined`.
- `cargo tree -e normal --workspace` failed with the same Snap AppArmor error.
- `/home/jmagar/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo metadata --no-deps --format-version 1` passed.
- `RUSTC_WRAPPER= /home/jmagar/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo tree -e normal --workspace` passed after bypassing `sccache`, which otherwise failed with `Operation not permitted`.
- No build, clippy, nextest, or docs audit result should be considered current from this review checkpoint.

## Critical Issues for Phase 3 Context

- Tests need explicit coverage proving production protected MCP router construction cannot use fixture bearer auth.
- Tests need JWT expiration/issuer failure cases once bearer validation is hardened.
- Runtime lifecycle tests should prove stdio child shutdown and session cleanup behavior.
- Documentation should state which API route families are local-only, admin-protected, or intentionally unauthenticated.
