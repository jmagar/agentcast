---
title: "Registry Contract"
doc_type: "contract"
status: "active"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs:
  - "docs/references/acp/docs/markdown/0023-agentclientprotocol-com-rfds-acp-agent-registry.md"
  - "docs/references/acp/docs/markdown/0043-agentclientprotocol-com-get-started-registry.md"
  - "docs/references/mcp/docs/markdown/0046-modelcontextprotocol-io-registry-versioning.md"
  - "docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md"
  - "docs/references/mcp/docs/markdown/0085-modelcontextprotocol-io-registry-registry-aggregators.md"
  - "docs/references/mcp/docs/markdown/0132-modelcontextprotocol-io-registry-authentication.md"
  - "docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md"
  - "docs/references/mcp/repos/modelcontextprotocol-registry.xml"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Registry Contract

This contract defines registry behavior for the MCP launcher MVP.

## Scope

v0 registry support implements a local MCP Registry aggregator/subregistry for MCP server discovery and install candidate normalization.

Registry lookup must not start servers, mutate config, or write `.env`.

The upstream MCP Registry corpus currently marks the registry as preview. AgentCast must surface provenance and freshness instead of implying cached data is current.

## Search Results

Registry search returns normalized candidates with:

- stable `candidate_id`.
- display name.
- source registry.
- upstream package reference.
- version or revision when available.
- supported transports.
- required config fields.
- required `.env` key names without secret values.
- provenance URL or registry record reference.
- freshness and status metadata.

Candidate IDs must be deterministic for the same registry record and version. If a registry does not provide a stable ID, AgentCast must derive one from source registry, package identity, version, and package index.

## Normalization

Requirements:

- registry-specific DTOs do not leak past `agent-registry`.
- missing optional metadata is represented explicitly as absent, not guessed.
- install commands are represented as candidate metadata, not shell strings to execute immediately.
- required secrets and endpoint URLs are represented as `.env` key requirements.
- non-secret package metadata is represented as future `config.toml` mutations.
- upstream metadata and AgentCast curation metadata remain separate.
- AgentCast curation uses namespaced `_meta` fields when exposed through an MCP-registry-compatible subregistry shape.

AgentCast registry behavior in this contract is MCP Registry behavior, not ACP Agent Registry behavior. ACP agent discovery/session metadata must be specified separately before AgentCast claims ACP registry compatibility.

## Cache

Requirements:

- cache entries include source, fetch time, and staleness status.
- cache entries include upstream status.
- offline cache use must be visible in output.
- cached records must not be described as latest unless freshness is verified.
- normalized registry data is locally indexable for fast search.
- search reads normalized metadata and reports provenance/freshness.
- refresh behavior is regular but infrequent by default.
- the official MCP Registry is not required to be reachable for cached local search.

## Acceptance Tests

Implementations must test:

- deterministic candidate IDs.
- registry DTO normalization.
- cache stale/fresh marking.
- indexed search over normalized metadata.
- pagination/incremental refresh behavior from registry source fixtures.
- preservation of upstream metadata separate from AgentCast `_meta` curation.
- no side effects during search.
- required secret fields appear as `.env` key names only.

## Upstream References

- `docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md`
- `docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md`
- `docs/references/mcp/docs/markdown/0046-modelcontextprotocol-io-registry-versioning.md`
- `docs/references/mcp/docs/markdown/0085-modelcontextprotocol-io-registry-registry-aggregators.md`
- `docs/references/mcp/docs/markdown/0132-modelcontextprotocol-io-registry-authentication.md`
- `docs/references/mcp/repos/modelcontextprotocol-registry.xml`
- `docs/references/acp/docs/markdown/0043-agentclientprotocol-com-get-started-registry.md`
- `docs/references/acp/docs/markdown/0023-agentclientprotocol-com-rfds-acp-agent-registry.md`
