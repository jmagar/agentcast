use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::NodeId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionTarget {
    pub node_id: NodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_name: Option<String>,
}

impl ExecutionTarget {
    #[must_use]
    pub fn node(node_id: impl Into<NodeId>) -> Self {
        Self {
            node_id: node_id.into(),
            capability_name: None,
        }
    }

    #[must_use]
    pub fn capability(mut self, name: impl Into<String>) -> Self {
        self.capability_name = Some(name.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoteExecutionRequest {
    pub request_id: String,
    pub target: ExecutionTarget,
    pub action_id: String,
    #[serde(default)]
    pub arguments: Value,
}

impl RemoteExecutionRequest {
    #[must_use]
    pub fn new(
        request_id: impl Into<String>,
        target: ExecutionTarget,
        action_id: impl Into<String>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            target,
            action_id: action_id.into(),
            arguments: Value::Object(Default::default()),
        }
    }

    #[must_use]
    pub fn with_arguments(mut self, arguments: Value) -> Self {
        self.arguments = arguments;
        self
    }
}
