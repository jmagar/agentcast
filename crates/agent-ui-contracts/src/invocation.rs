use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvocationResultView {
    pub action_id: String,
    pub output: Value,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct InvocationErrorView {
    pub action_id: String,
    pub kind: String,
    pub message: String,
}
