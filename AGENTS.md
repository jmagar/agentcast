# AgentCast Agent Guide

This repo is a Rust workspace for an agent runtime, registry, marketplace, and gateway for ACP/MCP-compatible coding agents and tool systems.

Treat the docs as development contracts, not background reading. When code and docs disagree, either update the code or make an explicit docs/decision change.

## Start Here

Read these before non-trivial work:

- `docs/MVP.md` for the current v0 implementation slice.
- `docs/PRINCIPLES.md` for product and engineering decision filters.
- `docs/CRATE_BOUNDARIES.md` before placing code.
- `docs/contracts/crates-and-dependencies.md` before adding or moving dependencies.
- `docs/CODE_ORGANIZATION.md` before adding modules or large files.
- `docs/QUALITY_GATES.md` and `docs/TESTING.md` before claiming completion.
- `docs/DEVELOPMENT.md` for local command workflow.

Use `docs/references/**` as read-only upstream/reference material. Do not edit reference snapshots as part of normal docs or code changes.

## Product Boundary

AgentCast owns the MCP launcher/runtime MVP first. ACP, multi-agent loadouts, broader marketplace surfaces, remote/fleet behavior, and AgentHub-style features are end-state or post-v0 unless `docs/MVP.md` explicitly promotes them.

Do not import Labby or homelab product gravity into AgentCast. Lab is a bootstrap source for generic patterns only.

## Development Commands

Prefer the Cargo-native task surface:

```bash
cargo xtask check
cargo xtask nextest
cargo xtask ci
cargo xtask verify
```

Use focused commands while iterating and `cargo xtask verify` before declaring substantial work complete.

Other useful tasks:

```bash
cargo xtask audit-docs
cargo xtask secrets
cargo xtask hooks
cargo xtask refresh-docs
cargo xtask review-docs
```

`cargo xtask ci` is the pre-push-sized suite: format check, workspace check, clippy, and nextest.

`cargo xtask verify` adds `cargo test --workspace` for doctest and compatibility coverage.

## Hook Policy

Pre-commit hooks are tightly controlled. Do not add or expand pre-commit hooks unless `docs/DECISIONS.md` records explicit user approval for that exact hook.

The approved pre-commit set is:

- staged secret scan.
- staged conflict-marker check.
- staged large-file guard.
- staged TOML/YAML/JSON syntax check, with Taplo for TOML.
- staged line-ending and executable-bit sanity check.

Heavier checks belong in pre-push, CI, or explicit xtasks.

## Code Shape

Keep modules small and single-purpose.

Default guardrails:

- target source module size: 250 non-test lines or less.
- split-review threshold: more than 400 non-test lines.
- documented-exception threshold: more than 600 non-test lines.
- hard review threshold: more than 800 total lines.
- target function size: 60 lines or less.

Avoid `utils.rs`, `helpers.rs`, `manager.rs`, and broad catch-all modules unless the responsibility is genuinely narrow and documented.

## Dependencies

Follow `docs/contracts/crates-and-dependencies.md`.

Shared external dependency versions belong in root `[workspace.dependencies]`. Low-level model crates must stay dependency-light, protocol SDKs belong in their adapter crates, and surface crates must not own business policy.

## Testing

Use source-side test sidecars by default. Do not put test bodies inline in production implementation files, and do not make internals public just to test them.

Default pattern:

- `src/foo.rs` contains implementation and, when needed, only `#[cfg(test)] mod tests;`
- `src/foo/tests.rs`, `src/foo_test.rs`, or `src/foo/tests/*.rs` contains test bodies.

Use `crates/*/tests/*.rs` only for public API, CLI, API, protocol, or full workflow behavior.

Default tests must not require network access. Live registry or upstream checks should be ignored or gated behind environment variables.

## Docs

Authored docs should keep unified frontmatter valid. Run:

```bash
cargo xtask audit-docs
```

When editing docs, update local links and `related`/`upstream_refs` paths. Do not leave `modified_at_commit: "unborn"`.

## Secrets

Run `cargo xtask secrets` for an explicit full-history secret scan. The repo uses `.gitleaks.toml`; authored docs and code remain in scope.
