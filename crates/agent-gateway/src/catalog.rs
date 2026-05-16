use agent_protocol::{LauncherAction, LauncherActionId, LauncherActionKind};
use agent_runtime::RuntimeCatalogSnapshot;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

use crate::GatewayExposurePolicy;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GatewayCatalog {
    pub actions: Vec<LauncherAction>,
    pub collisions: Vec<CollisionReport>,
    search_documents: Vec<GatewaySearchDocument>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollisionReport {
    pub action_id: LauncherActionId,
    pub existing_display_name: String,
    pub rejected_display_name: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GatewayCatalogDiff {
    pub added: Vec<LauncherActionId>,
    pub removed: Vec<LauncherActionId>,
    pub changed: Vec<LauncherActionId>,
}

pub type GatewaySearchDocument = agent_search::SearchDocument;

impl GatewayCatalog {
    pub fn from_snapshots(snapshots: Vec<RuntimeCatalogSnapshot>) -> Self {
        Self::from_snapshots_with_policy(snapshots, &GatewayExposurePolicy::default())
    }

    pub fn from_snapshots_with_policy(
        snapshots: Vec<RuntimeCatalogSnapshot>,
        exposure_policy: &GatewayExposurePolicy,
    ) -> Self {
        let mut actions_by_id = BTreeMap::<LauncherActionId, LauncherAction>::new();
        let mut collisions = Vec::new();
        let mut documents_by_id = BTreeMap::<LauncherActionId, GatewaySearchDocument>::new();

        for snapshot in snapshots {
            for tool in snapshot.tools {
                let action_id = LauncherActionId::from_server_tool(&snapshot.server_id, &tool.id);
                if !exposure_policy.allows(&action_id, &snapshot.server_id, &tool.id) {
                    continue;
                }
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
                    documents_by_id.insert(
                        action_id.clone(),
                        GatewaySearchDocument {
                            action_id: action_id.clone(),
                            name: action.display_name.clone(),
                            description: action.description.clone(),
                            metadata: action_metadata(&action),
                            schema_summary: agent_search::schema_summary(&tool.input_schema),
                            catalog_hash: String::new(),
                            truncated: false,
                        },
                    );
                    actions_by_id.insert(action_id, action);
                }
            }
        }

        let mut catalog = Self {
            actions: actions_by_id.into_values().collect(),
            collisions,
            search_documents: documents_by_id.into_values().collect(),
        };
        let catalog_hash = catalog.catalog_hash();
        for document in &mut catalog.search_documents {
            document.catalog_hash = catalog_hash.clone();
        }
        catalog
    }

    pub fn search_documents(&self) -> Vec<GatewaySearchDocument> {
        self.search_documents.clone()
    }

    pub fn diff(&self, next: &GatewayCatalog) -> GatewayCatalogDiff {
        let current = self
            .actions
            .iter()
            .map(|action| (action.id.clone(), action.clone()))
            .collect::<BTreeMap<_, _>>();
        let next_actions = next
            .actions
            .iter()
            .map(|action| (action.id.clone(), action.clone()))
            .collect::<BTreeMap<_, _>>();

        GatewayCatalogDiff {
            added: next_actions
                .keys()
                .filter(|id| !current.contains_key(*id))
                .cloned()
                .collect(),
            removed: current
                .keys()
                .filter(|id| !next_actions.contains_key(*id))
                .cloned()
                .collect(),
            changed: next_actions
                .iter()
                .filter(|(id, next)| current.get(id).is_some_and(|current| current != *next))
                .map(|(id, _next)| id.clone())
                .collect(),
        }
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
            vec![
                server_id.to_string(),
                tool_id.to_string(),
                "mcp tool".to_string(),
            ]
        }
    }
}
