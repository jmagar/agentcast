use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RegistryServerView {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub latest_version: Option<String>,
    #[serde(default)]
    pub package_count: usize,
}
