---
title: "Testing Contract"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/fastmcp/docs/inspecting.mdx"
  - "docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0172-modelcontextprotocol-io-registry-faq.md"
  - "docs/references/mcporter/docs/livetests.md"
related:
  - "docs/MVP.md"
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Testing Contract

AgentCast must keep tests aligned with crate boundaries.

MVP tests should focus first on the MCP launcher runtime path in [MVP.md](./MVP.md).

## Required Checks

Before merging:

```bash
cargo xtask verify
```

`cargo nextest run --workspace` is the preferred Rust test command per `docs/DECISIONS.md` decision 0009. Use `cargo test --workspace` when validating doctests, compatibility, or when nextest is unavailable.

`cargo xtask ci` runs the push/CI-sized local suite: format check, workspace check, clippy, and nextest.

`cargo xtask verify` is the broader "before declaring completion" command. It runs `cargo xtask ci` plus `cargo test --workspace` for doctest and compatibility coverage.

Use task-specific commands such as `cargo xtask fmt-check`, `cargo xtask check`, `cargo xtask clippy`, `cargo xtask nextest`, and `cargo xtask test` when narrowing a failure.

`cargo xtask audit-docs` checks authored docs frontmatter, local markdown links, `upstream_refs`, `related` paths, and rejects stale `modified_at_commit: "unborn"` values.

`cargo xtask secrets` runs the repo secret scan with `gitleaks detect --no-banner` and the repo `.gitleaks.toml` allowlist.

## Hook Policy

AgentCast endorses commit early, commit often.

No pre-commit hook is allowed by default.

Any pre-commit hook requires an explicit `docs/DECISIONS.md` entry quoting user approval for that exact hook.

Agents and contributors may recommend lightweight pre-commit hooks when they are worth an exception. Recommendation is not approval; the hook must not be installed until `docs/DECISIONS.md` records explicit user approval for that exact hook.

Currently approved pre-commit hooks are listed in `docs/DECISIONS.md` decision 0008. At the time of writing, the approved set is:

- staged secret scan.
- staged conflict-marker check.
- staged large-file guard.
- staged TOML/YAML/JSON syntax check, with Taplo for TOML.
- staged line-ending and executable-bit sanity check.

Examples of checks that belong before push, in CI, or in explicit verification commands unless separately approved as a pre-commit hook:

- full workspace builds.
- full test suites.
- clippy over the workspace.
- network checks.
- registry refreshes.
- generated-doc rewrites.
- formatting checks.
- metadata checks.
- any other commit-time hook.

Heavy verification belongs before push, in CI, or in explicit release/check commands.

## Test Ownership

- `agent-core`: pure unit tests.
- `agent-protocol`: serialization snapshot tests.
- `agent-acp`: ACP adapter tests using fixtures.
- `agent-mcp`: MCP adapter tests using mock servers.
- `agent-registry`: registry normalization fixtures.
- `agent-marketplace`: install-plan snapshot tests.
- `agent-runtime`: lifecycle and process supervision tests.
- `agent-gateway`: routing/exposure/collision tests.
- `agent-api`: route tests using in-memory state.
- `agent-cli`: CLI parse/render tests.

Protocol fixtures must be copied into AgentCast-owned fixtures or generated from local reference snapshots. Tests must not depend on live upstream docs paths at runtime.

## Test Placement

Tests should use source-side sidecar modules by default.

This is an accepted project decision. See `docs/DECISIONS.md` decision 0012.

Do not put test bodies inline in production implementation files. Keep implementation files focused.

Preferred pattern:

- `src/foo.rs` contains implementation and, if needed, only `#[cfg(test)] mod tests;`
- `src/foo/tests.rs`, `src/foo_test.rs`, or `src/foo/tests/*.rs` contains the test bodies.

Use `crates/*/tests/*.rs` integration tests only for public API, CLI, API, protocol, or full workflow behavior.

Do not make internal functions public just to test them from integration tests. Use source-side sidecars for internal behavior.

## MVP Acceptance Tests

Before calling the MCP launcher MVP complete, tests should cover:

- config parsing for local stdio MCP servers.
- MCP list-tools normalization into launcher actions.
- gateway collision handling.
- deterministic action invocation.
- structured success and error result rendering.
- registry entry normalization from fixtures.
- MCP server install-plan generation from fixtures.

## Snapshot Policy

Use snapshots for:

- normalized registry entries.
- install plans.
- API envelopes.
- runtime event streams.
- CLI JSON output.

Do not snapshot nondeterministic fields unless normalized.

## Fixture Policy

Source-side unit test fixtures should live next to the module that owns the behavior:

```txt
crates/agent-acp/src/session/fixtures/
crates/agent-mcp/src/stdio/fixtures/
crates/agent-registry/src/mcp/fixtures/
```

Use `crates/*/tests/fixtures/` only for public integration tests that exercise crate APIs, CLI/API surfaces, protocol compatibility, or full workflows.

## No Network Tests By Default

Default tests must not require internet access. Live registry tests should be ignored or gated behind env vars.

Live MCP/registry smoke tests may use mcporter, FastMCP, or direct clients when useful, but they must be opt-in and clearly separated from deterministic CI.

## Upstream References Checked

- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`: MCP tool fixtures and invocation behavior.
- `docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md`: pagination fixture expectations.
- `docs/references/mcp/docs/markdown/0172-modelcontextprotocol-io-registry-faq.md`: registry preview risk for live tests.
- `docs/references/mcporter/docs/livetests.md`: opt-in live MCP test precedent.
- `docs/references/fastmcp/docs/inspecting.mdx`: inspection tooling precedent for generated reports.
