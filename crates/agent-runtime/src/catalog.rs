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
    #[serde(default)]
    pub resources: Vec<RuntimeResource>,
    #[serde(default)]
    pub resource_templates: Vec<RuntimeResourceTemplate>,
    #[serde(default)]
    pub prompts: Vec<RuntimePrompt>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RuntimeTool {
    pub id: McpToolId,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub input_schema: Value,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RuntimeResource {
    pub uri: String,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RuntimeResourceTemplate {
    pub uri_template: String,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RuntimePrompt {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub arguments: Value,
}
