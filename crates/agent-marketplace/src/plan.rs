use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallPlan {
    pub package: String,
    #[serde(default)]
    pub steps: Vec<InstallStep>,
}

impl InstallPlan {
    pub fn new(package: impl Into<String>) -> Self {
        Self {
            package: package.into(),
            steps: Vec::new(),
        }
    }

    pub fn step(mut self, step: InstallStep) -> Self {
        self.steps.push(step);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallStep {
    pub kind: InstallStepKind,
    pub description: String,
    pub target: String,
    pub preview: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallStepKind {
    AddMcpUpstream,
    SetEnvVar,
    VerifyRuntime,
}
