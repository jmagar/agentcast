---
title: "Documentation Instructions"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/jmagar/jmagar-aurora-design-system.xml"
  - "docs/references/jmagar/jmagar-lab.xml"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Documentation Instructions

This directory contains AgentCast development contracts, implementation specs, reference material, and working records.

Agents are first-class citizens in this repo. Write docs, contracts, specs, commands, and examples assuming both humans and LLM agents will operate every surface. Prefer structured, discoverable, paginated, filterable, and JSON-capable interfaces.

## Documentation Layers

Use the right layer for the job:

- `docs/*.md` contains orientation, architecture narrative, decisions, roadmap, and cross-cutting guidance.
- `docs/contracts/*.md` contains stable behavioral contracts. A contract defines what implementations must obey and should be externally testable.
- `docs/specs/*.md` contains implementation specs. A spec explains how a specific slice should be built, including modules, flows, edge cases, and acceptance criteria.
- `docs/references/` contains current reference snapshots for protocols, SDKs, CLIs, and repos AgentCast uses, plus project-origin seed material.
- `docs/plans/` contains planning artifacts and task breakdowns.
- `docs/reports/` contains audits, investigations, reviews, and generated findings.
- `docs/sessions/` contains saved session notes and handoff records.
- `docs/generated/` contains generated docs or machine-produced artifacts that should be treated as derived output.

## Naming

Rust crates use the `agent-*` package and directory convention, for example `crates/agent-mcp` and `crates/agent-runtime`.

`AgentCast` remains the product name, and `agentcast` remains the CLI/server binary name unless explicitly changed. Do not rewrite raw upstream references, repopacks, or seed transcripts only to match current crate naming; those files preserve source/provenance text.

## MVP Versus End State

AgentCast has a broad end-state architecture: ACP sessions, MCP gateway behavior, registry/marketplace discovery, loadouts, remote execution, and agent operating-plane workflows.

The first implementation slice is narrower and is governed by `docs/MVP.md`: an MCP-first launcher/runtime.

When editing docs:

- keep end-state contracts instead of deleting them.
- label post-v0 behavior clearly when it is not part of the MCP launcher MVP.
- do not let ACP, loadouts, stash, fleet, AgentHub, desktop UI, or remote execution become required for v0 unless `docs/MVP.md` is explicitly updated.
- preserve the rule that CLI/API/MCP/UI surfaces are thin over shared runtime/domain behavior.

## Lab Bootstrap Source

AgentCast is a new repo, but it is expected to bootstrap by extracting and generalizing code and patterns from Lab.

Use:

- `../lab` when the live local repo is available.
- `docs/references/jmagar/jmagar-lab.xml` as the source-of-truth Lab repopack snapshot at the captured revision.
- `docs/references/jmagar/jmagar-aurora-design-system.xml` as the source-of-truth Aurora repopack snapshot at the captured revision.

Follow `docs/plans/extract-crates/README.md` when porting or designing from Lab. Do not treat seed transcript claims as sufficient evidence for what Lab contains; inspect the live repo or repopack.

Extraction should generalize patterns into AgentCast crate boundaries. Do not import Lab-specific homelab service workflows into AgentCast core.

## Contracts

Contracts should be concise and normative.

Use contracts for:

- data shapes and required fields.
- stable IDs and naming rules.
- lifecycle/state-machine requirements.
- error envelopes and error kinds.
- command/API output invariants.
- install-plan invariants.
- security and confirmation requirements.
- testable acceptance criteria.

Avoid in contracts:

- long implementation discussion.
- speculative alternatives.
- stale transcript excerpts.
- repo-specific debugging notes.

If a contract changes, update dependent specs and decisions in the same patch when practical.

## Specs

Specs should be concrete enough to guide implementation.

Use specs for:

- phased implementation steps.
- crate/module ownership for a feature.
- trait/function sketches.
- internal data flow.
- edge cases.
- fixture plans.
- verification commands.

Specs may evolve more often than contracts. If a spec contradicts a contract, the contract wins unless the contract is explicitly revised.

## Working Artifact Directories

Use working artifact directories intentionally:

- Put durable implementation plans in `docs/plans/`.
- Put audits, review results, research writeups, and investigation reports in `docs/reports/`.
- Put run/session handoffs in `docs/sessions/`.
- Put generated output in `docs/generated/` unless a generated artifact is explicitly promoted into a contract, spec, or top-level doc.

Artifacts in `plans`, `reports`, `sessions`, and `generated` can inform contracts and specs, but they do not override them. If a working artifact contains an accepted requirement, promote that requirement into `docs/contracts/`, `docs/specs/`, `docs/DECISIONS.md`, or `docs/MVP.md`.

## References

Reference material has different authority depending on what it is:

- `docs/references/acp/`, `docs/references/mcp/`, `docs/references/claude-code/`, `docs/references/fastmcp/`, and `docs/references/mcporter/` are snapshots of external upstream documentation and repositories. Treat them as the source-of-truth reference for those upstreams as captured in this repo.
- Repomix packs are full bundled repo contents. Treat a repopack as the source-of-truth snapshot for that repository at the captured revision.
- `docs/references/jmagar/` contains user-owned repo packs and should be treated as source-of-truth snapshots for those repos at capture time.
- `docs/references/seed/` stores raw origin transcripts and product-shaping notes. Treat them as project provenance, not current policy.

Do not cite seed transcripts as current truth when a contract, spec, decision, or official reference exists. If a seed idea becomes accepted, capture it in `docs/DECISIONS.md`, `docs/MVP.md`, a contract, or a spec.

When upstream marks material as preview, draft, proposal, RFD, SEP, or otherwise unstabilized, mirror that status in AgentCast docs. Current examples include the official MCP Registry preview and ACP Streamable HTTP/WebSocket draft work.

Prefer `docs/references/` before web search. When current protocol behavior matters and the captured reference is ambiguous or suspected stale, refresh or verify against the upstream source before changing contracts.

During major planning sessions, inspect the relevant `docs/references/` corpus first. If active development depends on fast-moving upstream behavior, refresh the relevant references daily or record why the captured snapshot is still sufficient.

## Style

- Prefer short, direct sections with clear ownership.
- Use `v0`, `post-v0`, and `end-state` labels where scope could be confused.
- Keep examples realistic and runnable when possible.
- Keep generated or historical material out of contracts unless distilled into current requirements.
- Do not move broad architecture into specs only; top-level docs should remain the map.
