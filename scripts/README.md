# Scripts

Repository maintenance scripts. Prefer `cargo xtask <task>` wrappers when one exists.

## `check-dependency-updates.sh`

Read-only dependency drift report for the Rust workspace.

```bash
cargo xtask check-deps
scripts/check-dependency-updates.sh --skip-search
scripts/check-dependency-updates.sh --fail-on-updates
```

Reports lockfile-compatible updates from `cargo update --dry-run`, then checks direct root `[workspace.dependencies]` against crates.io with `cargo search` unless `--skip-search` is used.

## Cargo Deny

Dependency license, advisory, ban, and source checks are configured in root `deny.toml`.

```bash
cargo xtask deny
just deny
```

This is an explicit/manual dependency quality task. It is not a pre-commit hook.

## `check-file-size.sh`

Checks tracked Rust/TypeScript source files against AgentCast module-size guardrails.

```bash
cargo xtask file-size
AGENTCAST_MAX_RS_LINES=450 cargo xtask file-size
scripts/check-file-size.sh crates/agent-runtime/src/lib.rs
```

Defaults:

- Rust production files: `400` effective production lines.
- TypeScript/TSX files: `350` effective production lines.
- Test files are exempt.
- A trailing Rust `#[cfg(test)] mod tests;` or inline test module is excluded from production-line counting.

This is an explicit/manual quality task, not a pre-commit hook.

## Hook Scripts

`scripts/hooks/` contains the approved lightweight pre-commit hook implementations used by `lefthook.yml`:

- conflict-marker check.
- large-file guard.
- staged secret scan.
- structured TOML/YAML/JSON syntax check.
- line-ending and executable-bit sanity check.

Do not add or expand pre-commit hooks unless `docs/DECISIONS.md` records explicit approval for that exact hook.

## Docs Scripts

- `refresh-docs.sh`: refreshes local reference snapshots.
- `review-changes.sh`: reviews changed authored docs/source against repository rules.
