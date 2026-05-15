use crate::ids::{LauncherActionId, McpServerId, McpToolId};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct LauncherAction {
    pub id: LauncherActionId,
    pub display_name: String,
    pub description: Option<String>,
    pub kind: LauncherActionKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LauncherActionKind {
    McpTool {
        server_id: McpServerId,
        tool_id: McpToolId,
    },
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ToolInvocation {
    pub action_id: LauncherActionId,
    pub arguments: Value,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ToolInvocationResult {
    pub action_id: LauncherActionId,
    pub output: Value,
}
