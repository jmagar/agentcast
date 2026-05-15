use thiserror::Error;

pub type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid MCP JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("MCP server `{name}` must define either command or url")]
    MissingTarget { name: String },
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    #[error("failed to parse config: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("failed to serialize config: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("failed to read or write config: {0}")]
    Io(#[from] std::io::Error),
    #[error("env merge conflict for `{0}`")]
    EnvConflict(String),
}

impl ConfigError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Json(_) => "json_error",
            Self::MissingTarget { .. } => "missing_target",
            Self::InvalidConfig(_) => "invalid_config",
            Self::Toml(_) => "parse_error",
            Self::TomlSerialize(_) => "serialize_error",
            Self::Io(_) => "io_error",
            Self::EnvConflict(_) => "env_conflict",
        }
    }
}
