//! Fleet and remote runtime coordination contracts for AgentCast.
//!
//! Fleet remains post-v0. This crate intentionally exposes transport-neutral
//! node, heartbeat, capability, and remote execution DTOs without importing Lab
//! node policy, service inventory, or local MCP process lifecycle behavior.

mod capability;
mod execution;
mod heartbeat;
mod node;

pub use capability::{CapabilityKind, CapabilitySummary};
pub use execution::{ExecutionTarget, RemoteExecutionRequest};
pub use heartbeat::{FleetHeartbeat, FleetStatus};
pub use node::{FleetNode, NodeId};

/// Returns the crate's public boundary label for diagnostics.
#[must_use]
pub fn crate_boundary() -> &'static str {
    "agent-fleet"
}
