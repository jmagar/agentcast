# Comprehensive Code Review Report

## Review Target

Full AgentCast workspace review from the current `main` checkout at `/home/jmagar/workspace/agentcast`, focused on the v0 MCP launcher/runtime contract.

## Executive Summary

AgentCast has strong written contracts and a coherent MCP-first crate plan, but the current implementation has one critical auth-default issue and several places where production code, tests, and docs normalize prototype behavior. The largest implementation risk is not missing functionality; it is that protected HTTP/MCP surfaces, gateway composition, and install/apply flows can look production-ready while still carrying fixture defaults, unauthenticated mutation routers, and unclear exposure boundaries.

## Findings by Priority

### Critical Issues

- Critical - Phase 2 - `crates/agent-api/src/protected_mcp.rs:21`
  Protected MCP API construction defaults to `FixtureBearerTokenVerifier`, allowing fixture-form bearer headers if production composition uses the default helper.
  Fix by requiring explicit production verifiers and gating fixture parsing to tests or a fixture feature.

### High Priority

- High - Phase 2 - `crates/agent-auth/src/bearer.rs:255`
  JWT verification lacks `exp`, `nbf`, and `iss` validation.
  Fix by adding issuer/time validation and tests for expired/missing claims.

- High - Phase 2 - `crates/agent-api/src/http.rs:903`
  Origin validation is based on request-derived `Host` and `x-forwarded-proto`.
  Fix by using configured allowed origins and trusted-proxy handling.

- High - Phase 2 - `crates/agent-api/src/http.rs:60`
  OAuth, marketplace, registry, and protected-route admin routers are unauthenticated by construction.
  Fix by requiring admin/local-only composition for mutating route families.

- High - Phase 1 - `crates/agent-server/src/main.rs:1`
  The binary composition file is 1,457 lines and owns too many concerns.
  Fix by splitting CLI, command, server, OAuth, and backend projection modules.

- High - Phase 1 - `crates/agent-api/src/http.rs:1`
  The HTTP module is 1,339 lines and combines multiple route families plus DTOs and transport helpers.
  Fix by splitting route families and shared response helpers.

- High - Phase 1 - `crates/agent-api/Cargo.toml:15`
  `agent-api` depends directly on `agent-mcp`, weakening the documented surface -> runtime -> adapter direction.
  Fix by removing the dependency or documenting a narrow exception.

- High - Phase 4 - `xtask/src/task.rs:72`
  The current visible environment cannot run the documented nextest/deny/taplo gates.
  Fix local toolchain wiring and make doctor fail clearly for missing gate tools.

### Medium Priority

- Medium - Phase 1 - `crates/agent-runtime/src/mcp_runtime.rs:74`
  Runtime starts upstreams sequentially.
  Fix with bounded concurrent startup.

- Medium - Phase 1 - `crates/agent-store/src/oauth.rs:187`
  Synchronous SQLite store calls run behind async HTTP handlers without an explicit blocking boundary.
  Fix by using `spawn_blocking` or documenting v0 local-only assumptions.

- Medium - Phase 1 - `crates/agent-marketplace/src/mcp.rs:172`
  Install apply reconstructs mutations from JSON preview fields.
  Fix with typed install action payloads.

- Medium - Phase 2 - `crates/agent-runtime/src/mcp_runtime.rs:208`
  Runtime shutdown does not retain explicit cancellation/process handles.
  Fix with stored cancellation handles and lifecycle tests.

- Medium - Phase 2 - `crates/agent-runtime/src/mcp_runtime.rs:323`
  Protected streamable HTTP calls create a new MCP client per operation.
  Fix with scoped short-lived client caching or document the isolation tradeoff.

- Medium - Phase 2 - `crates/agent-api/src/http.rs:954`
  Protected MCP session IDs have no TTL or capacity bound.
  Fix with session metadata, expiry, and maximum capacity.

- Medium - Phase 3 - `docs/API.md:31`
  API docs describe envelopes/pagination and route families that do not match current routes.
  Fix docs or response shapes before declaring API stability.

- Medium - Phase 4 - `crates/agent-api/src/http/tests.rs:215`
  Marketplace apply responses echo supplied secret env values.
  Fix by redacting or omitting secret values in API output.

### Low Priority

- Low - Phase 2 - `crates/agent-api/src/http.rs:876`
  `Accept` header behavior is more permissive than its error message says.

- Low - Phase 2 - `crates/agent-observability/src/redaction.rs:4`
  Redaction literals differ across crates.

- Low - Phase 3 - `docs/sessions/2026-05-16-gateway-extraction-merge-cleanup.md:1`
  A pre-existing untracked session note blocks `cargo xtask audit-docs`.

## Findings by Category

### Architecture and Code Quality

The crate plan is strong, but `agent-server`, `agent-api`, and `agent-runtime` have grown past the repo's own module-size standards. `agent-api` also has a direct adapter dependency that conflicts with the documented dependency direction.

### Security

The protected MCP auth default is the most serious issue. JWT freshness/issuer validation, request-derived origin checks, unauthenticated admin/mutation routers, and secret echoing in marketplace apply output need hardening before exposing HTTP surfaces beyond strictly local development.

### Performance

Cold start and catalog discovery are sequential, protected HTTP operations create a client per call, and protected MCP sessions are unbounded. These are manageable for v0 but should be explicit tradeoffs if not fixed immediately.

### Testing

The repo has broad sidecar test coverage, but tests currently exercise fixture auth as the happy path. Add negative production-auth tests, JWT time/issuer tests, lifecycle shutdown tests, and admin-route auth/composition tests.

### Documentation

The core docs are unusually strong and useful as contracts. The main gap is drift: API/security docs do not yet describe the implemented HTTP route families, auth defaults, route exposure boundaries, or current response shapes.

### Standards and Operations

Xtask, hooks, deny, and gitleaks are a good operational baseline. Current local tool visibility is not aligned with the documented gates: `cargo-nextest`, `cargo-deny`, and `taplo` are missing from PATH for this review's direct cargo route.

## Recommended Fix Order

1. Remove production access to fixture bearer verification and add tests that fixture headers are rejected unless a test-only verifier is explicitly selected.
2. Harden JWT validation with issuer and time claims.
3. Add explicit admin/local-only auth boundaries for OAuth, marketplace apply, registry mutation if added later, and protected-route admin routers.
4. Stop echoing secret env values from marketplace apply APIs.
5. Fix local verification tooling so `cargo xtask ci`, `audit-docs`, `secrets`, and `verify` are reproducible.
6. Split `agent-api/src/http.rs` and `agent-server/src/main.rs` along route/command boundaries.
7. Add bounded concurrency for runtime startup/catalog discovery and lifecycle tests for shutdown.
8. Align API/security docs with the implemented v0 HTTP surfaces.

## Residual Risks

- This was a source-level review, not a full exploit validation or runtime soak test.
- `cargo xtask check` passed, but nextest and full verify did not run because the visible direct cargo toolchain lacks `cargo-nextest`.
- `cargo xtask audit-docs` is currently blocked by a pre-existing untracked session note.
- The worktree was dirty before review; unrelated README/session files were not modified.

## Verification

- Passed: `/home/jmagar/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo metadata --no-deps --format-version 1`
- Passed: `RUSTC_WRAPPER= /home/jmagar/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo tree -e normal --workspace`
- Passed: `RUSTC_WRAPPER= /home/jmagar/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo xtask check`
- Failed: `RUSTC_WRAPPER= /home/jmagar/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo xtask audit-docs`, due to missing frontmatter in pre-existing untracked `docs/sessions/2026-05-16-gateway-extraction-merge-cleanup.md`
- Failed: `RUSTC_WRAPPER= /home/jmagar/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo xtask nextest`, because `cargo-nextest` is not installed for that cargo path
- Failed initially: normal `cargo metadata` and `cargo tree` through `/home/jmagar/.local/bin/cargo` hit Snap/sccache wrapper errors
