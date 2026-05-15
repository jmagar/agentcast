use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub String);

impl NodeId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for NodeId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for NodeId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetNode {
    pub node_id: NodeId,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub labels: BTreeMap<String, String>,
}

impl FleetNode {
    #[must_use]
    pub fn new(node_id: impl Into<NodeId>, display_name: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            display_name: display_name.into(),
            labels: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests;
