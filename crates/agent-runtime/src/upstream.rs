use agent_protocol::{McpServerId, McpToolId};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct ToolCallRequest {
    pub server_id: McpServerId,
    pub tool_id: McpToolId,
    pub arguments: Value,
    pub auth: Option<RuntimeRequestAuth>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolCallResponse {
    pub output: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeRequestAuth {
    pub bearer_token: String,
}

impl RuntimeRequestAuth {
    pub fn bearer(token: impl Into<String>) -> Self {
        Self {
            bearer_token: token.into(),
        }
    }
}
