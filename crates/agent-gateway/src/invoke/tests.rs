use super::*;
use agent_auth::{OAuthCredential, ScopeSet};
use agent_protocol::{
    LauncherActionId, McpServerConfig, McpServerId, McpToolId, McpTransportConfig,
};
use agent_runtime::McpRuntime;
use agent_runtime::{RuntimeCatalogSnapshot, RuntimeTool};
use serde_json::json;
use std::collections::BTreeMap;

fn fixture_config() -> McpServerConfig {
    let server = format!(
        "{}/../agent-mcp/src/fixtures/mcp_echo_server.js",
        env!("CARGO_MANIFEST_DIR")
    );
    McpServerConfig {
        id: McpServerId::new("fixture"),
        name: "Fixture".to_string(),
        enabled: true,
        transport: McpTransportConfig::Stdio {
            command: "node".to_string(),
            args: vec![server],
            env: BTreeMap::new(),
        },
        env_keys: Vec::new(),
    }
}

#[tokio::test]
async fn gateway_invokes_action_through_runtime_route() {
    let runtime = McpRuntime::start(vec![fixture_config()]).await;
    let gateway = GatewayService::from_runtime_snapshots(runtime.snapshots());
    let action_id =
        LauncherActionId::from_server_tool(&McpServerId::new("fixture"), &McpToolId::new("echo"));

    let result = gateway
        .invoke(
            &runtime,
            ToolInvocation {
                action_id,
                arguments: json!({"message": "hello"}),
            },
        )
        .await
        .expect("invoke");

    assert_eq!(result.output["content"][0]["text"], "hello");
}

#[tokio::test]
async fn unknown_action_returns_structured_gateway_error() {
    let runtime = McpRuntime::start(vec![fixture_config()]).await;
    let gateway = GatewayService::from_runtime_snapshots(runtime.snapshots());

    let error = gateway
        .invoke(
            &runtime,
            ToolInvocation {
                action_id: LauncherActionId::new("missing"),
                arguments: json!({}),
            },
        )
        .await
        .expect_err("unknown action");

    assert_eq!(error, GatewayError::UnknownAction("missing".to_string()));
}

#[tokio::test]
async fn gateway_can_invoke_with_subject_scoped_upstream_credential() {
    let runtime = McpRuntime::start(vec![fixture_config()]).await;
    let gateway = GatewayService::from_runtime_snapshots(runtime.snapshots());
    let action_id =
        LauncherActionId::from_server_tool(&McpServerId::new("fixture"), &McpToolId::new("echo"));

    let result = gateway
        .invoke_with_credential(
            &runtime,
            ToolInvocation {
                action_id,
                arguments: json!({"message": "credential-path"}),
            },
            Some(&OAuthCredential {
                subject: "user-1".to_string(),
                upstream_id: "fixture".to_string(),
                access_token: "upstream-token-not-inbound-token".to_string(),
                refresh_token: None,
                scopes: ScopeSet::parse("mcp:read").expect("scope"),
                expires_at_unix: 2000,
                refresh_failed: false,
            }),
        )
        .await
        .expect("invoke");

    assert_eq!(result.output["content"][0]["text"], "credential-path");
}

#[tokio::test]
async fn exposure_policy_blocks_direct_invocation_of_hidden_action() {
    let runtime = McpRuntime::start(vec![fixture_config()]).await;
    let gateway = GatewayService::from_runtime_snapshots_with_policy(
        runtime.snapshots(),
        &GatewayExposurePolicy::default().deny_tool(McpToolId::new("echo")),
    );
    let action_id =
        LauncherActionId::from_server_tool(&McpServerId::new("fixture"), &McpToolId::new("echo"));

    let error = gateway
        .invoke(
            &runtime,
            ToolInvocation {
                action_id,
                arguments: json!({"message": "hidden"}),
            },
        )
        .await
        .expect_err("hidden action must not route");

    assert_eq!(
        error,
        GatewayError::UnknownAction("mcp:fixture:echo".to_string())
    );
}

#[test]
fn health_summary_aggregates_runtime_status_and_catalog_counts() {
    let snapshots = vec![
        runtime_snapshot(
            "healthy",
            agent_protocol::ServerStatus::Healthy,
            vec!["echo"],
        ),
        runtime_snapshot("failed", agent_protocol::ServerStatus::Failed, Vec::new()),
    ];

    let summary = GatewayService::health_from_snapshots(&snapshots);

    assert_eq!(summary.server_count, 2);
    assert_eq!(summary.action_count, 1);
    assert_eq!(summary.status_count("healthy"), 1);
    assert_eq!(summary.status_count("failed"), 1);
}

fn runtime_snapshot(
    server_id: &str,
    status: agent_protocol::ServerStatus,
    tools: Vec<&str>,
) -> RuntimeCatalogSnapshot {
    RuntimeCatalogSnapshot {
        server_id: McpServerId::new(server_id),
        server_name: server_id.to_string(),
        status,
        tools: tools
            .into_iter()
            .map(|tool| RuntimeTool {
                id: McpToolId::new(tool),
                name: tool.to_string(),
                title: None,
                description: None,
                input_schema: json!({}),
                output_schema: None,
                annotations: None,
            })
            .collect(),
        resources: Vec::new(),
        resource_templates: Vec::new(),
        prompts: Vec::new(),
    }
}
