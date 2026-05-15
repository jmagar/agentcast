use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ScopeSet {
    scopes: Vec<String>,
}

impl ScopeSet {
    pub fn parse(raw: &str) -> Result<Self, ScopeError> {
        if raw.is_empty() {
            return Ok(Self { scopes: Vec::new() });
        }

        if raw.split(' ').any(str::is_empty) {
            return Err(ScopeError::EmptySegment);
        }

        let scopes = raw
            .split(' ')
            .map(validate_scope)
            .collect::<Result<BTreeSet<_>, _>>()?
            .into_iter()
            .collect();

        Ok(Self { scopes })
    }

    pub fn from_scopes(
        scopes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, ScopeError> {
        let scopes = scopes
            .into_iter()
            .map(|scope| validate_scope(&scope.into()))
            .collect::<Result<BTreeSet<_>, _>>()?
            .into_iter()
            .collect();

        Ok(Self { scopes })
    }

    pub fn as_slice(&self) -> &[String] {
        &self.scopes
    }

    pub fn contains_all(&self, required: &ScopeSet) -> bool {
        required
            .scopes
            .iter()
            .all(|scope| self.scopes.binary_search(scope).is_ok())
    }

    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }

    pub fn as_header_value(&self) -> String {
        self.scopes.join(" ")
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ScopeError {
    #[error("scope string contains an empty segment")]
    EmptySegment,
    #[error("scope `{0}` contains unsupported characters")]
    InvalidCharacters(String),
}

fn validate_scope(scope: &str) -> Result<String, ScopeError> {
    let valid = scope
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ':' | '.' | '_' | '-'));

    if valid {
        Ok(scope.to_string())
    } else {
        Err(ScopeError::InvalidCharacters(scope.to_string()))
    }
}
