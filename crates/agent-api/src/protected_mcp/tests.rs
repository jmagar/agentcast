use super::*;
use agent_auth::ScopeSet;
use agent_gateway::{ProtectedRouteConfig, ProtectedRouteTarget};
use agent_protocol::McpServerId;

fn api() -> ProtectedMcpRouteApi {
    let routes = ProtectedRouteIndex::from_routes(vec![ProtectedRouteConfig {
        name: "syslog".to_string(),
        enabled: true,
        public_host: "mcp.example.test".to_string(),
        public_path: "/syslog".to_string(),
        resource_uri: "https://mcp.example.test/syslog".to_string(),
        authorization_servers: vec!["https://auth.example.test".to_string()],
        required_scopes: ScopeSet::parse("mcp:read").expect("scope"),
        target: ProtectedRouteTarget::UpstreamMcp {
            server_id: McpServerId::new("syslog"),
        },
    }])
    .expect("route index");

    ProtectedMcpRouteApi::new(routes)
}

#[test]
fn metadata_request_returns_protected_resource_metadata() {
    let response = api().handle(ProtectedMcpRequest {
        host: "mcp.example.test".to_string(),
        path: "/.well-known/oauth-protected-resource/syslog".to_string(),
        public_origin: "https://mcp.example.test".to_string(),
        authorization: None,
    });

    let ProtectedMcpResponse::Metadata { status, metadata } = response else {
        panic!("expected metadata");
    };
    assert_eq!(status, ResponseStatus::Ok);
    assert_eq!(metadata.resource, "https://mcp.example.test/syslog");
}

#[test]
fn missing_bearer_returns_unauthorized_challenge() {
    let response = api().handle(ProtectedMcpRequest {
        host: "mcp.example.test".to_string(),
        path: "/syslog".to_string(),
        public_origin: "https://mcp.example.test".to_string(),
        authorization: None,
    });

    let ProtectedMcpResponse::Challenge {
        status,
        www_authenticate,
    } = response
    else {
        panic!("expected challenge");
    };
    assert_eq!(status, ResponseStatus::Unauthorized);
    assert!(www_authenticate.contains("resource_metadata="));
}

#[test]
fn insufficient_scope_returns_forbidden_challenge() {
    let response = api().handle(ProtectedMcpRequest {
        host: "mcp.example.test".to_string(),
        path: "/syslog".to_string(),
        public_origin: "https://mcp.example.test".to_string(),
        authorization: Some(
            "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:write".to_string(),
        ),
    });

    let ProtectedMcpResponse::Challenge {
        status,
        www_authenticate,
    } = response
    else {
        panic!("expected challenge");
    };
    assert_eq!(status, ResponseStatus::Forbidden);
    assert!(www_authenticate.contains("insufficient_scope"));
}

#[test]
fn authorized_request_returns_dispatch_target() {
    let response = api().handle(ProtectedMcpRequest {
        host: "mcp.example.test".to_string(),
        path: "/syslog/tools/list".to_string(),
        public_origin: "https://mcp.example.test".to_string(),
        authorization: Some(
            "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read".to_string(),
        ),
    });

    let ProtectedMcpResponse::DispatchAllowed {
        status,
        subject,
        target,
    } = response
    else {
        panic!("expected dispatch");
    };
    assert_eq!(status, ResponseStatus::Accepted);
    assert_eq!(subject, "user-1");
    assert_eq!(
        target,
        ProtectedRouteTarget::UpstreamMcp {
            server_id: McpServerId::new("syslog")
        }
    );
}
