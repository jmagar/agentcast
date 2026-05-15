use thiserror::Error;

pub type AcpResult<T> = Result<T, AcpError>;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum AcpError {
    #[error("ACP conversion failed: {0}")]
    Conversion(String),
    #[error("ACP session command invalid: {0}")]
    InvalidCommand(String),
}

impl AcpError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Conversion(_) => "acp_conversion",
            Self::InvalidCommand(_) => "acp_invalid_command",
        }
    }
}
