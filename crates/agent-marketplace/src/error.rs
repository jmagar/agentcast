use thiserror::Error;

pub type MarketplaceResult<T> = Result<T, MarketplaceError>;

#[derive(Debug, Error)]
pub enum MarketplaceError {
    #[error("invalid install target: {0}")]
    InvalidTarget(String),
    #[error("unsafe install parameter: {0}")]
    UnsafeParameter(String),
    #[error("install conflict: {0}")]
    Conflict(String),
}

impl MarketplaceError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::InvalidTarget(_) => "invalid_install_target",
            Self::UnsafeParameter(_) => "unsafe_install_parameter",
            Self::Conflict(_) => "install_conflict",
        }
    }
}
