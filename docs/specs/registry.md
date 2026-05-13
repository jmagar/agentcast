---
title: "Registry Spec"
doc_type: "spec"
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
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Registry Spec

This spec describes registry implementation for v0 MCP server discovery.

The v0 implementation should behave as a local MCP Registry aggregator/subregistry: fetch official-registry-compatible records, persist them locally, normalize/index them, and serve AgentCast surfaces from that local store.

The upstream MCP Registry is currently marked preview. Treat exact upstream API shape as versioned input data and keep AgentCast projections explicit.

## Owning Crates

- `agent-registry`: registry clients, DTO normalization, cache lookup contracts.
- `agent-marketplace`: convert normalized candidates into install plans.
- `agent-store`: optional persistent cache storage.
- `agent-cli`: expose search and candidate inspection commands.

## Module Shape

Initial `agent-registry` modules:

- `lib.rs`: public exports.
- `error.rs`: registry error type and kind mapping.
- `candidate.rs`: normalized candidate DTOs.
- `mcp.rs`: official MCP Registry DTO parsing and normalization.
- `cache.rs`: cache record model and freshness policy.
- `sync.rs`: paginated/incremental upstream refresh.
- `subregistry.rs`: MCP Registry OpenAPI-compatible projection when needed.
- `search.rs`: search orchestration over configured registry sources.

## Candidate Model

Normalized candidates should include:

```rust
pub struct RegistryCandidate {
    pub candidate_id: CandidateId,
    pub source: RegistrySource,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub transports: Vec<TransportKind>,
    pub packages: Vec<PackageCandidate>,
    pub required_env: Vec<EnvRequirement>,
    pub provenance: ProvenanceRef,
}
```

`EnvRequirement` contains key names, descriptions, and required/optional status only. It never contains values.

## Search Flow

1. load registry source config from `config.toml`.
2. read normalized local cache/index.
3. mark freshness and provenance.
4. sort deterministically by exact match, source rank, package name, and version.
5. return candidates without producing install plans.

Search should not require a live network call when local cache data exists.

## Sync Flow

1. fetch upstream pages using cursor-based pagination.
2. use incremental `updated_since`-style filters when supported.
3. persist raw upstream records and normalized records.
4. update server status, including deprecated/deleted states.
5. preserve fetch timestamp and source checkpoint.
6. rebuild or incrementally update the local search index.

Default refresh cadence should be regular but infrequent, in the hourly class unless configured otherwise.

This cadence mirrors upstream aggregator guidance and is not a claim that cached data is latest.

## Cache Flow

Cache records should include:

- normalized candidate payload.
- raw source checksum when available.
- fetched-at timestamp.
- source URL or local path.
- freshness state.
- upstream status.
- source cursor/update checkpoint when available.
- namespaced AgentCast curation metadata.

`agent-store` may provide persistence, but `agent-registry` owns freshness semantics.

`subregistry.rs` is MCP Registry subregistry support only. ACP Agent Registry records and ACP session metadata are out of scope for this v0 registry spec.

## Verification

Add source-side tests for:

- official MCP Registry fixture normalization.
- candidate ID determinism.
- stale cache visibility.
- paginated upstream sync.
- upstream status updates.
- subregistry `_meta` projection.
- absence of mutation side effects.
- required env key propagation into install candidates.

Run:

```bash
cargo test -p agent-registry
```

## Upstream References

- `docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md`
- `docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md`
- `docs/references/mcp/docs/markdown/0046-modelcontextprotocol-io-registry-versioning.md`
- `docs/references/mcp/docs/markdown/0085-modelcontextprotocol-io-registry-registry-aggregators.md`
- `docs/references/mcp/docs/markdown/0132-modelcontextprotocol-io-registry-authentication.md`
- `docs/references/mcp/repos/modelcontextprotocol-registry.xml`
- `docs/references/acp/docs/markdown/0043-agentclientprotocol-com-get-started-registry.md`
- `docs/references/acp/docs/markdown/0023-agentclientprotocol-com-rfds-acp-agent-registry.md`
