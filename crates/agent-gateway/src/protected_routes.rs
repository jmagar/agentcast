use crate::GatewayError;
use agent_auth::{AuthDecision, BearerClaims, ScopeSet};
use agent_protocol::McpServerId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProtectedRouteConfig {
    pub name: String,
    pub enabled: bool,
    pub public_host: String,
    pub public_path: String,
    pub resource_uri: String,
    pub authorization_servers: Vec<String>,
    pub required_scopes: ScopeSet,
    pub target: ProtectedRouteTarget,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProtectedRouteCollection {
    #[serde(default)]
    pub routes: Vec<ProtectedRouteConfig>,
}

impl ProtectedRouteCollection {
    pub fn new(routes: Vec<ProtectedRouteConfig>) -> Result<Self, GatewayError> {
        let collection = Self { routes };
        collection.validate()?;
        Ok(collection)
    }

    pub fn validate(&self) -> Result<(), GatewayError> {
        ProtectedRouteIndex::from_routes(self.routes.clone()).map(|_| ())
    }

    pub fn index(&self) -> Result<ProtectedRouteIndex, GatewayError> {
        ProtectedRouteIndex::from_routes(self.routes.clone())
    }

    pub fn list(&self) -> &[ProtectedRouteConfig] {
        &self.routes
    }

    pub fn get(&self, name: &str) -> Option<&ProtectedRouteConfig> {
        self.routes.iter().find(|route| route.name == name)
    }

    pub fn upsert(&mut self, route: ProtectedRouteConfig) -> Result<(), GatewayError> {
        let mut next = self.routes.clone();
        match next.iter().position(|existing| existing.name == route.name) {
            Some(index) => next[index] = route,
            None => next.push(route),
        }
        let next = Self::new(next)?;
        self.routes = next.routes;
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> Result<ProtectedRouteConfig, GatewayError> {
        let index = self
            .routes
            .iter()
            .position(|route| route.name == name)
            .ok_or_else(|| GatewayError::ProtectedRouteNotFound(name.to_string()))?;
        Ok(self.routes.remove(index))
    }

    pub fn status(&self, name: &str) -> Result<ProtectedRouteStatus, GatewayError> {
        let route = self
            .get(name)
            .ok_or_else(|| GatewayError::ProtectedRouteNotFound(name.to_string()))?;
        let index = self.index()?;
        Ok(ProtectedRouteStatus {
            name: route.name.clone(),
            enabled: route.enabled,
            resolves_public_route: route.enabled
                && index
                    .resolve(&route.public_host, &route.public_path)
                    .is_some_and(|resolved| resolved.name == route.name),
            resolves_metadata_route: route.enabled
                && index
                    .resolve_metadata(
                        &route.public_host,
                        &format!(
                            "/.well-known/oauth-protected-resource{}",
                            first_path_segment(&route.public_path)?
                        ),
                    )
                    .is_some_and(|resolved| resolved.name == route.name),
        })
    }

    pub fn test(
        &self,
        host: &str,
        path: &str,
    ) -> Result<Option<ResolvedProtectedRoute>, GatewayError> {
        Ok(self.index()?.resolve(host, path).cloned())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProtectedRouteStatus {
    pub name: String,
    pub enabled: bool,
    pub resolves_public_route: bool,
    pub resolves_metadata_route: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProtectedRouteTarget {
    UpstreamMcp { server_id: McpServerId },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedProtectedRoute {
    pub name: String,
    pub resource_uri: String,
    pub metadata_path: String,
    pub authorization_servers: Vec<String>,
    pub required_scopes: ScopeSet,
    pub target: ProtectedRouteTarget,
}

impl ResolvedProtectedRoute {
    pub fn protected_resource_metadata(&self) -> agent_auth::ProtectedResourceMetadata {
        agent_auth::ProtectedResourceMetadata {
            resource: self.resource_uri.clone(),
            authorization_servers: self.authorization_servers.clone(),
            scopes_supported: self.required_scopes.clone(),
            bearer_methods_supported: vec!["header".to_string()],
        }
    }

    pub fn authorize(&self, claims: Option<&BearerClaims>, public_origin: &str) -> AuthDecision {
        AuthDecision::authorize_route(
            claims,
            &self.resource_uri,
            &format!("{public_origin}{}", self.metadata_path),
            &self.required_scopes,
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProtectedRouteIndex {
    routes: BTreeMap<(String, String), ResolvedProtectedRoute>,
    metadata_routes: BTreeMap<(String, String), (String, String)>,
}

impl ProtectedRouteIndex {
    pub fn from_routes(routes: Vec<ProtectedRouteConfig>) -> Result<Self, GatewayError> {
        let mut index = Self::default();

        for route in routes.into_iter().filter(|route| route.enabled) {
            let host = normalize_host(&route.public_host)?;
            let segment = first_path_segment(&route.public_path)?;
            let key = (host.clone(), segment.clone());
            if index.routes.contains_key(&key) {
                return Err(GatewayError::DuplicateProtectedRoute {
                    host,
                    path: segment,
                });
            }

            let metadata_path = format!("/.well-known/oauth-protected-resource{}", segment);
            index.metadata_routes.insert(
                (host.clone(), metadata_path.clone()),
                (host.clone(), segment.clone()),
            );
            index.routes.insert(
                key,
                ResolvedProtectedRoute {
                    name: route.name,
                    resource_uri: route.resource_uri,
                    metadata_path,
                    authorization_servers: route.authorization_servers,
                    required_scopes: route.required_scopes,
                    target: route.target,
                },
            );
        }

        Ok(index)
    }

    pub fn resolve(&self, host: &str, path: &str) -> Option<&ResolvedProtectedRoute> {
        let host = normalize_host(host).ok()?;
        let segment = first_path_segment(path).ok()?;
        self.routes.get(&(host, segment))
    }

    pub fn resolve_metadata(&self, host: &str, path: &str) -> Option<&ResolvedProtectedRoute> {
        let host = normalize_host(host).ok()?;
        let route_key = self.metadata_routes.get(&(host, path.to_string()))?.clone();
        self.routes.get(&route_key)
    }
}

fn normalize_host(raw: &str) -> Result<String, GatewayError> {
    let first_forwarded = raw.split(',').next().unwrap_or_default().trim();
    let without_port = first_forwarded.split(':').next().unwrap_or_default();
    let host = without_port.trim_end_matches('.').to_ascii_lowercase();

    if host.is_empty() || host.contains('/') || host.contains('\\') {
        return Err(GatewayError::InvalidPublicHost(raw.to_string()));
    }

    Ok(host)
}

fn first_path_segment(raw: &str) -> Result<String, GatewayError> {
    if !raw.starts_with('/') || raw == "/" {
        return Err(GatewayError::InvalidPublicPath(raw.to_string()));
    }

    let lower = raw.to_ascii_lowercase();
    if lower.starts_with("/.well-known")
        || lower.starts_with("/v1")
        || lower.contains("//")
        || lower.contains("/.")
        || lower.contains("%2f")
        || lower.contains("%5c")
        || lower.contains("%2e")
        || raw.contains('?')
        || raw.contains('#')
    {
        return Err(GatewayError::InvalidPublicPath(raw.to_string()));
    }

    let segment = raw
        .trim_start_matches('/')
        .split('/')
        .next()
        .unwrap_or_default();

    if segment.is_empty() {
        return Err(GatewayError::InvalidPublicPath(raw.to_string()));
    }

    Ok(format!("/{segment}"))
}
