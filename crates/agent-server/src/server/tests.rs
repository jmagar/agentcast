use super::*;
use agent_auth::{BearerTokenVerifier, ScopeSet};
use agent_gateway::{ProtectedRouteConfig, ProtectedRouteTarget};
use agent_protocol::McpServerId;

#[test]
fn protected_mcp_static_verifier_requires_token() {
    let routes = protected_routes();

    let error = protected_mcp_static_verifier(None, &routes).expect_err("missing token fails");

    assert!(
        error
            .to_string()
            .contains("AGENTCAST_PROTECTED_MCP_BEARER_TOKEN")
    );
}

#[test]
fn protected_mcp_static_verifier_authorizes_configured_token() {
    let routes = protected_routes();
    let token = "secret-token".to_string();
    let verifier = protected_mcp_static_verifier(Some(&token), &routes).expect("verifier");

    let claims = verifier.verify("Bearer secret-token").expect("claims");

    assert_eq!(claims.subject, "protected-mcp-static");
    assert_eq!(claims.audience, "https://mcp.example.test/syslog");
    assert!(
        claims
            .scopes
            .contains_all(&ScopeSet::parse("mcp:read").expect("scope"))
    );
    assert!(verifier.verify("Bearer wrong-token").is_err());
}

fn protected_routes() -> ProtectedRouteIndex {
    ProtectedRouteIndex::from_routes(vec![ProtectedRouteConfig {
        name: "syslog".to_string(),
        enabled: true,
        public_host: "mcp.example.test".to_string(),
        public_path: "/syslog".to_string(),
        resource_uri: "https://mcp.example.test/syslog".to_string(),
        authorization_servers: vec!["https://auth.example.test".to_string()],
        required_scopes: ScopeSet::parse("mcp:read").expect("scope"),
        target: ProtectedRouteTarget::UpstreamMcp {
            server_id: McpServerId::new("fixture"),
        },
    }])
    .expect("routes")
}
