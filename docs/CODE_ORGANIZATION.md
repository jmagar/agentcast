---
title: "Code Organization"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs: []
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Code Organization

AgentCast is modular at the crate level and should stay modular inside each crate.

## File Size Rules

These thresholds are AgentCast guardrails, not universal Rust standards. They are intentionally stricter than what a compiler or formatter enforces because small files make this repo easier for humans and LLM agents to navigate safely.

These thresholds are an accepted project decision. See `docs/DECISIONS.md` decision 0011.

The numbers are pragmatic:

- around 250 non-test lines is usually still easy to scan in one editor pane or one agent context chunk.
- around 400 non-test lines is where a module often starts hiding multiple responsibilities.
- around 600 non-test lines should force an explicit decision because routine edits become riskier.
- around 800 total lines is a strong signal that the file is too large unless it is generated, fixture/reference material, or an upstream-shaped schema/protocol mirror.

Default thresholds:

- target source module size: 250 non-test lines or less.
- split-review threshold: more than 400 non-test lines.
- documented-exception threshold: more than 600 non-test lines.
- hard review threshold: more than 800 total lines.

Files over the split-review threshold need an explicit split/refactor note in the implementation plan or PR.

Files over the documented-exception threshold require a documented exception explaining why the module should remain together.

Files over the hard review threshold should be split unless they are:

- generated code.
- fixture data.
- vendored/reference material.
- protocol/schema mirrors where splitting would obscure the upstream structure.

## Function Size Rules

Default thresholds:

- target function size: 60 lines or less.
- documented-exception threshold: more than 100 lines.

Large functions should usually be split by phase, policy decision, parser step, or rendering step.

## Module Shape

Prefer small files with one clear responsibility.

Good module names:

- `config/import.rs`
- `registry/cache.rs`
- `registry/normalize.rs`
- `mcp/resources.rs`
- `mcp/prompts.rs`
- `gateway/collision.rs`
- `cli/pagination.rs`

Avoid broad names unless they are genuinely narrow in scope:

- `utils.rs`
- `helpers.rs`
- `manager.rs`
- `common.rs`

## Review Expectations

Code review should ask:

- can this file be understood without reading unrelated behavior?
- can this function be tested directly?
- does the module name describe one responsibility?
- would an LLM agent be able to edit this safely without pulling the whole crate into context?

Small, focused modules are an agent-operability feature.
