---
title: "Error Spec"
doc_type: "spec"
status: "active"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs:
  - "docs/references/claude-code/markdown/0083-code-claude-com-docs-en-permissions.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
  - "docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md"
  - "docs/references/mcporter/docs/cli-reference.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Error Spec

## Owning Crates

- `agent-core`: base error kind/envelope types.
- protocol/runtime crates: local error enums.
- `agent-cli`: exit code mapping.
- `agent-api`: HTTP status mapping.

## Error Type Shape

Suggested core types:

```rust
pub struct AgentCastErrorEnvelope {
    pub error: AgentCastErrorBody,
}

pub struct AgentCastErrorBody {
    pub kind: String,
    pub message: String,
    pub details: serde_json::Value,
}
```

Crates may use typed enums internally, but public surfaces map to this envelope.

## HTTP Status Mapping

Suggested mapping:

- config/validation/security confirmation errors -> `400`.
- auth errors -> `401` or `403`.
- not found errors -> `404`.
- runtime busy/conflict errors -> `409`.
- timeout errors -> `504`.
- internal bugs -> `500`.

## Redaction

Redaction should recursively replace values for keys matching:

- `token`
- `api_key`
- `apikey`
- `password`
- `secret`
- `authorization`
- `cookie`

Replacement value: `"[REDACTED]"`.

## Verification

Run:

```bash
cargo test -p agent-core error
cargo test -p agent-cli error
cargo test -p agent-api error
```

## Upstream References

- `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`
- `docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md`
- `docs/references/claude-code/markdown/0083-code-claude-com-docs-en-permissions.md`
- `docs/references/mcporter/docs/cli-reference.md`
