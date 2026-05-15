use thiserror::Error;

pub type RegistryResult<T> = Result<T, RegistryError>;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("registry request failed: {0}")]
    Request(String),
    #[error("registry data invalid: {0}")]
    InvalidData(String),
    #[error("registry input invalid: {0}")]
    InvalidInput(String),
}

impl RegistryError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Request(_) => "registry_request",
            Self::InvalidData(_) => "registry_invalid_data",
            Self::InvalidInput(_) => "registry_invalid_input",
        }
    }
}
