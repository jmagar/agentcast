use agent_api::{GatewayApi, GatewayApiSearchResult};
use agent_mcp::{
    GatewayMcpAction, GatewayMcpBackend, GatewayMcpPrompt, GatewayMcpResource,
    GatewayMcpSearchResult, GatewayMcpServer, GatewayMcpStatus,
};
use serde_json::Value;
use std::sync::Arc;

pub(crate) struct GatewayMcpApiBackend {
    api: Arc<GatewayApi>,
}

impl GatewayMcpApiBackend {
    pub(crate) fn new(api: GatewayApi) -> Self {
        Self { api: Arc::new(api) }
    }
}

#[async_trait::async_trait]
impl GatewayMcpBackend for GatewayMcpApiBackend {
    fn gateway_status(&self) -> GatewayMcpStatus {
        let snapshots = self.api.runtime().snapshots();
        GatewayMcpStatus {
            server_count: snapshots.len(),
            action_count: self.api.list_actions().len(),
        }
    }

    fn list_servers(&self) -> Vec<GatewayMcpServer> {
        self.api
            .runtime()
            .snapshots()
            .into_iter()
            .map(|snapshot| GatewayMcpServer {
                id: snapshot.server_id.to_string(),
                name: snapshot.server_name,
                status: serde_json::to_value(snapshot.status)
                    .ok()
                    .and_then(|value| value.as_str().map(ToOwned::to_owned))
                    .unwrap_or_else(|| "unknown".to_string()),
                tool_count: snapshot.tools.len(),
                resource_count: snapshot.resources.len(),
                prompt_count: snapshot.prompts.len(),
            })
            .collect()
    }

    fn list_actions(&self) -> Vec<GatewayMcpAction> {
        self.api
            .list_actions()
            .into_iter()
            .map(|action| GatewayMcpAction {
                id: action.id,
                name: action.display_name,
                description: action.description,
            })
            .collect()
    }

    fn search_actions(&self, query: &str, limit: usize) -> Vec<GatewayMcpSearchResult> {
        self.api
            .search_actions(query, limit)
            .into_iter()
            .map(search_result)
            .collect()
    }

    async fn call_action(&self, action_id: &str, arguments: Value) -> Result<Value, String> {
        self.api
            .call_action(action_id, arguments)
            .await
            .map_err(|error| error.to_string())
    }

    fn list_resources(&self, server_id: Option<&str>) -> Vec<GatewayMcpResource> {
        self.api
            .runtime()
            .snapshots()
            .into_iter()
            .filter(|snapshot| server_id.is_none_or(|id| snapshot.server_id.as_str() == id))
            .flat_map(|snapshot| {
                let server_id = snapshot.server_id.to_string();
                snapshot
                    .resources
                    .into_iter()
                    .map(move |resource| GatewayMcpResource {
                        server_id: server_id.clone(),
                        uri: resource.uri,
                        name: resource.name,
                        title: resource.title,
                        description: resource.description,
                        mime_type: resource.mime_type,
                    })
            })
            .collect()
    }

    async fn read_resource(&self, server_id: &str, uri: &str) -> Result<Value, String> {
        self.api
            .read_resource(server_id, uri)
            .await
            .map_err(|error| error.to_string())
    }

    fn list_prompts(&self, server_id: Option<&str>) -> Vec<GatewayMcpPrompt> {
        self.api
            .runtime()
            .snapshots()
            .into_iter()
            .filter(|snapshot| server_id.is_none_or(|id| snapshot.server_id.as_str() == id))
            .flat_map(|snapshot| {
                let server_id = snapshot.server_id.to_string();
                snapshot
                    .prompts
                    .into_iter()
                    .map(move |prompt| GatewayMcpPrompt {
                        server_id: server_id.clone(),
                        name: prompt.name,
                        title: prompt.title,
                        description: prompt.description,
                        arguments: prompt.arguments,
                    })
            })
            .collect()
    }

    async fn get_prompt(
        &self,
        server_id: &str,
        name: &str,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<Value, String> {
        self.api
            .runtime()
            .get_prompt(
                &agent_protocol::McpServerId::new(server_id),
                name,
                arguments,
            )
            .await
            .map_err(|error| error.to_string())
    }
}

fn search_result(result: GatewayApiSearchResult) -> GatewayMcpSearchResult {
    GatewayMcpSearchResult {
        action_id: result.action_id,
        name: result.name,
        score: result.score,
        match_kind: "Ranked".to_string(),
        truncated: false,
    }
}
