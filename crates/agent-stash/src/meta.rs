use serde::{Deserialize, Serialize};

use crate::SafeRelativePath;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StashItemKind {
    PromptTemplate,
    ActionTemplate,
    ReferenceBundle,
    HistoryBundle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftStatus {
    Clean,
    Dirty,
    Deleted,
    BaseMissing,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StashItemMeta {
    pub id: String,
    pub kind: StashItemKind,
    pub path: SafeRelativePath,
    pub title: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub drift_status: DriftStatus,
}

impl StashItemMeta {
    pub fn new(
        id: impl Into<String>,
        kind: StashItemKind,
        path: SafeRelativePath,
        title: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            path,
            title: title.into(),
            tags: Vec::new(),
            drift_status: DriftStatus::Unknown,
        }
    }

    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    #[must_use]
    pub fn with_drift_status(mut self, drift_status: DriftStatus) -> Self {
        self.drift_status = drift_status;
        self
    }
}

#[cfg(test)]
mod tests;
