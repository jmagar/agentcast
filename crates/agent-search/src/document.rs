use agent_protocol::LauncherActionId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct SearchDocument {
    pub action_id: LauncherActionId,
    pub name: String,
    pub description: Option<String>,
    pub metadata: Vec<String>,
    pub catalog_hash: String,
    pub truncated: bool,
}

impl SearchDocument {
    pub fn searchable_text(&self) -> String {
        let mut parts = vec![self.name.clone()];
        if let Some(description) = &self.description {
            parts.push(description.clone());
        }
        parts.extend(self.metadata.iter().cloned());
        parts.join(" ")
    }
}
