use crate::ScopeSet;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ProtectedResourceMetadata {
    pub resource: String,
    pub authorization_servers: Vec<String>,
    pub scopes_supported: ScopeSet,
    pub bearer_methods_supported: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthChallenge {
    resource_metadata: String,
    error: Option<String>,
    scope: ScopeSet,
}

impl AuthChallenge {
    pub fn unauthorized(resource_metadata: impl Into<String>) -> Self {
        Self {
            resource_metadata: resource_metadata.into(),
            error: None,
            scope: ScopeSet::parse("").expect("empty scope set is valid"),
        }
    }

    pub fn insufficient_scope(resource_metadata: impl Into<String>, scope: ScopeSet) -> Self {
        Self {
            resource_metadata: resource_metadata.into(),
            error: Some("insufficient_scope".to_string()),
            scope,
        }
    }

    pub fn www_authenticate(&self) -> String {
        let mut parts = Vec::new();
        if let Some(error) = &self.error {
            parts.push(format!("error=\"{error}\""));
        }
        parts.push(format!(
            "resource_metadata=\"{}\"",
            self.resource_metadata
        ));
        if !self.scope.is_empty() {
            parts.push(format!("scope=\"{}\"", self.scope.as_header_value()));
        }
        format!("Bearer {}", parts.join(", "))
    }
}
