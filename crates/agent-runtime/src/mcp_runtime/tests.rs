use super::*;
use agent_protocol::{McpServerConfig, McpTransportConfig};
use serde_json::json;

fn fixture_config(enabled: bool) -> McpServerConfig {
    fixture_config_with_id("fixture", enabled)
}

fn fixture_config_with_id(id: &str, enabled: bool) -> McpServerConfig {
    let server = format!(
        "{}/../agent-mcp/src/fixtures/mcp_echo_server.js",
        env!("CARGO_MANIFEST_DIR")
    );
    McpServerConfig {
        id: McpServerId::new(id),
        name: format!("Fixture {id}"),
        enabled,
        transport: McpTransportConfig::Stdio {
            command: "node".to_string(),
            args: vec![server],
            env: BTreeMap::new(),
        },
        env_keys: Vec::new(),
    }
}

#[tokio::test]
async fn runtime_discovers_enabled_stdio_upstream_catalog() {
    let runtime = McpRuntime::start(vec![fixture_config(true)]).await;
    let snapshot = runtime.snapshots().remove(0);

    assert_eq!(snapshot.status, ServerStatus::Healthy);
    assert_eq!(snapshot.tools[0].name, "echo");
    assert_eq!(snapshot.resources[0].name, "fixture");
    assert_eq!(snapshot.resource_templates[0].name, "fixture-template");
    assert_eq!(snapshot.prompts[0].name, "summarize");
}

#[tokio::test]
async fn runtime_start_preserves_deterministic_snapshot_order() {
    let runtime = McpRuntime::start(vec![
        fixture_config_with_id("z-fixture", false),
        fixture_config_with_id("a-fixture", false),
        fixture_config_with_id("m-fixture", false),
    ])
    .await;
    let server_ids = runtime
        .snapshots()
        .into_iter()
        .map(|snapshot| snapshot.server_id.to_string())
        .collect::<Vec<_>>();

    assert_eq!(server_ids, vec!["a-fixture", "m-fixture", "z-fixture"]);
}

#[tokio::test]
async fn runtime_invokes_tool_through_mcp_client() {
    let runtime = McpRuntime::start(vec![fixture_config(true)]).await;

    let response = runtime
        .call_tool(ToolCallRequest {
            server_id: McpServerId::new("fixture"),
            tool_id: McpToolId::new("echo"),
            arguments: json!({"message": "hello"}),
            auth: None,
        })
        .await
        .expect("tool call");

    assert_eq!(response.output["content"][0]["text"], "hello");
}

#[tokio::test]
async fn runtime_accepts_operation_timeout_options() {
    let runtime = McpRuntime::start_with_options(
        vec![fixture_config(true)],
        RuntimeOptions {
            operation_timeout: std::time::Duration::from_secs(5),
            ..RuntimeOptions::default()
        },
    )
    .await;
    let snapshot = runtime.snapshots().remove(0);

    assert_eq!(snapshot.status, ServerStatus::Healthy);
}

#[tokio::test]
async fn runtime_rejects_oversized_tool_responses() {
    let runtime = McpRuntime::start_with_options(
        vec![fixture_config(true)],
        RuntimeOptions {
            max_response_bytes: 8,
            ..RuntimeOptions::default()
        },
    )
    .await;

    let error = runtime
        .call_tool(ToolCallRequest {
            server_id: McpServerId::new("fixture"),
            tool_id: McpToolId::new("echo"),
            arguments: json!({"message": "hello"}),
            auth: None,
        })
        .await
        .expect_err("response should exceed cap");

    assert!(matches!(error, RuntimeError::ResponseTooLarge { .. }));
}

#[tokio::test]
async fn runtime_opens_circuit_after_repeated_operation_failures_and_reprobe_resets() {
    let mut runtime = McpRuntime::start_with_options(
        vec![fixture_config(true)],
        RuntimeOptions {
            circuit_breaker_failure_threshold: 1,
            max_response_bytes: 8,
            ..RuntimeOptions::default()
        },
    )
    .await;
    let server_id = McpServerId::new("fixture");

    let error = runtime
        .call_tool(ToolCallRequest {
            server_id: server_id.clone(),
            tool_id: McpToolId::new("echo"),
            arguments: json!({"message": "hello"}),
            auth: None,
        })
        .await
        .expect_err("oversized response should fail");
    assert!(matches!(error, RuntimeError::ResponseTooLarge { .. }));
    assert!(runtime.circuit_open(&server_id));

    let error = runtime
        .call_tool(ToolCallRequest {
            server_id: server_id.clone(),
            tool_id: McpToolId::new("echo"),
            arguments: json!({"message": "hello"}),
            auth: None,
        })
        .await
        .expect_err("open circuit should reject");
    assert_eq!(
        error,
        RuntimeError::CircuitOpen("Fixture fixture".to_string())
    );

    runtime.reprobe(&server_id).await.expect("reprobe");
    assert!(!runtime.circuit_open(&server_id));
}

#[tokio::test]
async fn runtime_shutdown_cancels_retained_lifecycle_tokens() {
    let runtime = McpRuntime::start(vec![fixture_config(true)]).await;
    let upstream = runtime
        .upstreams
        .get(&McpServerId::new("fixture"))
        .expect("fixture upstream");
    let token = upstream.lifecycle_token.clone();

    runtime.shutdown().await;

    assert!(token.is_cancelled());
}

#[test]
fn catalog_discovery_keeps_successful_kinds_when_one_kind_fails() {
    let mut failures = Vec::new();

    let tools = collect_catalog_result::<RuntimeTool>(
        "tools",
        Err(RuntimeError::Mcp("tools unavailable".to_string())),
        &mut failures,
    );
    let prompts = collect_catalog_result(
        "prompts",
        Ok(vec![RuntimePrompt {
            name: "summarize".to_string(),
            title: None,
            description: None,
            arguments: Value::Null,
        }]),
        &mut failures,
    );

    assert!(tools.is_empty());
    assert_eq!(prompts.len(), 1);
    assert_eq!(
        failures,
        vec!["tools: MCP runtime error: tools unavailable"]
    );
}

#[tokio::test]
async fn disabled_upstream_is_not_started() {
    let runtime = McpRuntime::start(vec![fixture_config(false)]).await;
    let snapshot = runtime.snapshots().remove(0);

    assert_eq!(snapshot.status, ServerStatus::Disabled);
    assert!(
        runtime
            .call_tool(ToolCallRequest {
                server_id: McpServerId::new("fixture"),
                tool_id: McpToolId::new("echo"),
                arguments: json!({}),
                auth: None,
            })
            .await
            .is_err()
    );
}

#[tokio::test]
async fn stdio_runtime_ignores_subject_bearer_token() {
    let runtime = McpRuntime::start(vec![fixture_config(true)]).await;

    let response = runtime
        .call_tool_with_bearer(
            ToolCallRequest {
                server_id: McpServerId::new("fixture"),
                tool_id: McpToolId::new("echo"),
                arguments: json!({"message": "hello"}),
                auth: None,
            },
            "upstream-subject-token",
        )
        .await
        .expect("tool call");

    assert_eq!(response.output["content"][0]["text"], "hello");
    assert!(!runtime.would_use_ephemeral_http_auth(
        &McpServerId::new("fixture"),
        Some(&crate::RuntimeRequestAuth::bearer("upstream-subject-token"))
    ));
}

#[tokio::test]
async fn streamable_http_runtime_would_use_subject_bearer_token() {
    let runtime = McpRuntime::start(vec![streamable_http_config(false)]).await;

    assert!(runtime.would_use_ephemeral_http_auth(
        &McpServerId::new("http-fixture"),
        Some(&crate::RuntimeRequestAuth::bearer("upstream-subject-token"))
    ));
}

fn streamable_http_config(enabled: bool) -> McpServerConfig {
    McpServerConfig {
        id: McpServerId::new("http-fixture"),
        name: "HTTP Fixture".to_string(),
        enabled,
        transport: McpTransportConfig::StreamableHttp {
            url: "https://mcp.example.test/mcp".to_string(),
            bearer_token_env: None,
        },
        env_keys: Vec::new(),
    }
}
