---
title: "Development Guide"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/jmagar/jmagar-lab.xml"
related:
  - "docs/CODE_ORGANIZATION.md"
  - "docs/contracts/crates-and-dependencies.md"
  - "docs/DECISIONS.md"
  - "docs/DEV_SPEED.md"
  - "docs/QUALITY_GATES.md"
  - "docs/TESTING.md"
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "local repo tooling policy"
---

# Development Guide

AgentCast development should optimize for simple local operation, small modules, fast verification, and agent-friendly surfaces.

## Setup

Expected local tools:

```bash
cargo nextest --version
lefthook version
gitleaks version
taplo --version
just --version
```

Install hooks explicitly:

```bash
cargo xtask hooks
```

## Daily Commands

Use Cargo-native task wrappers first:

```bash
cargo xtask check
cargo xtask nextest
cargo xtask ci
cargo xtask verify
```

Use `just` aliases when preferred:

```bash
just check
just test
just ci
just verify
```

## Config Boundary

Keep `.env` small. Use it only for secrets, tokens, endpoint URLs, and runtime process environment values.

Put durable non-secret settings in `config.toml`.

## Code Shape

Keep modules small and focused. Follow [CODE_ORGANIZATION.md](./CODE_ORGANIZATION.md) for file/function thresholds.

Place dependencies according to [contracts/crates-and-dependencies.md](./contracts/crates-and-dependencies.md). Shared dependency versions belong in root `[workspace.dependencies]`.

Use source-side test sidecars by default. Do not make internals public just to test them.

## References

Consult [references/](./references/) during major planning sessions. Treat repopacks as source-of-truth snapshots for their upstream repos, while preserving them as raw references.

Do not edit `docs/references/**` as part of normal docs work.
