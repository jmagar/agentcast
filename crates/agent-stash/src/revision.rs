use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RevisionId(pub String);

impl RevisionId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RevisionId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RevisionId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StashRevision {
    pub id: RevisionId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<RevisionId>,
    pub item_id: String,
    pub message: String,
    pub created_at: String,
}

impl StashRevision {
    #[must_use]
    pub fn new(
        id: impl Into<RevisionId>,
        item_id: impl Into<String>,
        message: impl Into<String>,
        created_at: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            parent: None,
            item_id: item_id.into(),
            message: message.into(),
            created_at: created_at.into(),
        }
    }

    #[must_use]
    pub fn with_parent(mut self, parent: impl Into<RevisionId>) -> Self {
        self.parent = Some(parent.into());
        self
    }
}

#[cfg(test)]
mod tests;
