use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreErrorKind {
    InvalidInput,
    NotFound,
    Conflict,
    Unavailable,
    Internal,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub kind: CoreErrorKind,
    pub message: String,
}

impl ErrorInfo {
    pub fn new(kind: CoreErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind_str(&self) -> &'static str {
        match self.kind {
            CoreErrorKind::InvalidInput => "invalid_input",
            CoreErrorKind::NotFound => "not_found",
            CoreErrorKind::Conflict => "conflict",
            CoreErrorKind::Unavailable => "unavailable",
            CoreErrorKind::Internal => "internal",
        }
    }
}
