use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcpEvent {
    pub kind: AcpEventKind,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpEventKind {
    MessageChunk,
    ReasoningChunk,
    ToolCall,
    ToolCallUpdate,
    PermissionRequest,
    UsageUpdate,
    SessionInfo,
    Unknown,
}
