use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GatewayActionView {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub server_id: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServerStatusView {
    pub id: String,
    pub name: String,
    pub status: String,
    #[serde(default)]
    pub detail: Option<String>,
}
