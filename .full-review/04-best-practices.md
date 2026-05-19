# Phase 4: Best Practices and Standards

## Findings

- High - `xtask/src/task.rs:72`
  `cargo xtask ci` includes `nextest-ci` and `deny`, and docs list `cargo-nextest`, `cargo-deny`, and `taplo` as expected tools, but the visible PATH for this review lacks `cargo-nextest`, `cargo-deny`, and `taplo`. The direct cargo path that avoids the Snap/sccache failure cannot run nextest.
  Impact: the documented pre-push and completion gates are not reproducible in the current environment.
  Fix by making `cargo xtask doctor` fail loudly for missing toolchain subcommands before verification, and document the direct toolchain workaround or repair the local cargo wrapper.

- High - `crates/agent-api/src/protected_mcp.rs:21`
  Test fixtures are available through production constructors and public exports (`FixtureBearerTokenVerifier` is re-exported from `agent-auth`). Best practice for auth code is to make insecure verifiers impossible to select accidentally outside tests.
  Impact: production safety depends on caller discipline.
  Fix by gating fixture verifiers behind `#[cfg(any(test, feature = "fixtures"))]` and requiring production constructors to accept explicit verifier implementations.

- Medium - `docs/specs/crates-and-dependencies.md:33`
  The dependency contract describes a future `cargo xtask audit-deps`, but the implemented task list only includes `check-deps` and `deny`; there is no `audit-deps` task. Meanwhile Phase 1 found a real boundary drift (`agent-api` depending on `agent-mcp`).
  Impact: documented dependency-layer rules rely on manual review and can drift.
  Fix by implementing the promised manifest/layer audit or adding an explicit `cargo deny`/custom check for forbidden crate edges.

- Medium - `crates/agent-api/src/http.rs:157`
  Collection routes such as list actions, list servers, list resources, and registry search return unpaginated arrays or caller-controlled limits with no hard maximum at the HTTP boundary. This conflicts with the API doc's "no endpoint returns unbounded collections by default" requirement.
  Impact: large catalogs can produce oversized responses and poor client behavior.
  Fix by adding default and maximum pagination limits consistently at API boundaries.

- Medium - `crates/agent-marketplace/src/mcp.rs:263`
  `resolve_install_env` gathers allowed environment variables from all packages on a registry server, not only the package selected by `plan_mcp_server_install`. A caller can supply env values for an unselected package variant and receive them back in `env_values`.
  Impact: install apply responses can include irrelevant secret/runtime values and blur which package is being installed.
  Fix by resolving env values against the selected package from the install plan rather than every package on the server.

- Medium - `crates/agent-api/src/http/tests.rs:215`
  The marketplace apply HTTP test asserts that `env_values` in the API response contains the supplied secret value. This conflicts with `docs/SECURITY.md`, which says CLI and API output must redact common secret fields.
  Impact: the API can echo secrets back to clients and tests currently lock that behavior in.
  Fix by returning only env key names, redacted values, or a write summary from apply endpoints.

- Low - `crates/agent-observability/src/redaction.rs:4`
  Redaction constants are duplicated across crates and have inconsistent casing. This violates the repo's own operational consistency expectations more than Rust best practice, but it will complicate log/search/API tests.
  Impact: redaction assertions and docs can diverge.
  Fix by defining one shared redaction literal and helper in an appropriate low-level crate or observability facade.

- Low - `docs/QUALITY_GATES.md:58`
  Quality gate docs mention `cargo xtask audit-deps` "when it exists", while the current task is named `check-deps` and invokes dependency-update reporting rather than boundary auditing.
  Impact: contributors may run the wrong check or assume a missing policy check exists.
  Fix by updating docs to distinguish dependency update checks, `cargo deny`, and the not-yet-implemented boundary audit.

## Standards and Operations Notes

- Rust module and test placement standards are well-defined, but the largest files are now past the repo's own hard review threshold.
- The hook policy is consistent with `docs/DECISIONS.md`: heavy checks stay in pre-push/explicit xtasks rather than pre-commit.
- `deny.toml`, `.gitleaks.toml`, and `lefthook.yml` are present, which gives the repo a good baseline for dependency/source/secret hygiene once local tools are installed.
