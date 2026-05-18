---
title: "Full Review Remediation And Sccache"
doc_type: "session"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs: []
related:
  - "FULL_REVIEW_ISSUES.md"
  - ".full-review/05-final-report.md"
  - "docs/DEVELOPMENT.md"
  - "docs/SECURITY.md"
  - "docs/contracts/crates-and-dependencies.md"
last_reviewed: "2026-05-18"
last_modified: "2026-05-18"
modified_on_branch: "review-remediation/full-review-issues"
modified_at_version: "0.1.0"
modified_at_commit: "f7e6166"
review_basis: "session save from full review remediation and PR verification"
---

# Full Review Remediation And Sccache

Saved: 2026-05-18T01:44:00-04:00

## Repo State

- Repository: `/home/jmagar/workspace/agentcast`
- Worktree: `/home/jmagar/workspace/agentcast/.worktrees/full-review-issues`
- Branch: `review-remediation/full-review-issues`
- PR: `https://github.com/jmagar/agentcast/pull/2`
- Latest pushed commit at save time: `f7e6166 Address full review follow-up findings`
- Tracking issue: `agentcast-ate`, closed after verification.

## Work Completed

- Added `FULL_REVIEW_ISSUES.md` and preserved `.full-review/` source artifacts.
- Hardened protected MCP auth defaults, JWT time/issuer validation, Origin/session handling, and DELETE transport validation.
- Added server-side protected MCP static bearer token wiring through `--protected-mcp-bearer-token` / `AGENTCAST_PROTECTED_MCP_BEARER_TOKEN`.
- Added bounded collection limits for gateway list endpoints.
- Improved runtime startup/discovery/shutdown behavior and released the protected HTTP client cache lock before awaited upstream operations.
- Split `agent-server` composition out of `main.rs` into focused modules and source-side tests.
- Added `cargo xtask audit-deps` with an explicit `agent-server` -> `agent-mcp` v0 stdio gateway exception.
- Updated API/security/development/dependency docs to match implemented v0 behavior.

## Review Follow-Ups Addressed

The review pass found five actionable issues. The branch now addresses them:

- protected MCP server routes require a real static bearer verifier configuration.
- protected HTTP cached clients are cloned out of the cache before awaited calls.
- protected MCP DELETE validates transport headers and Origin before session removal.
- dependency audit encodes the documented `agent-server` adapter exception.
- list endpoints use bounded limits consistently.

## Sccache Resolution

The pre-push failure was not an AgentCast code failure. The host had a user-level
`sccache.service` using `SCCACHE_SERVER_UDS=/tmp/sccache-jmagar.sock`, while
Cargo clients were not inheriting that socket variable. Cargo builds could spawn
an unmanaged default `sccache` daemon with separate stats and cache errors.

Fix applied:

- added `SCCACHE_SERVER_UDS = "/tmp/sccache-jmagar.sock"` under
  `/home/jmagar/.cargo/config.toml` `[env]`.
- stopped the unmanaged default daemon with `sccache --stop-server`.
- verified only the systemd-managed `/usr/bin/sccache` process remained.

Evidence after the fix:

- normal Cargo check with sccache enabled completed without `RUSTC_WRAPPER=`.
- managed service stats increased and still showed `Cache errors 0`.
- PR pre-push hook completed `audit-docs`, `ci`, and `secrets` successfully.

## Verification

Passed:

- `cargo fmt --all --check`
- `git diff --check`
- focused `cargo test -p agent-api -p agent-server -p agent-runtime -p xtask`
- `cargo run -p xtask -- audit-deps`
- `cargo run -p xtask -- verify`
- `git push` pre-push hook: `audit-docs`, `ci`, `secrets`

## Open Questions

- `crates/agent-api/src/http.rs` remains larger than the preferred code organization threshold. It has functional fixes in this branch, but a deeper route-family split should be considered separately if the PR reviewer wants more structural cleanup before merge.
