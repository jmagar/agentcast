# Phase 3: Testing and Documentation

## Findings

- High - `crates/agent-api/src/protected_mcp/tests.rs:9`
  Protected MCP route tests build the default API with `ProtectedMcpRouteApi::new(routes)`, which exercises the fixture bearer verifier as the normal path. There is a positive test for explicit `StaticBearerTokenVerifier`, but no test prevents production router helpers from using fixture parsing.
  Impact: the Critical Phase 2 auth bypass can regress silently because tests normalize the unsafe default.
  Fix by requiring verifier injection in production helpers and adding tests that fixture-shaped bearer headers fail unless a test-only fixture verifier is explicitly selected.

- High - `crates/agent-auth/src/bearer/tests.rs:117`
  JWT tests cover valid HS256, tampered payloads, and wrong audience, but not expired tokens, missing expiration, `nbf`, or issuer mismatch. The test suite therefore documents the current weak JWT contract as acceptable.
  Impact: future protected-route JWT mode can ship without freshness or issuer guarantees.
  Fix by adding failing tests for expired, not-yet-valid, missing `exp`, and wrong `iss` claims before hardening the verifier.

- Medium - `crates/agent-runtime/src/mcp_runtime/tests.rs:1`
  Runtime tests cover calls, response caps, circuit breaker behavior, reprobe, and ephemeral auth, but there is no process-lifecycle test proving `McpRuntime::shutdown` terminates stdio child processes.
  Impact: leaked child processes can survive a passing test suite.
  Fix by adding a fixture stdio server that records liveness and asserting it exits after runtime shutdown.

- Medium - `crates/agent-api/src/http/tests.rs:223`
  Protected-route admin tests prove CRUD behavior, but not authorization requirements around those admin routes. The absence matches the current unauthenticated router shape, but it means tests will not catch accidental network exposure of admin mutation endpoints.
  Impact: route-family auth expectations remain implicit.
  Fix by adding either an explicit local/admin-auth wrapper test or a documentation-backed test that these routers are only composed in local-only contexts.

- Medium - `docs/API.md:31`
  The API contract recommends stable success/error envelopes, pagination, filtering, and route families such as `/v1/mcp/servers`, but current HTTP routes return raw JSON arrays/values under `/v1/gateway/*`, `/v1/oauth/*`, `/v1/marketplace/*`, and `/v1/protected-routes/*`.
  Impact: docs and implementation disagree on the API contract, which makes client compatibility and generated UI contracts harder to stabilize.
  Fix by either updating the docs to mark the current route families as the v0 implemented surface or wrapping responses in the documented envelope.

- Medium - `docs/SECURITY.md:70`
  The security doc says future Streamable HTTP support must follow upstream Origin/auth/localhost guidance before non-stdio defaults, but the implementation already includes protected Streamable HTTP routes and HTTP-origin checks. The doc does not describe the current protected-route auth model, fixture verifier risk, or allowed-origin/trusted-proxy expectations.
  Impact: operators cannot tell which HTTP surfaces are safe to expose.
  Fix by documenting current HTTP route families, auth expectations, trusted proxy behavior, and local-only/admin-only boundaries.

- Low - `docs/sessions/2026-05-16-gateway-extraction-merge-cleanup.md:1`
  `cargo xtask audit-docs` currently fails on this pre-existing untracked session note because it lacks required frontmatter keys. This file was already untracked before the review.
  Impact: pre-push docs audit remains blocked if this note is intended to be committed.
  Fix by adding the required frontmatter to the session note or keeping it out of the authored-docs audit path.

- Low - `crates/agent-server/src/main.rs:1105`
  `agent-server` keeps inline tests inside a 1,457-line composition file. This conflicts with the test placement contract and makes the test suite harder to scan.
  Impact: future command/router work will touch a large, noisy file.
  Fix by moving those tests into sidecar modules as part of the module split.

## Verification

- `RUSTC_WRAPPER= /home/jmagar/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo xtask check` passed.
- `RUSTC_WRAPPER= /home/jmagar/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo xtask audit-docs` failed with 15 missing frontmatter-key errors in `docs/sessions/2026-05-16-gateway-extraction-merge-cleanup.md`.
- `RUSTC_WRAPPER= /home/jmagar/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo xtask nextest` failed because the direct cargo path does not have `cargo-nextest` installed.
- `which cargo-nextest cargo-deny gitleaks lefthook taplo just` found `gitleaks`, `lefthook`, and `just`, but not `cargo-nextest`, `cargo-deny`, or `taplo`.

## Documentation Strengths

- `docs/MVP.md` gives a clear v0 boundary and acceptance criteria.
- `docs/CRATE_BOUNDARIES.md`, `docs/contracts/crates-and-dependencies.md`, `docs/CODE_ORGANIZATION.md`, and `docs/TESTING.md` provide concrete reviewable contracts.
- The docs correctly preserve post-v0 ACP/fleet/marketplace ambitions while prioritizing the MCP launcher loop.
