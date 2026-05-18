use super::*;
use agent_auth::{BearerClaims, ScopeSet, StaticBearerTokenVerifier};
use agent_gateway::{ProtectedRouteConfig, ProtectedRouteTarget};
use agent_protocol::{McpServerConfig, McpServerId, McpTransportConfig};
use agent_runtime::McpRuntime;
use serde_json::json;
use std::{collections::BTreeMap, sync::Arc};

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

    ProtectedMcpRouteApi::new_with_fixture_verifier(routes)
}

fn route_index() -> ProtectedRouteIndex {
    ProtectedRouteIndex::from_routes(vec![ProtectedRouteConfig {
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
    .expect("route index")
}

#[test]
fn static_bearer_verifier_authorizes_opaque_tokens() {
    let verifier = StaticBearerTokenVerifier::new([(
        "opaque-token",
        BearerClaims {
            subject: "user-1".to_string(),
            audience: "https://mcp.example.test/syslog".to_string(),
            scopes: ScopeSet::parse("mcp:read").expect("scope"),
        },
    )]);
    let api = ProtectedMcpRouteApi::new_with_verifier(route_index(), Arc::new(verifier));

    let response = api.handle(ProtectedMcpRequest {
        host: "mcp.example.test".to_string(),
        path: "/syslog".to_string(),
        public_origin: "https://mcp.example.test".to_string(),
        authorization: Some("Bearer opaque-token".to_string()),
    });

    let ProtectedMcpResponse::DispatchAllowed {
        status, subject, ..
    } = response
    else {
        panic!("expected dispatch");
    };
    assert_eq!(status, ResponseStatus::Accepted);
    assert_eq!(subject, "user-1");
}

#[test]
fn default_verifier_rejects_fixture_shaped_authorization_header() {
    let api = ProtectedMcpRouteApi::new(route_index());

    let response = api.handle(ProtectedMcpRequest {
        host: "mcp.example.test".to_string(),
        path: "/syslog".to_string(),
        public_origin: "https://mcp.example.test".to_string(),
        authorization: Some(
            "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read".to_string(),
        ),
    });

    let ProtectedMcpResponse::Challenge { status, .. } = response else {
        panic!("expected challenge");
    };
    assert_eq!(status, ResponseStatus::Unauthorized);
}

#[test]
fn fixture_verifier_must_be_selected_explicitly() {
    let api = ProtectedMcpRouteApi::new_with_fixture_verifier(route_index());

    let response = api.handle(ProtectedMcpRequest {
        host: "mcp.example.test".to_string(),
        path: "/syslog".to_string(),
        public_origin: "https://mcp.example.test".to_string(),
        authorization: Some(
            "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read".to_string(),
        ),
    });

    let ProtectedMcpResponse::DispatchAllowed {
        status, subject, ..
    } = response
    else {
        panic!("expected dispatch");
    };
    assert_eq!(status, ResponseStatus::Accepted);
    assert_eq!(subject, "user-1");
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

#[tokio::test]
async fn authorized_json_rpc_call_dispatches_to_upstream_runtime() {
    let runtime = runtime().await;

    let response = api()
        .handle_json_rpc(
            &runtime,
            ProtectedMcpJsonRpcRequest {
                host: "mcp.example.test".to_string(),
                path: "/syslog".to_string(),
                public_origin: "https://mcp.example.test".to_string(),
                authorization: Some(
                    "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read"
                        .to_string(),
                ),
                body: json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": {
                        "name": "echo",
                        "arguments": { "message": "hello" }
                    }
                }),
            },
        )
        .await;

    let ProtectedMcpJsonRpcResponse::JsonRpc(body) = response else {
        panic!("expected json-rpc response");
    };
    assert_eq!(body["id"], 1);
    assert_eq!(body["result"]["content"][0]["text"], "hello");
}

#[tokio::test]
async fn unauthenticated_json_rpc_request_is_rejected_before_dispatch() {
    let runtime = runtime().await;

    let response = api()
        .handle_json_rpc(
            &runtime,
            ProtectedMcpJsonRpcRequest {
                host: "mcp.example.test".to_string(),
                path: "/syslog".to_string(),
                public_origin: "https://mcp.example.test".to_string(),
                authorization: None,
                body: json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/list"
                }),
            },
        )
        .await;

    let ProtectedMcpJsonRpcResponse::Rejected(ProtectedMcpResponse::Challenge { status, .. }) =
        response
    else {
        panic!("expected rejected challenge");
    };
    assert_eq!(status, ResponseStatus::Unauthorized);
}

#[tokio::test]
async fn authorized_json_rpc_lists_resources_and_prompts_from_runtime() {
    let runtime = runtime().await;

    let resources = api()
        .handle_json_rpc(
            &runtime,
            protected_request(json!({
                "jsonrpc": "2.0",
                "id": "resources",
                "method": "resources/list"
            })),
        )
        .await;
    let ProtectedMcpJsonRpcResponse::JsonRpc(resources) = resources else {
        panic!("expected resources");
    };
    assert_eq!(resources["result"]["resources"][0]["uri"], "fixture://echo");

    let prompts = api()
        .handle_json_rpc(
            &runtime,
            protected_request(json!({
                "jsonrpc": "2.0",
                "id": "prompts",
                "method": "prompts/list"
            })),
        )
        .await;
    let ProtectedMcpJsonRpcResponse::JsonRpc(prompts) = prompts else {
        panic!("expected prompts");
    };
    assert_eq!(prompts["result"]["prompts"][0]["name"], "summarize");
}

#[tokio::test]
async fn authorized_json_rpc_reads_resources_and_gets_prompts() {
    let runtime = runtime().await;

    let resource = api()
        .handle_json_rpc(
            &runtime,
            protected_request(json!({
                "jsonrpc": "2.0",
                "id": "resource",
                "method": "resources/read",
                "params": { "uri": "fixture://echo" }
            })),
        )
        .await;
    let ProtectedMcpJsonRpcResponse::JsonRpc(resource) = resource else {
        panic!("expected resource");
    };
    assert_eq!(
        resource["result"]["contents"][0]["text"],
        "fixture resource"
    );

    let prompt = api()
        .handle_json_rpc(
            &runtime,
            protected_request(json!({
                "jsonrpc": "2.0",
                "id": "prompt",
                "method": "prompts/get",
                "params": {
                    "name": "summarize",
                    "arguments": { "topic": "AgentCast" }
                }
            })),
        )
        .await;
    let ProtectedMcpJsonRpcResponse::JsonRpc(prompt) = prompt else {
        panic!("expected prompt");
    };
    assert_eq!(
        prompt["result"]["messages"][0]["content"]["text"],
        "Summarize AgentCast"
    );
}

fn protected_request(body: serde_json::Value) -> ProtectedMcpJsonRpcRequest {
    ProtectedMcpJsonRpcRequest {
        host: "mcp.example.test".to_string(),
        path: "/syslog".to_string(),
        public_origin: "https://mcp.example.test".to_string(),
        authorization: Some(
            "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read".to_string(),
        ),
        body,
    }
}

async fn runtime() -> McpRuntime {
    let fixture = format!(
        "{}/../agent-mcp/src/fixtures/mcp_echo_server.js",
        env!("CARGO_MANIFEST_DIR")
    );
    McpRuntime::start(vec![McpServerConfig {
        id: McpServerId::new("syslog"),
        name: "Syslog".to_string(),
        enabled: true,
        transport: McpTransportConfig::Stdio {
            command: "node".to_string(),
            args: vec![fixture],
            env: BTreeMap::new(),
        },
        env_keys: Vec::new(),
    }])
    .await
}
