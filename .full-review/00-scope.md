# Review Scope

## Target

Full AgentCast workspace review from the current `main` checkout at `/home/jmagar/workspace/agentcast`.

The review treats the repository's v0 MCP launcher/runtime contract as the primary product boundary. Source inspection covered the Rust workspace, authored docs, Cargo manifests, and repo tooling. Existing untracked user files were left untouched.

## Files

- `Cargo.toml`
- `crates/**`
- `xtask/**`
- `docs/**`
- `scripts/**`
- `.cargo/config.toml`
- `.config/nextest.toml`
- `lefthook.yml`
- `.gitleaks.toml`
- `AGENTS.md`

## Review Flags

- Security focus: yes
- Performance critical: yes
- Strict mode: yes
- Framework: Rust workspace with Axum, Tokio, RMCP, SQLite, OAuth, and CLI surfaces

## Review Phases

1. Code Quality and Architecture
2. Security and Performance
3. Testing and Documentation
4. Best Practices and Standards
5. Consolidated Report

## Commands and Evidence

- `bd prime` passed and confirmed the repo's Beads workflow context.
- `git status --short --branch` showed `main...origin/main [ahead 1]` plus untracked `README.md` and session docs before review artifacts were written.
- `rg --files ...` enumerated workspace docs and Rust source.
- `find crates xtask -name '*.rs' ... | xargs wc -l` found 19,082 Rust lines total and several source modules over project thresholds.
- `cargo metadata --no-deps --format-version 1` failed before Cargo execution with a Snap AppArmor error from the local toolchain wrapper.
- `cargo tree -e normal --workspace` failed with the same Snap AppArmor error.
- `/home/jmagar/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo metadata --no-deps --format-version 1` passed.
- `RUSTC_WRAPPER= /home/jmagar/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo tree -e normal --workspace` passed after bypassing the `sccache` wrapper.
