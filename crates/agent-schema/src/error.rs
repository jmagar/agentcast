use thiserror::Error;

pub type SchemaResult<T> = Result<T, SchemaError>;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SchemaError {
    #[error("schema is invalid: {0}")]
    InvalidSchema(String),
    #[error("payload is invalid: {0}")]
    InvalidPayload(String),
}

impl SchemaError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::InvalidSchema(_) => "invalid_schema",
            Self::InvalidPayload(_) => "invalid_payload",
        }
    }
}
