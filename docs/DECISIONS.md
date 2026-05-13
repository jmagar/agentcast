---
title: "Design Decisions"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/jmagar/jmagar-lab.xml"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Design Decisions

This file records accepted architecture decisions. Keep entries short.

## 0001 — AgentCast starts as a new repo

AgentCast starts as a separate repo instead of developing inside Labby.

The separate repo should still extract and generalize proven Lab code and patterns when they fit AgentCast boundaries.

Reason:

- Labby is a homelab control-plane product.
- AgentCast is an agent runtime/registry/gateway product.
- Developing directly inside Labby would carry service-integration gravity into the wrong product.
- Extracting generic Lab runtime/gateway/registry/API/CLI/UI patterns gives AgentCast a faster, evidence-backed bootstrap.

Sources:

- live repo: `../lab`.
- captured repo source of truth: `docs/references/jmagar/jmagar-lab.xml`.
- extraction guide: `docs/plans/extract-crates/README.md`.

## 0002 — Runtime is the main business layer

AgentCast does not start with a generic `dispatch` crate.

Reason:

- Labby needed dispatch for many homelab service actions.
- AgentCast’s central behavior is runtime/session/workspace orchestration.
- `agent-runtime` is the correct initial business layer.

## 0003 — Protocol crates are adapters

`agent-acp` and `agent-mcp` translate protocol-specific behavior into AgentCast internal models.

Reason:

- UI/API/CLI should not leak raw ACP/MCP SDK types.
- AgentCast needs a stable internal vocabulary across protocols.

## 0004 — Marketplace produces plans, runtime applies them

Marketplace code plans installs. Runtime applies installs.

Reason:

- planning should be previewable and testable.
- mutation belongs to the runtime layer.
- CLI/API/UI should share the same behavior.

## 0005 — MCP crate is a thin shim

`agent-mcp` is not the gateway policy layer.

Reason:

- MCP SDK integration should stay boring and replaceable.
- exposure, filtering, routing, and bridge policy belong in `agent-gateway`.
- process/session/workspace ownership belongs in `agent-runtime`.

## 0006 — MVP is a Raycast-like MCP launcher

AgentCast starts as a launcher/control surface for MCP servers.

The first product loop is:

1. discover configured MCP servers
2. index their tools as launcher actions
3. search actions quickly
4. invoke a tool
5. display structured results
6. install more MCP servers from the official MCP Registry

The broader agent runtime/operating plane remains the long-term direction, but it must not distort the MVP.

## 0007 — End-state contracts stay, MVP controls sequencing

The docs keep the full AgentCast end-state: ACP sessions, marketplace-compatible artifacts, loadouts, remote transports, and agent operating-plane workflows.

Reason:

- the broader architecture affects crate boundaries now.
- future-shape contracts prevent short-term code from closing off intended capabilities.
- `MVP.md` is the first-slice scope gate, not a replacement for the architecture docs.

If an end-state contract conflicts with MVP delivery, label that part post-v0 instead of deleting it.

## 0008 — No pre-commit hooks without explicit approval

AgentCast endorses commit early, commit often.

Pre-commit hooks must not exist by default. Checks belong at push/CI time unless a specific pre-commit hook is explicitly approved.

Policy:

- no pre-commit hook may exist unless this file contains an explicit decision approving that exact hook.
- the decision must quote the user approving that hook.
- if there is no such entry in `docs/DECISIONS.md`, the pre-commit hook should not exist.
- agents and contributors may still suggest a pre-commit hook when they believe it is worth an exception.
- suggested hooks are not approved until this file records explicit user approval for the exact hook.

User requirement:

> ANY AND ALL PRE-COMMIT HOOKS MUST BE PERSONALLY AND EXPLICITLY AGREED UPON BY ME - IF THERE IS NOT AN ENTRY IN DECISIONS.md QUOTING ME SAYING ITS OKAY - THEN THE PRE-COMMIT HOOK SHOULD NOT EXIST

Reason:

- commits should stay cheap.
- local iteration should not be blocked by commit-time checks.
- full verification should happen before push or in CI/release gates.

Approved pre-commit hook manager:

- `lefthook`

Approved pre-commit hooks:

- staged secret scan.
- staged conflict-marker check.
- staged large-file guard.
- staged TOML/YAML/JSON syntax check.
- staged line-ending and executable-bit sanity check.

User approval:

> well - you suggested 5 hooks - im approving those - update the decisions log. and then install / setup lefthook + the hooks - also what about taplo?

Notes:

- Taplo is approved as the TOML checker inside the staged TOML/YAML/JSON syntax hook.
- Heavy Cargo checks remain pre-push/CI/explicit verification commands, not pre-commit.

## 0009 — Use cargo-nextest for Rust test execution

AgentCast should use `cargo nextest` as the preferred Rust test runner once tests exist.

Policy:

- use `cargo nextest run --workspace` for normal workspace test execution.
- `cargo test --workspace` remains acceptable when validating doctests, compatibility, or when nextest is unavailable.
- CI/push verification should prefer nextest for speed and clearer failure reporting.

Reason:

- nextest is faster and more ergonomic for larger Rust workspaces.
- faster feedback supports the commit-early, commit-often workflow.
- test execution should stay aligned with modular crate boundaries.

## 0010 — Use Cranelift for fast local debug builds when practical

AgentCast should support Cranelift-backed local debug builds when practical.

Policy:

- Cranelift is approved for local/dev debug acceleration.
- release builds and CI release verification should keep the normal production codegen path unless explicitly changed.
- any repo config for Cranelift must be documented and easy to disable.
- the opt-in local wrapper is `cargo xtask check-cranelift`.

Reason:

- faster local Rust builds improve iteration speed.
- this supports the "everything must be fast" principle without changing production build semantics.

## 0011 — Keep source modules small and focused

AgentCast uses explicit code-size guardrails to keep modules simple and agent-operable.

Policy:

- target source module size: 250 non-test lines or less.
- split-review threshold: more than 400 non-test lines.
- documented-exception threshold: more than 600 non-test lines.
- hard review threshold: more than 800 total lines.
- target function size: 60 lines or less.
- documented function exception threshold: more than 100 lines.

Reason:

- small focused modules are easier for humans and LLM agents to understand and edit safely.
- the thresholds are AgentCast guardrails, not universal Rust standards.
- large files often indicate hidden multiple responsibilities.

## 0012 — Use source-side test sidecars by default

AgentCast tests should use source-side sidecar modules by default.

Policy:

- do not put test bodies inline in production implementation files.
- implementation files may include only the minimal `#[cfg(test)] mod tests;` declaration when needed.
- place test bodies in source-side sidecars such as `src/foo/tests.rs`, `src/foo_test.rs`, or `src/foo/tests/*.rs`.
- use `crates/*/tests/*.rs` integration tests only for public API, CLI, API, protocol, or full workflow behavior.
- do not make internal functions public only to test them from integration tests.

Reason:

- implementation files stay focused.
- internal behavior remains testable without widening visibility.
- source-side sidecars preserve access to private module internals while reducing production-file clutter.

## 0013 — Use cargo xtask for repository automation

AgentCast should expose common repository automation through `cargo xtask`.

Policy:

- `cargo xtask` is the contributor-facing entrypoint for local repository tasks.
- task implementations should wrap existing checks and scripts rather than duplicating business logic.
- hook installation remains explicit through `cargo xtask hooks` or `cargo xtask install-hooks`.
- `cargo xtask ci` is a local push/CI-sized suite: format check, workspace check, clippy, and nextest.
- `cargo xtask verify` is the broader completion gate and adds `cargo test --workspace`.
- `cargo xtask audit-docs` and `cargo xtask secrets` are explicit integrity checks for docs and secret leakage.
- docs refresh and review workflows remain in `scripts/` and are invoked through xtask wrappers.

Reason:

- a Cargo-native task surface is discoverable in a Rust workspace.
- wrappers keep command spelling stable as underlying tools change.
- explicit xtasks preserve the project policy that heavy verification is run intentionally, before push, in CI, or on request.

## 0014 — Exclude upstream references from secret scans

Gitleaks should ignore `docs/references/**`.

Policy:

- `.gitleaks.toml` must allowlist `docs/references/**`.
- authored docs and code remain in scope for secret scanning.
- upstream reference snapshots must not be rewritten to satisfy local secret-scanner heuristics.

User approval:

> do all 10 of these and yes to gitleaks config - make it ignore docs/references since thats where we got 145 false positives from

Reason:

- `docs/references/**` is a raw upstream/reference corpus.
- false positives in copied upstream snapshots should not block commits.
- preserving upstream snapshots matters more than normalizing their contents.
