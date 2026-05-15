use super::*;
use crate::ScopeSet;

#[test]
fn metadata_serializes_authorization_servers_and_scopes() {
    let metadata = ProtectedResourceMetadata {
        resource: "https://mcp.example.test/syslog".to_string(),
        authorization_servers: vec!["https://auth.example.test".to_string()],
        scopes_supported: ScopeSet::parse("mcp:read mcp:write").expect("parse scopes"),
        bearer_methods_supported: vec!["header".to_string()],
    };

    let value = serde_json::to_value(&metadata).expect("serialize metadata");

    assert_eq!(value["resource"], "https://mcp.example.test/syslog");
    assert_eq!(
        value["authorization_servers"][0],
        "https://auth.example.test"
    );
    assert_eq!(value["scopes_supported"][0], "mcp:read");
    assert_eq!(value["bearer_methods_supported"][0], "header");
}

#[test]
fn unauthorized_challenge_points_to_resource_metadata() {
    let challenge = AuthChallenge::unauthorized(
        "https://mcp.example.test/.well-known/oauth-protected-resource/syslog",
    );

    assert_eq!(
        challenge.www_authenticate(),
        "Bearer resource_metadata=\"https://mcp.example.test/.well-known/oauth-protected-resource/syslog\""
    );
}

#[test]
fn insufficient_scope_challenge_includes_required_scopes() {
    let scopes = ScopeSet::parse("mcp:read").expect("parse scopes");
    let challenge = AuthChallenge::insufficient_scope(
        "https://mcp.example.test/.well-known/oauth-protected-resource/syslog",
        scopes,
    );

    assert_eq!(
        challenge.www_authenticate(),
        "Bearer error=\"insufficient_scope\", resource_metadata=\"https://mcp.example.test/.well-known/oauth-protected-resource/syslog\", scope=\"mcp:read\""
    );
}
