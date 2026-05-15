use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{ConfigResult, McpConfig};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default)]
    pub mcp: McpConfig,
}

impl AgentConfig {
    pub fn validate(&mut self) -> ConfigResult<()> {
        self.mcp.validate()
    }
}

pub fn load_from_str(raw: &str) -> ConfigResult<AgentConfig> {
    let mut config: AgentConfig = toml::from_str(raw)?;
    config.validate()?;
    Ok(config)
}

pub fn load_from_path(path: &Path) -> ConfigResult<AgentConfig> {
    load_from_str(&fs::read_to_string(path)?)
}

pub fn write_to_path(path: &Path, config: &AgentConfig) -> ConfigResult<()> {
    fs::write(path, toml::to_string_pretty(config)?)?;
    Ok(())
}
