use agent_protocol::{McpServerId, McpToolId};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct ToolCallRequest {
    pub server_id: McpServerId,
    pub tool_id: McpToolId,
    pub arguments: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolCallResponse {
    pub output: Value,
}
