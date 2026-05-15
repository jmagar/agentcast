use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum StashPathError {
    #[error("stash path must not be empty")]
    Empty,
    #[error("stash path must be relative: {0}")]
    Absolute(String),
    #[error("stash path must not escape the stash root: {0}")]
    Traversal(String),
    #[error("stash path must use forward slashes: {0}")]
    Backslash(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SafeRelativePath(String);

impl SafeRelativePath {
    pub fn new(path: impl Into<String>) -> Result<Self, StashPathError> {
        let path = path.into();
        validate_relative_stash_path(&path)?;
        Ok(Self(path))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SafeRelativePath {
    type Error = StashPathError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<SafeRelativePath> for String {
    fn from(value: SafeRelativePath) -> Self {
        value.0
    }
}

pub fn validate_relative_stash_path(path: &str) -> Result<(), StashPathError> {
    if path.trim().is_empty() {
        return Err(StashPathError::Empty);
    }
    if path.contains('\\') {
        return Err(StashPathError::Backslash(path.to_owned()));
    }

    let path_ref = Path::new(path);
    if path_ref.is_absolute() {
        return Err(StashPathError::Absolute(path.to_owned()));
    }

    for component in path_ref.components() {
        match component {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(StashPathError::Traversal(path.to_owned()));
            }
            Component::CurDir | Component::Normal(_) => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
