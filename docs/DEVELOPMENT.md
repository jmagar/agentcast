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
last_reviewed: "2026-05-18"
last_modified: "2026-05-18"
modified_on_branch: "review-remediation/full-review-issues"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "local repo tooling policy"
---

# Development Guide

AgentCast development should optimize for simple local operation, small modules, fast verification, and agent-friendly surfaces.

## Setup

Expected local tools:

```bash
rustup --version
cargo nextest --version
lefthook version
gitleaks version
cargo deny --version
taplo --version
just --version
```

Expected Rustup setup:

```bash
rustup toolchain install nightly
rustup component add rustc-codegen-cranelift --toolchain nightly
```

Use the Cargo-native setup task to install or sync nightly/Cranelift support, verify the local toolchain, and install or sync Git hooks:

```bash
cargo xtask setup
```

Use the doctor task when you only want to verify the environment without modifying hooks:

```bash
cargo xtask doctor
```

Doctor also runs cargo metadata and a small `xtask` compile smoke test. If the
active cargo/rustc path is routed through a broken wrapper, Snap-backed rustc, or
failing `RUSTC_WRAPPER`/`sccache`, doctor should report that before longer gates.
For local debugging only, clearing the wrapper for one command can isolate the
problem:

```bash
RUSTC_WRAPPER= cargo xtask doctor
```

Install hooks explicitly when needed:

```bash
cargo xtask hooks
```

## Daily Commands

Use Cargo-native task wrappers first:

```bash
cargo xtask setup
cargo xtask doctor
cargo xtask check
cargo xtask nextest
cargo xtask nextest-ci
cargo xtask file-size
cargo xtask deny
cargo xtask audit-deps
cargo xtask ci
cargo xtask verify
```

Use `just` aliases when preferred:

```bash
just setup
just doctor
just check
just test
just test-ci
just file-size
just deny
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
