use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AcpSessionId(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcpPromptRequest {
    pub prompt: String,
    #[serde(default)]
    pub attachments: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AcpSessionCommand {
    Prompt {
        request: AcpPromptRequest,
    },
    Cancel,
    ApprovePermission {
        request_id: String,
        option_id: String,
    },
    RejectPermission {
        request_id: String,
    },
}
