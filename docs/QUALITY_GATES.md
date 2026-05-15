---
title: "Quality Gates"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs: []
related:
  - "docs/contracts/crates-and-dependencies.md"
  - "docs/DECISIONS.md"
  - "docs/DEVELOPMENT.md"
  - "docs/TESTING.md"
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "local repo tooling policy"
---

# Quality Gates

AgentCast uses lightweight commit-time checks and heavier intentional verification.

## Pre-Commit

Only the approved hooks in [DECISIONS.md](./DECISIONS.md) may run at pre-commit time:

- staged secret scan.
- staged conflict-marker check.
- staged large-file guard.
- staged TOML/YAML/JSON syntax check, with Taplo for TOML.
- staged line-ending and executable-bit sanity check.

No Cargo build, test, clippy, network, registry refresh, generated-doc rewrite, or metadata audit may be added to pre-commit unless a future decision explicitly approves that exact hook.

## Pre-Push

Pre-push may run heavier local verification:

```bash
cargo xtask ci
cargo xtask audit-docs
cargo xtask secrets
```

## Before Merge

Before merging substantial changes:

```bash
cargo xtask verify
```

For crate dependency changes, also review [contracts/crates-and-dependencies.md](./contracts/crates-and-dependencies.md) and [specs/crates-and-dependencies.md](./specs/crates-and-dependencies.md). When `cargo xtask audit-deps` exists, dependency changes should run it before merge.

Run narrower commands when debugging:

```bash
cargo xtask fmt-check
cargo xtask check
cargo xtask clippy
cargo xtask nextest
cargo xtask test
```

## Release Gate

Release verification must use normal production codegen unless a decision explicitly changes that path. Cranelift is local debug acceleration, not a release shortcut.
