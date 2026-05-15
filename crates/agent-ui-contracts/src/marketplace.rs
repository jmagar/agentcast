use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct InstallPlanView {
    pub package: String,
    pub step_count: usize,
    #[serde(default)]
    pub warnings: Vec<String>,
}
