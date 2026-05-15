use agent_protocol::{LauncherAction, LauncherActionId, LauncherActionKind};
use agent_runtime::RuntimeCatalogSnapshot;
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
}
