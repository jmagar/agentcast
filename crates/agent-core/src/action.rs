use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActionMetadata {
    pub action_id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub category: Option<ActionCategory>,
    #[serde(default)]
    pub risk: ActionRisk,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionCategory {
    Filesystem,
    Network,
    Registry,
    Marketplace,
    Runtime,
    Other(String),
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionRisk {
    #[default]
    ReadOnly,
    WritesConfig,
    RunsProcess,
    UsesNetwork,
}
