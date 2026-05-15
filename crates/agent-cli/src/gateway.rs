use agent_auth::OAuthStatus;
use agent_gateway::{GatewayCatalog, ProtectedRouteTarget, ResolvedProtectedRoute};
use agent_search::SearchResult;

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
