# Crates Agent Guide

This directory contains the reusable Rust crates for AgentCast. Place code by ownership, not convenience. See `../docs/CRATE_BOUNDARIES.md` for ownership and `../docs/contracts/crates-and-dependencies.md` for dependency legality.

## Dependency Direction

Surface crates parse input and render output. Runtime/domain crates make decisions.

```txt
agent-cli / agent-api / agent-ui-contracts
    -> agent-runtime / agent-gateway / agent-marketplace / agent-registry
    -> agent-protocol
    -> agent-acp / agent-mcp
    -> external SDKs and systems
```

Keep business decisions out of CLI handlers, HTTP routes, and UI DTO shaping. Put orchestration in `agent-runtime`, exposure/routing policy in `agent-gateway`, registry aggregation in `agent-registry`, and install planning in `agent-marketplace`.

Shared external dependency versions belong in root `[workspace.dependencies]`. Consume them with `.workspace = true` unless a documented exception says otherwise.

## Crate Ownership

- `agent-core`: shared IDs, errors, events, timestamps, small dependency-light primitives.
- `agent-config`: config files, env loading, precedence, validation.
- `agent-protocol`: AgentCast protocol-neutral internal models.
- `agent-schema`: JSON Schema normalization, validation, CLI/form metadata.
- `agent-acp`: ACP adapter shim only.
- `agent-mcp`: MCP adapter shim only.
- `agent-registry`: registry fetch/search/cache/normalization.
- `agent-marketplace`: install/marketplace planning and previews.
- `agent-runtime`: process/session/workspace lifecycle and orchestration.
- `agent-observability`: tracing, audit, redaction, health primitives.
- `agent-store`: SQLite/local persistence and migrations.
- `agent-search`: launcher action indexing and ranking.
- `agent-gateway`: tool exposure, routing, collision, allow/deny policy.
- `agent-auth`: generic auth primitives and auth errors.
- `agent-api`: Axum routers, HTTP DTOs, OpenAPI wiring.
- `agent-ui-contracts`: shared client-facing view/API DTOs.
- `agent-cli`: Clap commands and terminal rendering.
- `agent-server`: thin binary composition and graceful shutdown.
- `agent-stash`: saved user artifacts and reusable context collections.
- `agent-fleet`: future multi-node/remote-agent coordination contracts.

If a change does not fit cleanly, stop and update the boundary docs or design before spreading logic across crates.

## Adapter Rules

`agent-acp` and `agent-mcp` are thin protocol adapters. They convert SDK/upstream types into AgentCast internal models and map protocol errors into AgentCast errors.

They must not own:

- runtime lifecycle.
- process supervision.
- gateway exposure policy.
- install policy.
- CLI commands.
- HTTP routes.

## Surface Rules

`agent-cli`, `agent-api`, and `agent-ui-contracts` must stay thin.

They may:

- parse requests or CLI args.
- call domain/runtime crates.
- render output or DTOs.
- map domain errors to surface-specific responses.

They must not:

- duplicate business policy.
- spawn protocol servers directly.
- fetch registries directly.
- mutate install targets outside runtime/apply APIs.

## Module Shape

Prefer small modules with one clear responsibility. Split by behavior, policy, parser step, or rendering step.

Avoid broad module names such as `utils`, `helpers`, `manager`, and `common` unless the file is demonstrably narrow.

Follow the repo guardrails:

- target source module size: 250 non-test lines or less.
- split-review threshold: more than 400 non-test lines.
- target function size: 60 lines or less.

## Tests

Use source-side sidecar tests by default:

```txt
src/foo.rs
src/foo/tests.rs
src/foo_test.rs
src/foo/tests/*.rs
```

Use `crates/*/tests/*.rs` only for public API, CLI, API, protocol compatibility, or full workflow tests.

Do not make internal functions public just to test them. Use sidecar tests for private module behavior.

Default tests must be deterministic and offline. Live upstream tests require explicit gating.

## Verification

For crate work, run the narrowest useful command while iterating:

```bash
cargo test -p agent-runtime
cargo test -p agent-gateway
cargo xtask nextest
```

Before declaring substantial crate work complete, run:

```bash
cargo xtask verify
```
