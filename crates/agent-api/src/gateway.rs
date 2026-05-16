use agent_gateway::{GatewayError, GatewayService};
use agent_protocol::{LauncherActionId, McpServerConfig, McpServerId, ToolInvocation};
use agent_runtime::{McpRuntime, RuntimeCatalogSnapshot, RuntimeError};
use agent_search::{SearchIndex, SearchMatchField, SearchMatchKind, SearchQuery};
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct GatewayApi {
    runtime: Arc<McpRuntime>,
    gateway: GatewayService,
}

impl GatewayApi {
    pub async fn start(configs: Vec<McpServerConfig>) -> Self {
        let runtime = McpRuntime::start(configs).await;
        let gateway = GatewayService::from_runtime_snapshots(runtime.snapshots());
        Self {
            runtime: Arc::new(runtime),
            gateway,
        }
    }

    pub fn runtime(&self) -> Arc<McpRuntime> {
        self.runtime.clone()
    }

    pub fn status(&self) -> GatewayApiStatus {
        let snapshots = self.runtime.snapshots();
        let health = agent_gateway::GatewayHealthSummary::from_catalog_and_snapshots(
            &self.gateway.catalog,
            &snapshots,
        );
        GatewayApiStatus {
            server_count: health.server_count,
            action_count: health.action_count,
            collision_count: health.collision_count,
            status_counts: health.status_counts,
        }
    }

    pub fn list_servers(&self) -> Vec<GatewayApiServer> {
        self.runtime
            .snapshots()
            .into_iter()
            .map(server_view)
            .collect()
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
                match_kind: result.match_kind,
                matched_fields: result.matched_fields,
                matched_terms: result.matched_terms,
                summary: result.summary,
                truncated: result.truncated,
            })
            .collect()
    }

    pub async fn call_action(
        &self,
        action_id: &str,
        arguments: Value,
    ) -> Result<Value, GatewayError> {
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

    pub fn list_resources(&self, server_id: Option<&str>) -> Vec<GatewayApiResource> {
        self.runtime
            .snapshots()
            .into_iter()
            .filter(|snapshot| server_id.is_none_or(|id| snapshot.server_id.as_str() == id))
            .flat_map(|snapshot| {
                let server_id = snapshot.server_id.to_string();
                snapshot
                    .resources
                    .into_iter()
                    .map(move |resource| GatewayApiResource {
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

    pub fn list_prompts(&self, server_id: Option<&str>) -> Vec<GatewayApiPrompt> {
        self.runtime
            .snapshots()
            .into_iter()
            .filter(|snapshot| server_id.is_none_or(|id| snapshot.server_id.as_str() == id))
            .flat_map(|snapshot| {
                let server_id = snapshot.server_id.to_string();
                snapshot
                    .prompts
                    .into_iter()
                    .map(move |prompt| GatewayApiPrompt {
                        server_id: server_id.clone(),
                        name: prompt.name,
                        title: prompt.title,
                        description: prompt.description,
                        arguments: prompt.arguments,
                    })
            })
            .collect()
    }

    pub async fn get_prompt(
        &self,
        server_id: &str,
        name: &str,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<Value, RuntimeError> {
        self.runtime
            .get_prompt(&McpServerId::new(server_id), name, arguments)
            .await
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GatewayApiStatus {
    pub server_count: usize,
    pub action_count: usize,
    pub collision_count: usize,
    pub status_counts: std::collections::BTreeMap<String, usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GatewayApiServer {
    pub id: String,
    pub name: String,
    pub status: agent_protocol::ServerStatus,
    pub tool_count: usize,
    pub resource_count: usize,
    pub prompt_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GatewayApiAction {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GatewayApiSearchResult {
    pub action_id: String,
    pub name: String,
    pub score: u16,
    pub match_kind: SearchMatchKind,
    pub matched_fields: Vec<SearchMatchField>,
    pub matched_terms: Vec<String>,
    pub summary: Option<String>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GatewayApiResource {
    pub server_id: String,
    pub uri: String,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GatewayApiPrompt {
    pub server_id: String,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub arguments: Value,
}

fn server_view(snapshot: RuntimeCatalogSnapshot) -> GatewayApiServer {
    GatewayApiServer {
        id: snapshot.server_id.to_string(),
        name: snapshot.server_name,
        status: snapshot.status,
        tool_count: snapshot.tools.len(),
        resource_count: snapshot.resources.len(),
        prompt_count: snapshot.prompts.len(),
    }
}
