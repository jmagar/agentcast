use agent_auth::OAuthStatus;
use agent_gateway::{GatewayCatalog, GatewayService, ProtectedRouteTarget, ResolvedProtectedRoute};
use agent_protocol::{LauncherActionId, McpServerConfig, McpServerId, ToolInvocation};
use agent_runtime::McpRuntime;
use agent_search::{SearchIndex, SearchQuery, SearchResult};
use serde::Serialize;
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
                matched_fields: result
                    .matched_fields
                    .iter()
                    .map(|field| format!("{field:?}"))
                    .collect(),
                matched_terms: result.matched_terms.clone(),
                summary: result.summary.clone(),
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

    pub fn render_actions_table(rows: &[GatewayActionRow]) -> String {
        render_table(
            &["ACTION ID", "NAME", "DESCRIPTION"],
            rows.iter().map(|row| {
                [
                    row.id.clone(),
                    row.name.clone(),
                    row.description.clone().unwrap_or_default(),
                ]
            }),
        )
    }

    pub fn render_search_table(rows: &[GatewaySearchRow]) -> String {
        render_table(
            &["ACTION ID", "NAME", "SCORE", "MATCH", "FIELDS", "SUMMARY"],
            rows.iter().map(|row| {
                [
                    row.action_id.clone(),
                    row.name.clone(),
                    row.score.to_string(),
                    row.match_kind.clone(),
                    row.matched_fields.join(","),
                    row.summary.clone().unwrap_or_default(),
                ]
            }),
        )
    }

    pub fn render_oauth_status(row: &OAuthStatusRow) -> String {
        render_table(
            &["SUBJECT", "UPSTREAM", "STATUS"],
            [[
                row.subject.clone(),
                row.upstream_id.clone(),
                row.status.clone(),
            ]],
        )
    }

    pub fn render_protected_route(row: &ProtectedRouteRow) -> String {
        render_table(
            &["NAME", "RESOURCE", "METADATA", "SCOPES", "TARGET"],
            [[
                row.name.clone(),
                row.resource_uri.clone(),
                row.metadata_path.clone(),
                row.required_scopes.join(" "),
                row.target.clone(),
            ]],
        )
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GatewayActionRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GatewaySearchRow {
    pub action_id: String,
    pub name: String,
    pub score: u16,
    pub match_kind: String,
    pub matched_fields: Vec<String>,
    pub matched_terms: Vec<String>,
    pub summary: Option<String>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProtectedRouteRow {
    pub name: String,
    pub resource_uri: String,
    pub metadata_path: String,
    pub required_scopes: Vec<String>,
    pub target: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OAuthStatusRow {
    pub subject: String,
    pub upstream_id: String,
    pub status: String,
}

fn render_table<const N: usize>(
    headers: &[&str; N],
    rows: impl IntoIterator<Item = [String; N]>,
) -> String {
    let rows = rows.into_iter().collect::<Vec<_>>();
    let mut widths = headers.map(str::len);
    for row in &rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(cell.len());
        }
    }

    let mut output = String::new();
    render_row(&mut output, &headers.map(str::to_string), &widths);
    render_separator(&mut output, &widths);
    for row in rows {
        render_row(&mut output, &row, &widths);
    }
    output.trim_end().to_string()
}

fn render_row<const N: usize>(output: &mut String, cells: &[String; N], widths: &[usize; N]) {
    for (index, cell) in cells.iter().enumerate() {
        if index > 0 {
            output.push_str("  ");
        }
        output.push_str(cell);
        for _ in cell.len()..widths[index] {
            output.push(' ');
        }
    }
    output.push('\n');
}

fn render_separator<const N: usize>(output: &mut String, widths: &[usize; N]) {
    for (index, width) in widths.iter().enumerate() {
        if index > 0 {
            output.push_str("  ");
        }
        output.push_str(&"-".repeat(*width));
    }
    output.push('\n');
}
