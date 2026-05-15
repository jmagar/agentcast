use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid MCP JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("MCP server `{name}` must define either command or url")]
    MissingTarget { name: String },
}
