---
title: "AgentCast Docs Principles, MVP, And Registry Session"
doc_type: "session"
status: "historical"
owner: "agentcast"
audience:
  - "maintainers"
  - "contributors"
scope: "historical"
source_of_truth: false
upstream_refs: []
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "historical note reviewed against current local docs/references snapshot"
historical: true
---

# AgentCast Docs Principles, MVP, And Registry Session

Date: 2026-05-13

## Scope

This session refined AgentCast's docs around project philosophy, MVP expectations, agent-friendly surfaces, test policy, code organization, and MCP Registry behavior.

The user emphasized that AgentCast should optimize for:

- fast, simple, composable, reusable, portable, organized, logical design.
- agents as first-class operators across CLI, API, MCP, web, desktop, logs, docs, and config.
- operator simplicity as agent operating simplicity.
- context/token discipline through pagination, filtering, concise output, and structured JSON/DTOs.
- fully self-hosted operation with no required hosted control plane.
- one binary, one local port when serving, local SQLite by default, PostgreSQL only as a later optional backing store.
- no pre-commit hooks unless explicitly approved in `docs/DECISIONS.md`.

Scope E audit note, 2026-05-13: this is a historical session record of AgentCast doc decisions.
Current technical claims about MCP registry behavior, MCP transports, ACP session features,
FastMCP/mcporter-style CLI behavior, and Claude Code plugin/marketplace packaging should be
cross-checked against `docs/references/mcp/`, `docs/references/acp/`,
`docs/references/fastmcp/`, `docs/references/mcporter/`, and `docs/references/claude-code/`
before implementation work relies on them.

## Created

- `docs/PRINCIPLES.md`
  - durable project principles extracted from seed conversations and user direction.
  - includes agent-first surfaces, context/token discipline, simplicity as core IP, small focused modules, self-hosted defaults, FastMCP/mcporter parity, registry-first behavior, and "everything must be fast."

- `docs/CODE_ORGANIZATION.md`
  - module/file/function size guardrails.
  - thresholds are project guardrails, not universal Rust standards:
    - target source module: 250 non-test lines or less.
    - split-review threshold: more than 400 non-test lines.
    - documented exception: more than 600 non-test lines.
    - hard review: more than 800 total lines.
    - target function: 60 lines or less.
    - function exception: more than 100 lines.

- `docs/OBSERVABILITY.md`
  - local/self-hosted observability expectations.
  - structured runtime events for config discovery, MCP lifecycle, registry sync/cache/index, install plans, confirmations, errors, and retries.
  - pagination/filtering expectations for large event streams.

## Updated

- `docs/README.md`
  - added `PRINCIPLES.md`, `CODE_ORGANIZATION.md`, and `OBSERVABILITY.md` to the read-first docs.

- `docs/AGENTS.md`
  - added that agents are first-class citizens in the repo.
  - reinforced structured, discoverable, paginated, filterable, JSON-capable surfaces.
  - added guidance to consult and refresh `docs/references/` during major planning when upstream behavior is fast-moving.

- `docs/MVP.md`
  - expanded MVP to lean on FastMCP/mcporter-style foundations:
    - discover configured MCP servers from Claude Code, Codex, Gemini CLI, project MCP configs, and AgentCast config.
    - import raw MCP JSON config.
    - dedupe server inventory.
    - list tools/resources/prompts.
    - read resources.
    - call tools deterministically.
    - expose the same behavior through CLI, API, and MCP tools.
  - added one-line install, configure-nothing, under-2-minute setup target, and 5-minute cold-start hard max.
  - reframed registry MVP as a local MCP Registry aggregator/subregistry with cache/index/freshness/status behavior.

- `docs/CLI.md` and `docs/contracts/cli.md`
  - added `ac` short commands.
  - added server/tool/resource/prompt list and resource read commands.
  - required `--json`, pagination, filtering, bounded output, and stable continuation metadata.

- `docs/API.md`, `docs/contracts/api.md`, and `docs/specs/api.md`
  - added MCP server/tool/resource/prompt routes.
  - required pagination/filtering for collection routes.
  - kept API thin over runtime/gateway behavior.

- `docs/PROTOCOLS.md`, `docs/MCP_RUNTIME.md`, `docs/contracts/mcp-runtime.md`, and `docs/specs/mcp-runtime.md`
  - added AgentCast MCP surface pagination/filtering requirements.
  - added tool/resource/prompt discovery expectations.
  - added initial MCP server tool-shape guidance for list/search tools.

- `docs/ERRORS.md` and `docs/contracts/errors.md`
  - tightened informative error requirements.
  - errors should include relevant IDs, retryability/next-action hints when available, and enough context for an agent to choose the next step.

- `docs/REGISTRY.md`, `docs/contracts/registry.md`, and `docs/specs/registry.md`
  - reframed v0 registry as a local MCP Registry aggregator/subregistry.
  - added official-registry-compatible OpenAPI shape as compatibility target.
  - added cursor pagination, incremental refresh when supported, local persistence, local index/search, freshness/status metadata, namespaced `_meta` curation, and offline cached search.

- `docs/SCHEMA_UX.md` and `docs/contracts/schema-ux.md`
  - reinforced schema-generated interfaces and avoidance of arbitrary custom frontend extension APIs.

- `docs/DECISIONS.md`
  - added decision 0008: no pre-commit hooks without explicit approval.
  - decision quotes the user's requirement that any pre-commit hook must be explicitly approved in `DECISIONS.md`.
  - clarified that hooks may still be suggested, but are not approved until recorded there.

- `docs/TESTING.md`
  - added hook policy matching decision 0008.
  - currently does not yet include the final sidecar-test placement rule.

- `docs/ARCH.md` and `docs/RUNTIME.md`
  - added one binary, one local port when serving, fully self-hosted, SQLite default, PostgreSQL optional later.

## Important Clarifications

- The earlier "built-in local capabilities" idea was rejected. AgentCast should not optimize around bundling utility apps like calculator/clipboard. The important differentiator is schema-generated extension UX from MCP servers so users do not need to be extension developers.

- File-size thresholds are not presented as industry-standard Rust limits. They are AgentCast guardrails chosen for maintainability, simplicity, and agent-operability.

- Pre-commit hooks are not forbidden from discussion. They are forbidden from existence unless the user explicitly approves the exact hook in `docs/DECISIONS.md`.

- `docs/TESTING.md` does not yet say "no tests inside actual code files; all tests should be sidecars." The user asked about this at the end of the session. Suggested rule:

```md
## Test Placement

Tests should use source-side sidecar modules by default.

Do not put test bodies inline in production implementation files. Keep implementation files focused.

Preferred pattern:

- `src/foo.rs` contains implementation and, if needed, only `#[cfg(test)] mod tests;`
- `src/foo/tests.rs` or `src/foo_test.rs` contains the test bodies.

Use `crates/*/tests/*.rs` integration tests only for public API, CLI, API, protocol, or full workflow behavior. Do not make internal functions public just to test them from integration tests.
```

## Verification

Docs-only verification was done with `rg` scans for:

- new principles and terms: simplicity, small modules, setup/cold-start, self-hosted, one binary/one port.
- pre-commit hook policy and decision text.
- registry aggregator/subregistry language, `_meta`, cursor/update checkpoint, and offline cache/index behavior.
- MCP pagination/filtering requirements.
- rejected built-in utility framing such as calculator/clipboard.

No code tests were run because this session only edited docs.

## Current Repo State

At the time of saving, `git status --short docs` showed `?? docs/`, meaning the docs tree is untracked from Git's perspective in this workspace.

If committing these docs, check `.gitignore` behavior first. Session notes under `docs/sessions/` may need `git add -f` if ignored.

## Open Questions

- Add the source-side test sidecar placement rule to `docs/TESTING.md`.
- Decide whether any specific lightweight pre-commit hook is worth proposing for explicit approval.
- Re-run a full stale-reference scan before committing if these docs remain untracked for a while.
