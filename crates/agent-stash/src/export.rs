use serde::{Deserialize, Serialize};

use crate::StashItemMeta;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportManifest {
    pub version: u32,
    #[serde(default)]
    pub items: Vec<StashItemMeta>,
}

impl ExportManifest {
    #[must_use]
    pub fn v1(items: Vec<StashItemMeta>) -> Self {
        Self { version: 1, items }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportManifest {
    pub source: String,
    pub manifest: ExportManifest,
}

impl ImportManifest {
    #[must_use]
    pub fn new(source: impl Into<String>, manifest: ExportManifest) -> Self {
        Self {
            source: source.into(),
            manifest,
        }
    }
}
