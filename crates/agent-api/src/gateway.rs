use agent_gateway::{GatewayError, GatewayService};
use agent_protocol::{LauncherActionId, McpServerConfig, McpServerId, ToolInvocation};
use agent_runtime::{McpRuntime, RuntimeError};
use agent_search::{SearchIndex, SearchQuery};
use serde_json::Value;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct GatewayApi {
    runtime: McpRuntime,
    gateway: GatewayService,
}

impl GatewayApi {
    pub async fn start(configs: Vec<McpServerConfig>) -> Self {
        let runtime = McpRuntime::start(configs).await;
        let gateway = GatewayService::from_runtime_snapshots(runtime.snapshots());
        Self { runtime, gateway }
    }

    pub fn list_actions(&self) -> Vec<GatewayApiAction> {
        self.gateway
            .catalog
            .actions
            .iter()
            .map(|action| GatewayApiAction {
                id: action.id.to_string(),
                display_name: action.display_name.clone(),
                description: action.description.clone(),
            })
            .collect()
    }

    pub fn search_actions(&self, query: &str, limit: usize) -> Vec<GatewayApiSearchResult> {
        SearchIndex::new(self.gateway.catalog.search_documents())
            .search(SearchQuery::new(query).limit(limit))
            .into_iter()
            .map(|result| GatewayApiSearchResult {
                action_id: result.action_id.to_string(),
                name: result.name,
                score: result.score,
            })
            .collect()
    }

    pub async fn call_action(&self, action_id: &str, arguments: Value) -> Result<Value, GatewayError> {
        self.gateway
            .invoke(
                &self.runtime,
                ToolInvocation {
                    action_id: LauncherActionId::new(action_id),
                    arguments,
                },
            )
            .await
            .map(|result| result.output)
    }

    pub async fn read_resource(&self, server_id: &str, uri: &str) -> Result<Value, RuntimeError> {
        self.runtime
            .read_resource(&McpServerId::new(server_id), uri)
            .await
            .map(|result| serde_json::to_value(result).unwrap_or(Value::Null))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayApiAction {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayApiSearchResult {
    pub action_id: String,
    pub name: String,
    pub score: u16,
}
