use serde::{Deserialize, Serialize};

use crate::{CapabilitySummary, NodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FleetStatus {
    Online,
    Degraded,
    Offline,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetHeartbeat {
    pub node_id: NodeId,
    pub status: FleetStatus,
    pub checked_at: String,
    #[serde(default)]
    pub capabilities: Vec<CapabilitySummary>,
}

impl FleetHeartbeat {
    #[must_use]
    pub fn new(
        node_id: impl Into<NodeId>,
        status: FleetStatus,
        checked_at: impl Into<String>,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            status,
            checked_at: checked_at.into(),
            capabilities: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_capability(mut self, capability: CapabilitySummary) -> Self {
        self.capabilities.push(capability);
        self
    }

    #[must_use]
    pub fn is_available(&self) -> bool {
        matches!(self.status, FleetStatus::Online | FleetStatus::Degraded)
    }
}

#[cfg(test)]
mod tests;
