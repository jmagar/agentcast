use crate::ids::McpServerId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct McpServerConfig {
    pub id: McpServerId,
    pub name: String,
    pub enabled: bool,
    pub transport: McpTransportConfig,
    pub env_keys: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpTransportConfig {
    Stdio {
        command: String,
        args: Vec<String>,
        env: BTreeMap<String, String>,
    },
    StreamableHttp {
        url: String,
        bearer_token_env: Option<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerStatus {
    Disabled,
    Healthy,
    Degraded,
    Failed,
}
