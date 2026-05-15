use agent_protocol::{McpServerId, McpToolId, ServerStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RuntimeCatalogSnapshot {
    pub server_id: McpServerId,
    pub server_name: String,
    pub status: ServerStatus,
    pub tools: Vec<RuntimeTool>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RuntimeTool {
    pub id: McpToolId,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub input_schema: Value,
}
