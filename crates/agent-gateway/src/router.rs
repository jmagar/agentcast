use agent_protocol::{LauncherActionId, McpServerId, McpToolId};
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionRoute {
    pub action_id: LauncherActionId,
    pub server_id: McpServerId,
    pub tool_id: McpToolId,
}

#[derive(Clone, Debug, Default)]
pub struct GatewayRouter {
    routes: BTreeMap<LauncherActionId, ActionRoute>,
}

impl GatewayRouter {
    pub fn new(routes: Vec<ActionRoute>) -> Self {
        Self {
            routes: routes
                .into_iter()
                .map(|route| (route.action_id.clone(), route))
                .collect(),
        }
    }

    pub fn resolve(&self, action_id: &LauncherActionId) -> Option<&ActionRoute> {
        self.routes.get(action_id)
    }
}
