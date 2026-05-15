use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct HealthSummary {
    pub status: HealthStatus,
    pub component: String,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
