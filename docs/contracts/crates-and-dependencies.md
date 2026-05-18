---
title: "Crates And Dependencies Contract"
doc_type: "contract"
status: "active"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs: []
related:
  - "docs/CRATE_BOUNDARIES.md"
  - "docs/CODE_ORGANIZATION.md"
  - "docs/specs/crates-and-dependencies.md"
last_reviewed: "2026-05-18"
last_modified: "2026-05-18"
modified_on_branch: "review-remediation/full-review-issues"
modified_at_version: "0.1.0"
modified_at_commit: "d327495"
review_basis: "local workspace crate and dependency policy plus gateway OAuth token exchange implementation"
---

# Crates And Dependencies Contract

This contract defines dependency legality for AgentCast crates.

`docs/CRATE_BOUNDARIES.md` defines what each crate owns. This contract defines which crate dependencies are allowed to support that ownership.

## Workspace Policy

Shared external dependency versions must live in root `Cargo.toml` under `[workspace.dependencies]`.

Crates should consume shared dependencies with `.workspace = true` unless there is an explicit reason to pin a crate-local version or feature set.

New dependency additions must answer:

- which crate owns the dependency.
- why the dependency belongs in that crate.
- whether default features are disabled when they bring unnecessary runtime, TLS, network, or platform behavior.
- whether the dependency changes test, binary, security, or platform behavior.
- whether an existing workspace dependency already covers the need.

## Dependency Direction

Dependency direction must follow ownership:

```txt
surfaces -> runtime/domain -> protocol models -> adapters -> external SDKs
```

Surface crates are:

- `agent-cli`
- `agent-api`
- `agent-ui-contracts`
- `agent-server`

Runtime/domain crates are:

- `agent-runtime`
- `agent-gateway`
- `agent-marketplace`
- `agent-registry`
- `agent-store`
- `agent-search`
- `agent-auth`
- `agent-observability`
- `agent-config`

Protocol/model crates are:

- `agent-core`
- `agent-protocol`
- `agent-schema`

Adapter crates are:

- `agent-mcp`
- `agent-acp`

Post-v0 crates are:

- `agent-stash`
- `agent-fleet`

Post-v0 crates must not become required dependencies of the MCP launcher v0 runtime path unless `docs/MVP.md` explicitly promotes that behavior.

## External SDK Ownership

External protocol SDK ownership is exclusive unless a future decision expands it.

- `rmcp` belongs in `agent-mcp`.
- `agent-client-protocol` belongs in `agent-acp`.
- `axum`, `tower`, `tower-http`, `utoipa`, and `utoipa-scalar` belong in `agent-api` and top-level server composition only.
- `clap` belongs in `agent-cli`.
- process supervision dependencies belong in `agent-runtime`.
- SQLite dependencies belong in `agent-store`.
- registry HTTP client dependencies belong in `agent-registry` unless another crate has an explicit external fetch responsibility.
- OAuth token endpoint HTTP client dependencies belong in `agent-gateway` because upstream OAuth lifecycle orchestration is gateway-owned.

Other crates must consume AgentCast-owned models and services instead of importing those external SDKs directly.

## Low-Level Crate Restrictions

`agent-core` must stay dependency-light.

`agent-core` must not depend on:

- async runtimes.
- HTTP servers or clients.
- CLI parsers.
- ACP or MCP SDKs.
- database clients.
- UI/frontend crates.

`agent-protocol` must not depend on protocol SDKs such as `rmcp` or `agent-client-protocol`. It owns AgentCast internal protocol-neutral models, not upstream SDK types.

`agent-schema` may own schema parsing and validation helpers, but must not render CLI/UI output, invoke tools, fetch registries, or own runtime policy.

## Surface Crate Restrictions

Surface crates may parse input and render output. They must call runtime/domain crates for decisions.

Surface crates must not:

- duplicate gateway routing policy.
- duplicate runtime lifecycle behavior.
- fetch registries directly.
- invoke protocol SDKs directly.
- mutate install targets outside runtime/apply APIs.

Current documented exception: `agent-server` may depend on `agent-mcp` only for
the temporary v0 stdio gateway composition path. It must not grow additional
MCP adapter policy, and `cargo xtask audit-deps` must encode this exception
explicitly rather than allowing broad surface-to-adapter dependencies.

## Adapter Restrictions

Protocol adapters must be thin.

`agent-mcp` and `agent-acp` may:

- wrap protocol SDK types.
- convert upstream SDK types to AgentCast internal models.
- map upstream errors into AgentCast errors.
- expose transport/client/server helper functions.

They must not own:

- process/session lifecycle.
- provider selection.
- tool exposure policy.
- install planning.
- CLI commands.
- HTTP routes.

## Dependency Exceptions

Exceptions must be documented before or with the change.

Acceptable exception documentation locations:

- `docs/DECISIONS.md` for broad or durable exceptions.
- the owning contract/spec when the exception changes the architecture.
- an implementation plan when the exception is temporary and scoped.

An exception must include:

- the crate.
- the dependency edge.
- why the normal boundary is insufficient.
- how the exception is verified.
- whether it is temporary or durable.

## Verification Requirement

Substantial crate or dependency changes must run:

```bash
cargo xtask verify
```

Dependency-policy changes should also run the dependency audit once it exists:

```bash
cargo xtask audit-deps
```

The audit is implemented as a fast, repo-local manifest check. It currently
enforces targeted forbidden edges and SDK-owner rules from this contract, such
as preventing `agent-api` from depending directly on `agent-mcp`.
