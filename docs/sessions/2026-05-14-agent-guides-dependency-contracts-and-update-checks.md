---
title: "Agent Guides, Dependency Contracts, And Update Checks"
doc_type: "session"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs: []
related:
  - "AGENTS.md"
  - "apps/AGENTS.md"
  - "crates/AGENTS.md"
  - "docs/contracts/crates-and-dependencies.md"
  - "docs/specs/crates-and-dependencies.md"
  - "scripts/check-dependency-updates.sh"
last_reviewed: "2026-05-14"
last_modified: "2026-05-14"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "e0bd04f"
review_basis: "session save from current workspace state"
---

# Agent Guides, Dependency Contracts, And Update Checks

Saved: 2026-05-14T02:05:24-04:00

## Repo State

- Repository: `/home/jmagar/workspace/agentcast`
- Branch: `main`
- Upstream: `origin/main`
- HEAD at save time: `e0bd04f Update docs commit metadata`
- Branch status at save time: `main...origin/main`
- Worktree status at save time: dirty with uncommitted docs/tooling changes.

Recent commits:

```txt
e0bd04f (HEAD -> main, origin/main) Update docs commit metadata
b941533 Add AgentCast docs and repo tooling
fe10007 Merge remote-tracking branch 'origin/main'
48ef47d Initial AgentCast workspace
6db874b Initial commit
```

## Work Completed

### Agent Guides

Created local agent guidance files:

- `AGENTS.md`
- `crates/AGENTS.md`
- `apps/AGENTS.md`

The root guide covers repo-wide development workflow, docs-as-contracts, product boundaries, xtask usage, hook policy, code shape, testing, docs, and secrets.

The crates guide covers dependency direction, crate ownership, adapter rules, surface rules, module shape, source-side tests, and verification.

The apps guide covers deployable app surface boundaries, thin entrypoint rules, current app directories, frontend/desktop protocol boundaries, and verification.

### Crates And Dependencies Contract/Spec

Added:

- `docs/contracts/crates-and-dependencies.md`
- `docs/specs/crates-and-dependencies.md`

The contract defines:

- workspace dependency policy.
- dependency direction.
- crate layer categories.
- external SDK ownership.
- low-level crate restrictions.
- surface crate restrictions.
- adapter restrictions.
- exception requirements.
- verification expectations.

The spec defines future `cargo xtask audit-deps` behavior:

- manifest inputs.
- initial failure checks.
- crate layer classification.
- output format.
- exception allowlist rules.
- future integration points.

Linked the new contract/spec from:

- `docs/README.md`
- `docs/contracts/README.md`
- `docs/specs/README.md`
- `docs/CRATE_BOUNDARIES.md`
- `docs/DEVELOPMENT.md`
- `docs/QUALITY_GATES.md`
- `AGENTS.md`
- `crates/AGENTS.md`

### Dependency Update Script

Added:

- `scripts/check-dependency-updates.sh`

Registered xtask wrapper:

- `cargo xtask check-deps`

The script reports:

- lockfile-compatible updates with `cargo update --dry-run`.
- direct root `[workspace.dependencies]` latest versions from crates.io.

Options:

- `--skip-search`
- `--fail-on-updates`

The script is portable enough for other Rust repos on this machine when run under Bash. Known limitation: renamed dependencies using `package = "..."` may need enhancement because the script currently checks the dependency key against crates.io.

Observed output during validation:

- Compatible lockfile updates were available for `aws-lc-rs`, `aws-lc-sys`, `pin-project`, `pin-project-internal`, `rmcp`, and `rmcp-macros`.
- Direct manifest review candidate: `rmcp` requirement `1.6`, latest `1.7.0`.

## Verification

Commands run successfully during this session:

```bash
cargo xtask audit-docs
cargo xtask check-deps --skip-search
cargo xtask check-deps
bash -n scripts/check-dependency-updates.sh
cargo xtask fmt-check
cargo check -p xtask
```

`cargo xtask audit-docs` checked 79 authored markdown files after the new contract/spec docs were added.

## Current Uncommitted Files

Modified tracked files at save time:

```txt
docs/CRATE_BOUNDARIES.md
docs/DEVELOPMENT.md
docs/QUALITY_GATES.md
docs/README.md
docs/contracts/README.md
docs/specs/README.md
xtask/src/task.rs
```

Untracked files at save time:

```txt
AGENTS.md
apps/AGENTS.md
crates/AGENTS.md
docs/contracts/crates-and-dependencies.md
docs/sessions/2026-05-13-agentcast-repo-tooling-and-docs-push.md
docs/sessions/2026-05-14-agent-guides-dependency-contracts-and-update-checks.md
docs/specs/crates-and-dependencies.md
scripts/check-dependency-updates.sh
```

## Open Questions

- Whether to implement `cargo xtask audit-deps` now or wait until dependency policy starts changing more often.
- Whether `scripts/check-dependency-updates.sh` should grow support for renamed dependencies with `package = "..."`.
- Whether dependency update reporting should be included in `cargo xtask verify` or kept as an explicit manual check.
- Whether to commit the current AGENTS/dependency-contract/update-script work as one docs/tooling commit or split it by topic.
