use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub kind: ActivityKind,
    pub target: String,
    pub message: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityKind {
    ServerLifecycle,
    Invocation,
    InstallPlan,
    OAuth,
    RegistrySync,
}
