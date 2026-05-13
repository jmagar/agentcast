---
title: "CLI Spec"
doc_type: "spec"
status: "active"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs:
  - "docs/references/fastmcp/docs/client.mdx"
  - "docs/references/fastmcp/docs/generate-cli.mdx"
  - "docs/references/fastmcp/docs/overview.mdx"
  - "docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcporter/docs/cli-reference.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# CLI Spec

## Owning Crates

- `agent-cli`: Clap command model and output rendering.
- `agent-server`: binary entrypoint and top-level dispatch.
- `agent-runtime`: command behavior.

## Module Shape

Suggested modules:

```txt
crates/agent-cli/src/commands.rs
crates/agent-cli/src/output.rs
crates/agent-cli/src/pagination.rs
crates/agent-cli/src/servers.rs
crates/agent-cli/src/tools.rs
crates/agent-cli/src/resources.rs
crates/agent-cli/src/prompts.rs
crates/agent-cli/src/actions.rs
crates/agent-cli/src/registry.rs
crates/agent-cli/src/install.rs
crates/agent-cli/src/error.rs
```

## Command Handling

Each command handler should:

1. parse CLI args.
2. load config through `agent-config`.
3. construct or connect runtime.
4. call one runtime/gateway/registry/marketplace operation.
5. render output.
6. return normalized exit status.

## JSON Output

`--json` should serialize the same domain structs that API routes return where practical.

Errors under `--json` use:

```json
{
  "error": {
    "kind": "launcher.action_not_found",
    "message": "Launcher action not found: mcp:missing:tool:nope",
    "details": {
      "action_id": "mcp:missing:tool:nope"
    }
  }
}
```

## Pagination And Filtering

List/search commands should parse common pagination and filtering flags before calling runtime/search APIs:

```txt
--limit <n>
--cursor <cursor>
--filter <expr>
--server <server-id>
```

Filtering applies before pagination. Human output should show a continuation hint when more results exist. JSON output should include stable metadata:

```json
{
  "data": [],
  "meta": {
    "next_cursor": null
  }
}
```

## MCP Discovery Commands

MVP CLI implementation should expose the short `ac` command set:

```txt
ac mcp list
ac tools list
ac resources list
ac prompts list
ac mcp resources <server-id> [uri]
ac call <server-id> <tool-name>
```

These commands call the same runtime operations as the long `agentcast` commands.

## Long Command Parity

The implementation must also expose every long `agentcast ...` command listed in `docs/contracts/cli.md`. The short `ac` commands are convenience aliases for MCP discovery and direct calls; they do not reduce the required long command surface.

## Verification

Run:

```bash
cargo test -p agent-cli
cargo test -p agent-server
```

## Upstream References

- `docs/references/fastmcp/docs/overview.mdx`
- `docs/references/fastmcp/docs/client.mdx`
- `docs/references/fastmcp/docs/generate-cli.mdx`
- `docs/references/mcporter/docs/cli-reference.md`
- `docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md`
- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`
