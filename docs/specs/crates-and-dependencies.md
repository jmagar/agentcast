---
title: "Crates And Dependencies Spec"
doc_type: "spec"
status: "active"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs: []
related:
  - "docs/contracts/crates-and-dependencies.md"
  - "docs/CRATE_BOUNDARIES.md"
  - "docs/QUALITY_GATES.md"
last_reviewed: "2026-05-18"
last_modified: "2026-05-18"
modified_on_branch: "review-remediation/full-review-issues"
modified_at_version: "0.1.0"
modified_at_commit: "e0bd04f"
review_basis: "local workspace crate and dependency policy"
---

# Crates And Dependencies Spec

This spec describes how AgentCast should implement and verify the [crates and dependencies contract](../contracts/crates-and-dependencies.md).

## Dependency Audit Command

Use the implemented xtask:

```bash
cargo xtask audit-deps
```

The command is fast enough for pre-push and does not require network access.

The audit should parse the workspace manifests from:

- root `Cargo.toml`
- `crates/*/Cargo.toml`
- `xtask/Cargo.toml`

Use a TOML parser when the implementation needs more than simple file inventory.

## Initial Checks

`cargo xtask audit-deps` should fail when:

- a crate uses an external dependency version directly instead of `.workspace = true` for a dependency already listed in root `[workspace.dependencies]`.
- `agent-core` depends on `tokio`, `axum`, `clap`, `rmcp`, `agent-client-protocol`, `reqwest`, `rusqlite`, or UI/frontend dependencies.
- `agent-protocol` depends on `rmcp`, `agent-client-protocol`, `axum`, `clap`, or UI/frontend dependencies.
- `agent-mcp` is not the owner of direct `rmcp` usage.
- `agent-acp` is not the owner of direct `agent-client-protocol` usage.
- `agent-cli` imports HTTP route/server dependencies directly.
- `agent-api` imports CLI parser dependencies directly.
- post-v0 crates become dependencies of v0 runtime path crates without an explicit decision.

The v0 runtime path crates are:

- `agent-config`
- `agent-mcp`
- `agent-runtime`
- `agent-gateway`
- `agent-registry`
- `agent-marketplace`
- `agent-store`
- `agent-schema`
- `agent-protocol`
- `agent-core`
- `agent-cli`
- `agent-api`
- `agent-server`

## Dependency Layers

The audit should classify crates into layers:

```txt
core:        agent-core
models:      agent-protocol, agent-schema
adapters:    agent-mcp, agent-acp
domain:      agent-config, agent-registry, agent-marketplace, agent-runtime, agent-gateway, agent-store, agent-search, agent-auth, agent-observability
surfaces:    agent-cli, agent-api, agent-ui-contracts, agent-server
post-v0:     agent-stash, agent-fleet
tooling:     xtask
```

Layer checks should flag dependency edges that invert ownership. The current implementation starts with targeted deny-list checks before adding a complete dependency graph policy.

The only current surface-to-adapter exception is `agent-server` -> `agent-mcp`
for the temporary v0 stdio gateway composition path. The audit must encode that
exception explicitly and continue to reject broad surface crate dependencies on
protocol adapters.

## Output Format

Human output should include:

- checked manifests count.
- dependency-policy issue count.
- one finding per line with crate, dependency, and violated rule.

Example:

```txt
audit-deps: checked 22 manifests
- agent-protocol: direct dependency on rmcp violates protocol-neutral model rule
- agent-core: direct dependency on tokio violates low-level crate restriction
```

Exit status:

- `0` when no findings exist.
- non-zero when any policy finding exists.
- non-zero when manifest parsing fails.

## Exceptions

The first implementation can use a small static allowlist in `xtask`.

Each allowlist entry must include:

- crate name.
- dependency name.
- reason.
- owning docs path for the exception.

Do not silently ignore dependency edges.

## Integration

`cargo xtask audit-deps` is part of:

- `cargo xtask verify`.
- `docs/QUALITY_GATES.md`.

It is also included in the `cargo xtask ci` sequence.

Do not add it to pre-commit without explicit approval in `docs/DECISIONS.md`.
