---
title: "Registry Contract"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/mcp/docs/markdown/0038-modelcontextprotocol-io-registry-remote-servers.md"
  - "docs/references/mcp/docs/markdown/0046-modelcontextprotocol-io-registry-versioning.md"
  - "docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md"
  - "docs/references/mcp/docs/markdown/0172-modelcontextprotocol-io-registry-faq.md"
  - "docs/references/mcp/repos/modelcontextprotocol-registry.xml"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Registry Contract

`agent-registry` owns discovery and normalization of registry data.

For v0, registry work implements a local MCP Registry aggregator/subregistry for MCP server install candidates.

AgentCast should not treat the official MCP Registry as a live dependency on every host operation. It should pull registry metadata on a regular but infrequent basis, persist it locally, normalize/index it, and serve AgentCast surfaces from the local store.

The official MCP Registry is preview. AgentCast should treat its API/OpenAPI shape as the target compatibility source, but implementation must tolerate schema/API drift and data resets.

## Registry Sources

Initial sources:

- official MCP Registry.
- curated AgentCast metadata.
- local registry cache.

Future sources:

- ACP Registry.
- GitHub repos containing Claude Code plugin marketplaces.
- Codex-compatible marketplaces.
- local/private registries.

## Aggregator Model

AgentCast acts as a downstream aggregator for the official MCP Registry.

Requirements:

- use the official MCP Registry REST API/OpenAPI shape as the compatibility target.
- fetch server pages with cursor-based pagination.
- support incremental refresh using update timestamps when available.
- persist fetched metadata in AgentCast's local store.
- keep server lifecycle metadata up to date when exposed by the registry, including active/deprecated/deleted status fields from the registry repo/API shape.
- preserve upstream metadata separately from AgentCast curation.
- inject AgentCast curation under namespaced `_meta` keys when exposing subregistry-compatible records.
- never require the official registry service to be online for local search over already cached data.

The official MCP Registry codebase itself is not treated as AgentCast's self-hosted registry server. AgentCast owns its local aggregator/cache/index behavior.

Do not imply that ordinary publishers can freely unpublish/delete servers. The public FAQ says unpublish/delete is not generally available at the time of the reference snapshot; lifecycle status fields exist in the registry repo/API shape and must be represented as registry metadata when present.

## Normalized Registry Entry

Every external registry entry should normalize into an AgentCast entry model with:

- id.
- source.
- name.
- description.
- version.
- homepage/repository.
- license when known.
- package/distribution info.
- installability status.
- tags/categories.
- required env/config metadata.
- trust/review metadata.
- raw source metadata.

## Curation Overlay

AgentCast-owned metadata must not mutate upstream registry data. Store curation separately.

Examples:

- featured.
- recommended.
- reviewed.
- hidden.
- tags.
- notes.
- risk flags.
- compatibility notes.

## Cache Rules

Registry cache entries must track:

- source.
- fetched timestamp.
- source version/hash if available.
- normalized schema version.
- fetch error state.
- upstream status.
- upstream cursor/update checkpoint when available.

Never silently treat stale cache as fresh.

## Indexing Rules

Registry data should be cached and indexed locally early.

Requirements:

- local cache/index must be self-hosted.
- search must run against normalized metadata, not raw registry DTOs.
- indexing must preserve provenance and freshness state.
- background refresh should update cache/index without blocking existing offline reads.
- refresh cadence should be regular but infrequent; hourly-class refresh is the intended shape unless configured otherwise.
- stale/offline results must be labeled as such in CLI/API/MCP surfaces.
- metadata normalization should support aliases, tags, categories, setup difficulty, risk labels, transport support, and auth/env requirements.

## MVP Registry Output

The first normalized registry entry must support MCP launcher install planning and local aggregator search:

- source id.
- name.
- description.
- repository/homepage when known.
- package or launch metadata when known.
- required env/config metadata when known.
- status.
- updated/fetched timestamp when known.
- cursor/update provenance when known.
- namespaced `_meta` curation overlay.
- raw source metadata for later re-normalization.

Trust scores, review status, Claude/Codex marketplace metadata, and ACP agent metadata are post-v0 overlays.

## Upstream References Checked

- `docs/references/mcp/docs/markdown/0172-modelcontextprotocol-io-registry-faq.md`: official registry terminology, preview status, custom `_meta`, and unpublish caveat.
- `docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md`: official `mcp-publisher` and publish workflow.
- `docs/references/mcp/docs/markdown/0038-modelcontextprotocol-io-registry-remote-servers.md`: `server.json` remote metadata.
- `docs/references/mcp/docs/markdown/0046-modelcontextprotocol-io-registry-versioning.md`: version immutability and package alignment.
- `docs/references/mcp/repos/modelcontextprotocol-registry.xml`: registry repo status fields, API examples, server.json schema, and metaregistry/subregistry model.
