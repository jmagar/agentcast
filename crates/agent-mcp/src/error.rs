use thiserror::Error;

pub type McpResult<T> = Result<T, McpError>;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("mcp connection failed: {0}")]
    Connection(String),
    #[error("mcp protocol error: {0}")]
    Protocol(String),
    #[error("mcp tool error: {0}")]
    Tool(String),
    #[error("mcp arguments must be a JSON object")]
    InvalidArguments,
}

impl McpError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Connection(_) => "mcp_connection",
            Self::Protocol(_) => "mcp_protocol",
            Self::Tool(_) => "mcp_tool",
            Self::InvalidArguments => "invalid_arguments",
        }
    }
}
