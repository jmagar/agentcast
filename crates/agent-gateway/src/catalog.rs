use agent_protocol::{LauncherAction, LauncherActionId, LauncherActionKind};
use agent_runtime::RuntimeCatalogSnapshot;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GatewayCatalog {
    pub actions: Vec<LauncherAction>,
    pub collisions: Vec<CollisionReport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollisionReport {
    pub action_id: LauncherActionId,
    pub existing_display_name: String,
    pub rejected_display_name: String,
}

pub type GatewaySearchDocument = agent_search::SearchDocument;

impl GatewayCatalog {
    pub fn from_snapshots(snapshots: Vec<RuntimeCatalogSnapshot>) -> Self {
        let mut actions_by_id = BTreeMap::<LauncherActionId, LauncherAction>::new();
        let mut collisions = Vec::new();

        for snapshot in snapshots {
            for tool in snapshot.tools {
                let action_id = LauncherActionId::from_server_tool(&snapshot.server_id, &tool.id);
                let display_name = tool.title.clone().unwrap_or(tool.name.clone());
                let action = LauncherAction {
                    id: action_id.clone(),
                    display_name: display_name.clone(),
                    description: tool.description.clone(),
                    kind: LauncherActionKind::McpTool {
                        server_id: snapshot.server_id.clone(),
                        tool_id: tool.id.clone(),
                    },
                };

                if let Some(existing) = actions_by_id.get(&action_id) {
                    collisions.push(CollisionReport {
                        action_id,
                        existing_display_name: existing.display_name.clone(),
                        rejected_display_name: display_name,
                    });
                } else {
                    actions_by_id.insert(action_id, action);
                }
            }
        }

        Self {
            actions: actions_by_id.into_values().collect(),
            collisions,
        }
    }

    pub fn search_documents(&self) -> Vec<GatewaySearchDocument> {
        let catalog_hash = self.catalog_hash();
        self.actions
            .iter()
            .map(|action| GatewaySearchDocument {
                action_id: action.id.clone(),
                name: action.display_name.clone(),
                description: action.description.clone(),
                metadata: action_metadata(action),
                catalog_hash: catalog_hash.clone(),
                truncated: false,
            })
            .collect()
    }

    pub fn catalog_hash(&self) -> String {
        let mut hasher = Sha256::new();
        for action in &self.actions {
            hasher.update(action.id.as_str().as_bytes());
            hasher.update([0]);
            hasher.update(action.display_name.as_bytes());
            hasher.update([0]);
            if let Some(description) = &action.description {
                hasher.update(description.as_bytes());
            }
            hasher.update([0]);
        }
        hex::encode(hasher.finalize())
    }
}

fn action_metadata(action: &LauncherAction) -> Vec<String> {
    match &action.kind {
        LauncherActionKind::McpTool { server_id, tool_id } => {
            vec![server_id.to_string(), tool_id.to_string(), "mcp tool".to_string()]
        }
    }
}
