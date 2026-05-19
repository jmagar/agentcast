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
    #[serde(default, skip_serializing_if = "InstallStepApply::is_none")]
    pub apply: InstallStepApply,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallStepKind {
    AddMcpUpstream,
    SetEnvVar,
    VerifyRuntime,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstallStepApply {
    #[default]
    None,
    RequireEnv {
        name: String,
    },
    AddMcpUpstream {
        id: String,
        transport: InstallMcpUpstreamTransport,
    },
}

impl InstallStepApply {
    fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub enum InstallMcpUpstreamTransport {
    Stdio {
        command: String,
        args: Vec<String>,
    },
    StreamableHttp {
        url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bearer_token_env: Option<String>,
    },
}
