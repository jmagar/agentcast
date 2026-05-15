use crate::ScopeSet;
use serde::{Deserialize, Serialize};

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
