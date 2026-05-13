---
title: "Developer Speed"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs: []
related:
  - "docs/DECISIONS.md"
  - "docs/DEVELOPMENT.md"
  - "docs/TESTING.md"
  - "docs/QUALITY_GATES.md"
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "local tooling policy"
---

# Developer Speed

AgentCast should be fast to install, fast to run, fast to test, and fast to operate across CLI, MCP, API, web, desktop, and command palette surfaces.

## Default Loop

Use these commands for normal local work:

```bash
cargo xtask check
cargo xtask nextest
cargo xtask ci
```

`just` is a convenience wrapper over `cargo xtask`:

```bash
just check
just test
just ci
just verify
```

## Nextest

`cargo nextest run --workspace` is the preferred test runner for Rust tests. Keep `cargo test --workspace` available for doctests, compatibility checks, and cases where nextest is unavailable.

## Cranelift

Cranelift is approved for local debug build acceleration when the active Rust toolchain supports it.

Do not make Cranelift the default release or CI release path. Treat it as an opt-in local speed path because toolchain support varies across machines.

Suggested local-only usage when the active toolchain supports Cranelift:

```bash
cargo xtask check-cranelift
```

If the toolchain rejects `-Zcodegen-backend` or `+nightly` is unavailable, use the normal `cargo xtask check` path.

## Hooks

Pre-commit hooks stay intentionally light and limited to the approved set in [DECISIONS.md](./DECISIONS.md). Heavy checks belong in `cargo xtask ci`, `cargo xtask verify`, pre-push, or CI.
