# Apps Agent Guide

`apps/` contains deployable AgentCast surfaces. Reusable behavior belongs in `crates/`.

Read `../docs/specs/app-surfaces.md` before adding or expanding an app directory.

## App Boundary

Apps may own:

- binary wrappers and launch composition.
- packaging metadata.
- app-specific configuration examples.
- deployment assets.
- shell completion or installer assets.
- smoke tests for the packaged surface.
- frontend or desktop shell code when that app exists.

Apps must not become the only owner of:

- runtime lifecycle.
- gateway routing policy.
- registry fetch/cache behavior.
- marketplace install planning.
- protocol adapter behavior.
- shared API/CLI/UI DTO contracts.
- config loading and precedence rules.

Move reusable logic into the appropriate crate.

## Current Surfaces

- `apps/cli`: deployable CLI packaging and wrapper concerns. Reusable command parsing stays in `crates/agent-cli`.
- `apps/mcp`: deployable AgentCast MCP server entrypoint. Reusable MCP logic stays in `crates/agent-mcp`; gateway policy stays in `crates/agent-gateway`.
- `apps/api`: deployable HTTP API app concerns. Reusable Axum routes stay in `crates/agent-api`.
- `apps/web`: future frontend application. It consumes API/runtime contracts and shared DTOs from `crates/agent-ui-contracts`.
- `apps/desktop`: future desktop shell. It consumes shared contracts; reusable native logic should move into a crate once it becomes substantial.

## Thin Entrypoint Rule

App entrypoints should compose crates; they should not hide product decisions.

Good app code:

- loads app-specific packaging/config glue.
- initializes tracing and shutdown.
- calls crate-owned builders or runners.
- contains deployment-specific metadata.

Bad app code:

- implements MCP tool discovery directly.
- performs registry normalization.
- decides gateway collision behavior.
- owns install-plan mutation.
- duplicates CLI/API rendering contracts.

## Frontend And Desktop

For `apps/web` and `apps/desktop`, use shared DTOs from `crates/agent-ui-contracts` where possible.

Frontend/desktop code must not implement ACP or MCP protocol behavior directly. It should call AgentCast API/runtime surfaces.

Keep reusable state, schema, and API contract types out of app-only folders when more than one surface needs them.

## Verification

For app-only changes, run the narrowest useful app-specific check once that app has tooling.

Before declaring substantial app work complete, run:

```bash
cargo xtask verify
```

If the change affects docs or contracts, also run:

```bash
cargo xtask audit-docs
```

