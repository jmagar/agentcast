use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallConflictKind {
    ExistingMcpUpstream,
    ExistingEnvVar,
    UnsupportedRuntime,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct InstallConflict {
    pub kind: InstallConflictKind,
    pub target: String,
    pub message: String,
}
