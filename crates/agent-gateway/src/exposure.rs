use agent_protocol::{LauncherActionId, McpServerId, McpToolId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct GatewayExposurePolicy {
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub allow_actions: BTreeSet<LauncherActionId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub deny_actions: BTreeSet<LauncherActionId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub allow_servers: BTreeSet<McpServerId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub deny_servers: BTreeSet<McpServerId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub allow_tools: BTreeSet<McpToolId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub deny_tools: BTreeSet<McpToolId>,
}

impl GatewayExposurePolicy {
    #[must_use]
    pub fn allows(
        &self,
        action_id: &LauncherActionId,
        server_id: &McpServerId,
        tool_id: &McpToolId,
    ) -> bool {
        if self.deny_actions.contains(action_id)
            || self.deny_servers.contains(server_id)
            || self.deny_tools.contains(tool_id)
        {
            return false;
        }

        let has_allowlist = !self.allow_actions.is_empty()
            || !self.allow_servers.is_empty()
            || !self.allow_tools.is_empty();
        if !has_allowlist {
            return true;
        }

        self.allow_actions.contains(action_id)
            || self.allow_servers.contains(server_id)
            || self.allow_tools.contains(tool_id)
    }

    #[must_use]
    pub fn allow_action(mut self, action_id: LauncherActionId) -> Self {
        self.allow_actions.insert(action_id);
        self
    }

    #[must_use]
    pub fn deny_action(mut self, action_id: LauncherActionId) -> Self {
        self.deny_actions.insert(action_id);
        self
    }

    #[must_use]
    pub fn allow_server(mut self, server_id: McpServerId) -> Self {
        self.allow_servers.insert(server_id);
        self
    }

    #[must_use]
    pub fn deny_server(mut self, server_id: McpServerId) -> Self {
        self.deny_servers.insert(server_id);
        self
    }

    #[must_use]
    pub fn allow_tool(mut self, tool_id: McpToolId) -> Self {
        self.allow_tools.insert(tool_id);
        self
    }

    #[must_use]
    pub fn deny_tool(mut self, tool_id: McpToolId) -> Self {
        self.deny_tools.insert(tool_id);
        self
    }
}

#[cfg(test)]
mod tests;
