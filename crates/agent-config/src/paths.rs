use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentPaths {
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
    pub env_file: PathBuf,
}

impl AgentPaths {
    pub fn from_home(home: &Path) -> Self {
        let config_dir = home.join(".agentcast");
        Self {
            config_file: config_dir.join("config.toml"),
            env_file: config_dir.join(".env"),
            config_dir,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathResolution {
    pub raw: PathBuf,
    pub expanded: PathBuf,
}
