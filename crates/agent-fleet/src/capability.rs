use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityKind {
    McpRuntime,
    AcpRuntime,
    MarketplaceInstall,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySummary {
    pub kind: CapabilityKind,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl CapabilitySummary {
    #[must_use]
    pub fn new(kind: CapabilityKind, name: impl Into<String>) -> Self {
        Self {
            kind,
            name: name.into(),
            version: None,
        }
    }

    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}
