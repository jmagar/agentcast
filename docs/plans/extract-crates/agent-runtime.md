---
title: "agent-runtime Extraction Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md"
  - "docs/references/acp/docs/markdown/0062-agentclientprotocol-com-protocol-prompt-turn.md"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-runtime Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract local runtime primitives for process supervision, MCP stdio launch, session state, event sequencing, and durable local state.

**Architecture:** `agent-runtime` owns local process and state lifecycles. Protocol adapters live in `agent-mcp` and `agent-acp`; gateway projection lives in `agent-gateway`.

**Tech Stack:** Rust 2024, Tokio, process-wrap, nix, rusqlite, directories, tracing.

---

## MVP Position

Runtime is required for local stdio MCP server lifecycle in the MVP. ACP session registry and provider runtime are post-v0.

## Lab Source Files

- `../lab/crates/lab/src/dispatch/gateway/runtime.rs`
- `../lab/crates/lab/src/process.rs`
- `../lab/crates/lab/src/process/unix.rs`
- `../lab/crates/lab/src/acp/runtime.rs`
- `../lab/crates/lab/src/acp/registry.rs`
- `../lab/crates/lab/src/dispatch/acp/persistence.rs`
- `../lab/crates/lab/src/dispatch/deploy/runner.rs`
- `../lab/crates/lab/src/dispatch/deploy/lock.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab runtime source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`; the snapshot does not include a `crates/lab/src/dispatch/gateway/manager.rs` file entry, so runtime-specific GatewayManager evidence is taken from `crates/lab/src/dispatch/gateway/runtime.rs`.
- MCP local stdio lifecycle and Streamable HTTP boundaries are cross-checked against `docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md` and `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`.
- ACP post-v0 session, cancellation, and turn-drain claims are cross-checked against `docs/references/acp/docs/markdown/0062-agentclientprotocol-com-protocol-prompt-turn.md` and `docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md`.

## Live Lab Findings

- `dispatch/gateway/runtime.rs` has runtime owner views, persisted process entries, process matchers, cleanup matching, and process group termination logic.
- `process/unix.rs` is the cleaner extraction source for Unix signal, process group, cmdline, ancestor, and executable-path helpers.
- `acp/runtime.rs` and `acp/registry.rs` should be read for post-v0 session sequencing and turn drain, not MVP local MCP stdio launch.
- Deploy runner/lock code is not v0 runtime; only generic cancellation and locking ideas should be borrowed.

## Extraction Boundary

Extract for v0:

- local process launch and shutdown patterns.
- bounded channel patterns for runtime event flow.
- health and lifecycle state tracking.
- deterministic cleanup on cancellation and process exit.

Extract post-v0 for ACP:

- session registry.
- event sequence stamping.
- event replay/backfill.
- permission pending map and timeout.
- prompt idle and turn-drain behavior.
- SQLite persistence with HMAC/redaction rules.

Leave behind:

- Lab deploy pipeline.
- node/fleet runtime policy.
- homelab service process assumptions.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-runtime/src/lib.rs` - public exports for process, supervisor, registry, health, and error modules.
- Create: `crates/agent-runtime/src/error.rs` - runtime error type and stable kind strings.
- Create: `crates/agent-runtime/src/process.rs` - command specification, child handle, and redacted process metadata.
- Create: `crates/agent-runtime/src/process_unix.rs` - Unix process-group and signal helpers behind `cfg(unix)`.
- Create: `crates/agent-runtime/src/health.rs` - runtime status and health snapshots.
- Create: `crates/agent-runtime/src/registry.rs` - in-memory runtime registry keyed by upstream ID.
- Create: `crates/agent-runtime/src/supervisor.rs` - local MCP upstream lifecycle supervisor.
- Add source-side test sidecars for: `crates/agent-runtime/src/process.rs` - process config and redaction tests.
- Add source-side test sidecars for: `crates/agent-runtime/src/process.rs` - launch/stop tests with fixture commands.
- Add source-side test sidecars for: `crates/agent-runtime/src/registry.rs` - runtime health registry tests.
- Add source-side test sidecars for: `crates/agent-runtime/src/supervisor.rs` - upstream lifecycle supervisor tests.

## Implementation Tasks

### Task 1: Define Runtime Process Model

**Files:**
- Modify: `crates/agent-runtime/src/lib.rs`
- Create: `crates/agent-runtime/src/process.rs`
- Create: `crates/agent-runtime/src/error.rs`
- Test sidecar: `crates/agent-runtime/src/process.rs`

- [ ] **Step 1: Inspect Lab process ownership boundaries.**

Run:

```bash
rg -n "process group|kill|shutdown|cleanup|RuntimeEntry|GatewayRuntime|persisted|pid|Command" ../lab/crates/lab/src/dispatch/gateway/runtime.rs ../lab/crates/lab/src/process.rs ../lab/crates/lab/src/process/unix.rs
```

Expected: AgentCast extracts generic process launch/termination and health snapshots, not Lab service process matching.

- [ ] **Step 2: Write failing process spec tests.**

Create a source-side test sidecar next to `crates/agent-runtime/src/process.rs` with:

```rust
use super::*;

#[test]
fn process_spec_keeps_command_args_cwd_and_env() {
    let spec = ProcessSpec::new("npx")
        .arg("-y")
        .arg("@modelcontextprotocol/server-filesystem")
        .cwd("/tmp")
        .env("API_TOKEN", "secret");

    assert_eq!(spec.command, "npx");
    assert_eq!(spec.args, ["-y", "@modelcontextprotocol/server-filesystem"]);
    assert_eq!(spec.cwd.as_deref().map(|p| p.to_str().unwrap()), Some("/tmp"));
    assert_eq!(spec.env.get("API_TOKEN").map(String::as_str), Some("secret"));
}

#[test]
fn redacted_process_spec_hides_env_values() {
    let spec = ProcessSpec::new("node").env("API_TOKEN", "secret");
    let redacted = RedactedProcessSpec::from(&spec);
    assert_eq!(redacted.env.get("API_TOKEN").map(String::as_str), Some("[REDACTED]"));
}
```

Run:

```bash
cargo nextest run -p agent-runtime process_spec
```

Expected: FAIL because `ProcessSpec` and `RedactedProcessSpec` do not exist yet.

- [ ] **Step 3: Export runtime modules and error type.**

Update `crates/agent-runtime/src/lib.rs`:

```rust
mod error;
mod process;

pub use error::{RuntimeError, RuntimeResult};
pub use process::{ProcessSpec, RedactedProcessSpec};
```

Create `crates/agent-runtime/src/error.rs`:

```rust
use thiserror::Error;

pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("invalid process spec: {0}")]
    InvalidProcess(String),
    #[error("process launch failed: {0}")]
    Launch(String),
    #[error("process shutdown failed: {0}")]
    Shutdown(String),
}

impl RuntimeError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::InvalidProcess(_) => "invalid_process",
            Self::Launch(_) => "launch_failed",
            Self::Shutdown(_) => "shutdown_failed",
        }
    }
}
```

- [ ] **Step 4: Implement process spec and redaction.**

Create `crates/agent-runtime/src/process.rs`:

```rust
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessSpec {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: BTreeMap<String, String>,
}

impl ProcessSpec {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            cwd: None,
            env: BTreeMap::new(),
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactedProcessSpec {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: BTreeMap<String, String>,
}

impl From<&ProcessSpec> for RedactedProcessSpec {
    fn from(spec: &ProcessSpec) -> Self {
        Self {
            command: spec.command.clone(),
            args: spec.args.clone(),
            cwd: spec.cwd.clone(),
            env: spec.env.keys().map(|key| (key.clone(), "[REDACTED]".into())).collect(),
        }
    }
}
```

- [ ] **Step 5: Verify process spec tests.**

Run:

```bash
cargo nextest run -p agent-runtime process_spec
```

Expected: PASS.

### Task 2: Add Runtime Registry And Health Snapshots

**Files:**
- Modify: `crates/agent-runtime/src/lib.rs`
- Create: `crates/agent-runtime/src/registry.rs`
- Create: `crates/agent-runtime/src/health.rs`
- Test sidecar: `crates/agent-runtime/src/registry.rs`

- [ ] **Step 1: Write failing registry tests.**

Create a source-side test sidecar next to `crates/agent-runtime/src/registry.rs` with:

```rust
use super::*;

#[test]
fn registry_tracks_upstream_health_by_id() {
    let mut registry = RuntimeRegistry::default();
    registry.set_health(RuntimeHealth {
        upstream_id: "filesystem".into(),
        status: RuntimeStatus::Running,
        pid: Some(1234),
        message: None,
    });

    let health = registry.health("filesystem").expect("health exists");
    assert_eq!(health.status, RuntimeStatus::Running);
    assert_eq!(health.pid, Some(1234));
}

#[test]
fn registry_remove_drops_health_snapshot() {
    let mut registry = RuntimeRegistry::default();
    registry.set_health(RuntimeHealth {
        upstream_id: "filesystem".into(),
        status: RuntimeStatus::Starting,
        pid: None,
        message: Some("launching".into()),
    });

    assert!(registry.remove("filesystem").is_some());
    assert!(registry.health("filesystem").is_none());
}
```

- [ ] **Step 2: Export registry and health types.**

Update `crates/agent-runtime/src/lib.rs`:

```rust
mod health;
mod registry;

pub use health::{RuntimeHealth, RuntimeStatus};
pub use registry::RuntimeRegistry;
```

- [ ] **Step 3: Implement health snapshots.**

Create `crates/agent-runtime/src/health.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHealth {
    pub upstream_id: String,
    pub status: RuntimeStatus,
    pub pid: Option<u32>,
    pub message: Option<String>,
}
```

- [ ] **Step 4: Implement in-memory registry.**

Create `crates/agent-runtime/src/registry.rs`:

```rust
use std::collections::BTreeMap;

use crate::RuntimeHealth;

#[derive(Debug, Default)]
pub struct RuntimeRegistry {
    health: BTreeMap<String, RuntimeHealth>,
}

impl RuntimeRegistry {
    pub fn set_health(&mut self, health: RuntimeHealth) {
        self.health.insert(health.upstream_id.clone(), health);
    }

    pub fn health(&self, upstream_id: &str) -> Option<&RuntimeHealth> {
        self.health.get(upstream_id)
    }

    pub fn remove(&mut self, upstream_id: &str) -> Option<RuntimeHealth> {
        self.health.remove(upstream_id)
    }
}
```

- [ ] **Step 5: Verify registry tests.**

Run:

```bash
cargo nextest run -p agent-runtime registry
```

Expected: PASS.

### Task 3: Extract Process Helpers Before Runtime Policy

**Files:**
- Modify: `crates/agent-runtime/src/lib.rs`
- Modify: `crates/agent-runtime/src/process.rs`
- Create: `crates/agent-runtime/src/process_unix.rs`
- Test sidecar: `crates/agent-runtime/src/process.rs`

- [ ] **Step 1: Read the minimal process helper sources.**

Run:

```bash
sed -n '1,240p' ../lab/crates/lab/src/process.rs
sed -n '1,260p' ../lab/crates/lab/src/process/unix.rs
```

Expected: AgentCast extracts process-group launch and termination helpers without Lab-specific matchers.

- [ ] **Step 2: Write failing lifecycle test.**

Create a source-side test sidecar next to `crates/agent-runtime/src/process.rs` with:

```rust
use super::*;

#[tokio::test]
async fn process_handle_spawns_and_stops_short_lived_command() {
    let spec = ProcessSpec::new("sh").arg("-c").arg("sleep 30");
    let mut handle = ProcessHandle::spawn(spec).await.expect("process starts");

    assert!(handle.pid().is_some());
    handle.stop().await.expect("process stops");
    assert!(handle.is_stopped());
}
```

- [ ] **Step 3: Add process handle API.**

Update `crates/agent-runtime/src/lib.rs`:

```rust
pub use process::ProcessHandle;
```

Update `crates/agent-runtime/src/process.rs`:

```rust
pub struct ProcessHandle {
    child: tokio::process::Child,
}

impl ProcessHandle {
    pub async fn spawn(spec: ProcessSpec) -> crate::RuntimeResult<Self> {
        if spec.command.trim().is_empty() {
            return Err(crate::RuntimeError::InvalidProcess("command cannot be blank".into()));
        }

        let mut command = tokio::process::Command::new(&spec.command);
        command.args(&spec.args);
        command.env_clear();
        command.envs(&spec.env);
        if let Some(cwd) = &spec.cwd {
            command.current_dir(cwd);
        }

        #[cfg(unix)]
        crate::process_unix::configure_process_group(&mut command);

        let child = command
            .spawn()
            .map_err(|error| crate::RuntimeError::Launch(error.to_string()))?;
        Ok(Self { child })
    }

    pub fn pid(&self) -> Option<u32> {
        self.child.id()
    }

    pub async fn stop(&mut self) -> crate::RuntimeResult<()> {
        #[cfg(unix)]
        if let Some(pid) = self.child.id() {
            crate::process_unix::terminate_process_group(pid)?;
        }
        let _ = self.child.kill().await;
        Ok(())
    }

    pub fn is_stopped(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(Some(_)))
    }
}
```

- [ ] **Step 4: Add Unix helpers.**

Create `crates/agent-runtime/src/process_unix.rs`:

```rust
pub fn configure_process_group(command: &mut tokio::process::Command) {
    unsafe {
        command.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }
}

pub fn terminate_process_group(pid: u32) -> crate::RuntimeResult<()> {
    let pgid = -(pid as i32);
    let result = unsafe { libc::kill(pgid, libc::SIGTERM) };
    if result == -1 {
        return Err(crate::RuntimeError::Shutdown(
            std::io::Error::last_os_error().to_string(),
        ));
    }
    Ok(())
}
```

- [ ] **Step 5: Wire Unix module behind cfg.**

Update `crates/agent-runtime/src/lib.rs`:

```rust
#[cfg(unix)]
mod process_unix;
```

- [ ] **Step 6: Verify lifecycle tests.**

Run:

```bash
cargo nextest run -p agent-runtime process_lifecycle
```

Expected: PASS and no child process remains running.

### Task 4: Add MCP Upstream Supervisor

**Files:**
- Create: `crates/agent-runtime/src/supervisor.rs`
- Modify: `crates/agent-runtime/src/lib.rs`
- Test sidecar: `crates/agent-runtime/src/supervisor.rs`

- [ ] **Step 1: Write failing supervisor test.**

Create a source-side test sidecar next to `crates/agent-runtime/src/supervisor.rs` with:

```rust
use super::*;

#[tokio::test]
async fn supervisor_starts_and_stops_upstream() {
    let mut supervisor = UpstreamSupervisor::default();
    supervisor
        .start("fixture", ProcessSpec::new("sh").arg("-c").arg("sleep 30"))
        .await
        .expect("starts fixture");

    assert_eq!(
        supervisor.health("fixture").unwrap().status,
        RuntimeStatus::Running
    );

    supervisor.stop("fixture").await.expect("stops fixture");
    assert_eq!(
        supervisor.health("fixture").unwrap().status,
        RuntimeStatus::Stopped
    );
}
```

- [ ] **Step 2: Export supervisor.**

Update `crates/agent-runtime/src/lib.rs`:

```rust
mod supervisor;
pub use supervisor::UpstreamSupervisor;
```

- [ ] **Step 3: Implement supervisor with registry updates.**

Create `crates/agent-runtime/src/supervisor.rs`:

```rust
use std::collections::BTreeMap;

use crate::{ProcessHandle, ProcessSpec, RuntimeHealth, RuntimeRegistry, RuntimeResult, RuntimeStatus};

#[derive(Default)]
pub struct UpstreamSupervisor {
    registry: RuntimeRegistry,
    handles: BTreeMap<String, ProcessHandle>,
}

impl UpstreamSupervisor {
    pub async fn start(&mut self, upstream_id: &str, spec: ProcessSpec) -> RuntimeResult<()> {
        self.registry.set_health(RuntimeHealth {
            upstream_id: upstream_id.into(),
            status: RuntimeStatus::Starting,
            pid: None,
            message: None,
        });

        let handle = ProcessHandle::spawn(spec).await?;
        let pid = handle.pid();
        self.handles.insert(upstream_id.into(), handle);
        self.registry.set_health(RuntimeHealth {
            upstream_id: upstream_id.into(),
            status: RuntimeStatus::Running,
            pid,
            message: None,
        });
        Ok(())
    }

    pub async fn stop(&mut self, upstream_id: &str) -> RuntimeResult<()> {
        if let Some(mut handle) = self.handles.remove(upstream_id) {
            handle.stop().await?;
        }
        self.registry.set_health(RuntimeHealth {
            upstream_id: upstream_id.into(),
            status: RuntimeStatus::Stopped,
            pid: None,
            message: None,
        });
        Ok(())
    }

    pub fn health(&self, upstream_id: &str) -> Option<&RuntimeHealth> {
        self.registry.health(upstream_id)
    }
}
```

- [ ] **Step 4: Verify supervisor tests.**

Run:

```bash
cargo nextest run -p agent-runtime supervisor
```

Expected: PASS.

### Task 5: Prepare ACP Runtime Hand-Off

**Files:**
- Modify: `docs/plans/extract-crates/agent-runtime.md`
- Source: `../lab/crates/lab/src/acp/runtime.rs`
- Source: `../lab/crates/lab/src/acp/registry.rs`

- [ ] **Step 1: Read ACP runtime sections before post-v0 ACP work.**

Run:

```bash
rg -n "DEFAULT_PROMPT_IDLE_TIMEOUT|DEFAULT_TURN_DRAIN_TIMEOUT|PendingPermissions|spawn_event_forwarder|restore_from_persistence" ../lab/crates/lab/src/acp/runtime.rs ../lab/crates/lab/src/acp/registry.rs
```

Expected: ACP runtime ownership stays in `agent-runtime`, not `agent-acp`.

- [ ] **Step 2: Record post-v0 runtime boundaries in code comments only when implementing ACP runtime.**

Expected: the MCP launcher implementation does not add ACP session registry types until `docs/MVP.md` promotes ACP.

### Task 6: Verify Full Runtime Extraction

**Files:**
- Test sidecar: `crates/agent-runtime/src/*.rs`
- Read: `docs/plans/extract-crates/agent-runtime.md`

- [ ] **Step 1: Run focused runtime tests.**

Run:

```bash
cargo nextest run -p agent-runtime
```

Expected: PASS.

- [ ] **Step 2: Scan for Lab-specific runtime leakage.**

Run:

```bash
rg -n "LAB_|\\.lab|Plex|Sonarr|Radarr|Unraid|Gotify|AcpSessionRegistry|NodeRuntime" crates/agent-runtime
```

Expected: no output.

- [ ] **Step 3: Commit the runtime extraction slice.**

Run:

```bash
git add crates/agent-runtime docs/plans/extract-crates/agent-runtime.md
git commit -m "feat(runtime): extract local process supervisor"
```

Expected: commit contains only `agent-runtime` implementation, tests, and this plan if executing this slice alone.
