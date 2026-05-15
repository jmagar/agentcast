use agent_auth::OAuthStatus;
use agent_gateway::{GatewayCatalog, GatewayService, ProtectedRouteTarget, ResolvedProtectedRoute};
use agent_protocol::{LauncherActionId, McpServerConfig, McpServerId, ToolInvocation};
use agent_runtime::McpRuntime;
use agent_search::{SearchIndex, SearchQuery, SearchResult};
use serde_json::Value;

#[cfg(test)]
mod tests;

pub struct GatewayCliView;

impl GatewayCliView {
    pub fn actions(catalog: &GatewayCatalog) -> Vec<GatewayActionRow> {
        catalog
            .actions
            .iter()
            .map(|action| GatewayActionRow {
                id: action.id.to_string(),
                name: action.display_name.clone(),
                description: action.description.clone(),
            })
            .collect()
    }

    pub fn search_results(results: &[SearchResult]) -> Vec<GatewaySearchRow> {
        results
            .iter()
            .map(|result| GatewaySearchRow {
                action_id: result.action_id.to_string(),
                name: result.name.clone(),
                score: result.score,
                match_kind: format!("{:?}", result.match_kind),
                truncated: result.truncated,
            })
            .collect()
    }

    pub fn protected_route(route: &ResolvedProtectedRoute) -> ProtectedRouteRow {
        ProtectedRouteRow {
            name: route.name.clone(),
            resource_uri: route.resource_uri.clone(),
            metadata_path: route.metadata_path.clone(),
            required_scopes: route.required_scopes.as_slice().to_vec(),
            target: match &route.target {
                ProtectedRouteTarget::UpstreamMcp { server_id } => {
                    format!("upstream_mcp:{server_id}")
                }
            },
        }
    }

    pub fn oauth_status(subject: &str, upstream_id: &str, status: OAuthStatus) -> OAuthStatusRow {
        OAuthStatusRow {
            subject: subject.to_string(),
            upstream_id: upstream_id.to_string(),
            status: format!("{status:?}").to_ascii_lowercase(),
        }
    }
}

#[derive(Debug)]
pub struct GatewayCliHandlers {
    runtime: McpRuntime,
    gateway: GatewayService,
}

impl GatewayCliHandlers {
    pub async fn start(configs: Vec<McpServerConfig>) -> Self {
        let runtime = McpRuntime::start(configs).await;
        let gateway = GatewayService::from_runtime_snapshots(runtime.snapshots());
        Self { runtime, gateway }
    }

    pub fn list_actions(&self) -> Vec<GatewayActionRow> {
        GatewayCliView::actions(&self.gateway.catalog)
    }

    pub fn search_actions(&self, query: &str, limit: usize) -> Vec<GatewaySearchRow> {
        let index = SearchIndex::new(self.gateway.catalog.search_documents());
        let results = index.search(SearchQuery::new(query).limit(limit));
        GatewayCliView::search_results(&results)
    }

    pub async fn call_action(
        &self,
        action_id: &str,
        arguments: Value,
    ) -> Result<Value, agent_gateway::GatewayError> {
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

    pub async fn read_resource(
        &self,
        server_id: &str,
        uri: &str,
    ) -> Result<Value, agent_runtime::RuntimeError> {
        self.runtime
            .read_resource(&McpServerId::new(server_id), uri)
            .await
            .map(|result| serde_json::to_value(result).unwrap_or(Value::Null))
    }

    pub async fn get_prompt(
        &self,
        server_id: &str,
        name: &str,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<Value, agent_runtime::RuntimeError> {
        self.runtime
            .get_prompt(&McpServerId::new(server_id), name, arguments)
            .await
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayActionRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewaySearchRow {
    pub action_id: String,
    pub name: String,
    pub score: u16,
    pub match_kind: String,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedRouteRow {
    pub name: String,
    pub resource_uri: String,
    pub metadata_path: String,
    pub required_scopes: Vec<String>,
    pub target: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OAuthStatusRow {
    pub subject: String,
    pub upstream_id: String,
    pub status: String,
}
