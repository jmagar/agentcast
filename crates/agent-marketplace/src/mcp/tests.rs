use super::*;
use agent_config::AgentConfig;
use agent_registry::{
    NormalizedMcpEnvVar, NormalizedMcpPackage, NormalizedMcpRemote, NormalizedMcpServer,
    NormalizedRegistryMetadata, RegistryProvenance,
};

fn server(package: NormalizedMcpPackage) -> NormalizedMcpServer {
    NormalizedMcpServer {
        name: "io.modelcontextprotocol/filesystem".into(),
        title: None,
        description: Some("Filesystem MCP server".into()),
        latest_version: Some("0.6.2".into()),
        packages: vec![package],
        remotes: vec![],
        repository_url: None,
        website_url: None,
        provenance: RegistryProvenance::official_mcp(),
        registry_metadata: NormalizedRegistryMetadata::default(),
    }
}

fn remote_server(package: NormalizedMcpPackage) -> NormalizedMcpServer {
    let mut server = server(package);
    server.remotes = vec![NormalizedMcpRemote {
        transport_type: "http".into(),
        url: Some("https://remote.example.test/mcp".into()),
    }];
    server
}

fn package(runtime_hint: Option<&str>) -> NormalizedMcpPackage {
    NormalizedMcpPackage {
        registry_type: "npm".into(),
        identifier: "@modelcontextprotocol/server-filesystem".into(),
        version: Some("0.6.2".into()),
        runtime_hint: runtime_hint.map(str::to_string),
        transport: Some("stdio".into()),
        runtime_arguments: vec![serde_json::json!("-y")],
        package_arguments: vec![serde_json::json!("/tmp")],
        environment_variables: vec![NormalizedMcpEnvVar {
            name: "FILESYSTEM_TOKEN".into(),
            description: Some("Access token".into()),
            is_required: true,
            is_secret: true,
            default: None,
        }],
    }
}

fn remote_package() -> NormalizedMcpPackage {
    let mut package = package(None);
    package.transport = Some("http".into());
    package.environment_variables = vec![NormalizedMcpEnvVar {
        name: "REMOTE_TOKEN".into(),
        description: Some("Bearer token".into()),
        is_required: true,
        is_secret: true,
        default: None,
    }];
    package
}

#[test]
fn creates_stdio_install_plan_from_npm_registry_package() {
    let server = server(package(Some("npx")));

    let plan = plan_mcp_server_install(&server).expect("plan created");
    assert_eq!(plan.package, "io.modelcontextprotocol/filesystem");

    let upstream = plan.steps.last().unwrap();
    assert_eq!(upstream.target, "mcp.upstreams.filesystem");
    assert_eq!(upstream.preview["command"], "npx");
    assert_eq!(
        upstream.apply,
        InstallStepApply::AddMcpUpstream {
            id: "filesystem".into(),
            transport: InstallMcpUpstreamTransport::Stdio {
                command: "npx".into(),
                args: vec![
                    "-y".into(),
                    "@modelcontextprotocol/server-filesystem".into(),
                    "/tmp".into()
                ],
            },
        }
    );
    assert_eq!(
        upstream.preview["args"],
        serde_json::json!(["-y", "@modelcontextprotocol/server-filesystem", "/tmp"])
    );
    assert_eq!(plan.steps[1].target, "env.FILESYSTEM_TOKEN");
}

#[test]
fn rejects_missing_runtime_hint() {
    let err = plan_mcp_server_install(&server(package(None))).unwrap_err();
    assert_eq!(err.kind(), "invalid_install_target");
}

#[test]
fn rejects_dangerous_package_args() {
    let mut package = package(Some("npx"));
    package.package_arguments = vec![serde_json::json!("--inspect=0.0.0.0:9229")];

    let err = plan_mcp_server_install(&server(package)).unwrap_err();
    assert_eq!(err.kind(), "unsafe_install_parameter");
}

#[test]
fn rejects_mcpb_until_integrity_apply_exists() {
    let mut package = package(Some("npx"));
    package.registry_type = "mcpb".into();

    let err = plan_mcp_server_install(&server(package)).unwrap_err();
    assert_eq!(err.kind(), "invalid_install_target");
}

#[test]
fn applies_install_plan_through_agent_config_mutation() {
    let plan = plan_mcp_server_install(&server(package(Some("npx")))).unwrap();
    let mut config = AgentConfig::default();

    let result = apply_install_plan_to_config(&mut config, &plan).unwrap();

    assert_eq!(result.added_or_replaced_upstreams, ["filesystem"]);
    assert_eq!(result.env_keys_required, ["FILESYSTEM_TOKEN"]);
    let upstream = config.mcp.upstreams.get("filesystem").unwrap();
    let agent_config::McpTransport::Stdio(stdio) = &upstream.transport else {
        panic!("expected stdio");
    };
    assert_eq!(stdio.command, "npx");
    assert_eq!(
        stdio.args,
        ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    );
}

#[test]
fn applies_install_plan_from_typed_action_not_preview_fields() {
    let mut plan = plan_mcp_server_install(&server(package(Some("npx")))).unwrap();
    let upstream_step = plan.steps.last_mut().unwrap();
    upstream_step.preview = serde_json::json!({
        "id": "wrong",
        "transport": "stdio",
        "command": "wrong-command",
        "args": ["wrong-package"],
    });
    let env_step = &mut plan.steps[1];
    env_step.preview = serde_json::json!({ "name": "WRONG_TOKEN" });
    let mut config = AgentConfig::default();

    let result = apply_install_plan_to_config(&mut config, &plan).unwrap();

    assert_eq!(result.added_or_replaced_upstreams, ["filesystem"]);
    assert_eq!(result.env_keys_required, ["FILESYSTEM_TOKEN"]);
    let upstream = config.mcp.upstreams.get("filesystem").unwrap();
    let agent_config::McpTransport::Stdio(stdio) = &upstream.transport else {
        panic!("expected stdio");
    };
    assert_eq!(stdio.command, "npx");
    assert_eq!(
        stdio.args,
        ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    );
    assert!(!config.mcp.upstreams.contains_key("wrong"));
}

#[test]
fn creates_remote_http_install_plan_from_registry_remote() {
    let plan = plan_mcp_server_install(&remote_server(remote_package())).expect("plan created");

    let upstream = plan.steps.last().unwrap();
    assert_eq!(upstream.preview["transport"], "streamable_http");
    assert_eq!(upstream.preview["url"], "https://remote.example.test/mcp");
    assert_eq!(upstream.preview["bearer_token_env"], "REMOTE_TOKEN");
    assert_eq!(
        upstream.apply,
        InstallStepApply::AddMcpUpstream {
            id: "filesystem".into(),
            transport: InstallMcpUpstreamTransport::StreamableHttp {
                url: "https://remote.example.test/mcp".into(),
                bearer_token_env: Some("REMOTE_TOKEN".into()),
            },
        }
    );
}

#[test]
fn applies_remote_http_install_plan_through_agent_config_mutation() {
    let plan = plan_mcp_server_install(&remote_server(remote_package())).unwrap();
    let mut config = AgentConfig::default();

    let result = apply_install_plan_to_config(&mut config, &plan).unwrap();

    assert_eq!(result.added_or_replaced_upstreams, ["filesystem"]);
    let upstream = config.mcp.upstreams.get("filesystem").unwrap();
    let agent_config::McpTransport::StreamableHttp(http) = &upstream.transport else {
        panic!("expected streamable http");
    };
    assert_eq!(http.url, "https://remote.example.test/mcp");
    assert_eq!(http.bearer_token_env.as_deref(), Some("REMOTE_TOKEN"));
}

#[test]
fn resolves_install_env_values_defaults_and_missing_required() {
    let mut package = package(Some("npx"));
    package.environment_variables.push(NormalizedMcpEnvVar {
        name: "OPTIONAL_URL".into(),
        description: None,
        is_required: false,
        is_secret: false,
        default: Some("https://example.test".into()),
    });
    let server = server(package);
    let supplied = BTreeMap::from([("FILESYSTEM_TOKEN".to_string(), "secret".to_string())]);

    let resolution = resolve_install_env(&server, &supplied).expect("resolution");

    assert_eq!(resolution.values["FILESYSTEM_TOKEN"], REDACTED_ENV_VALUE);
    assert_eq!(resolution.values["OPTIONAL_URL"], "https://example.test");
    assert!(resolution.missing_required.is_empty());
    assert_eq!(
        install_env_merge(&resolution).unwrap(),
        EnvMerge::new([
            ("FILESYSTEM_TOKEN", "secret"),
            ("OPTIONAL_URL", "https://example.test"),
        ])
    );
}

#[test]
fn resolves_install_env_against_selected_package_only() {
    let mut stdio = package(Some("npx"));
    stdio.environment_variables = vec![NormalizedMcpEnvVar {
        name: "SELECTED_TOKEN".into(),
        description: None,
        is_required: true,
        is_secret: true,
        default: None,
    }];
    let mut remote = remote_package();
    remote.environment_variables = vec![NormalizedMcpEnvVar {
        name: "UNSELECTED_TOKEN".into(),
        description: None,
        is_required: false,
        is_secret: true,
        default: None,
    }];
    let mut server = server(stdio);
    server.packages.push(remote);
    let supplied = BTreeMap::from([
        ("SELECTED_TOKEN".to_string(), "selected".to_string()),
        ("UNSELECTED_TOKEN".to_string(), "unselected".to_string()),
    ]);

    let err = resolve_install_env(&server, &supplied).unwrap_err();

    assert_eq!(err.kind(), "invalid_install_target");
    assert!(
        err.to_string()
            .contains("env value `UNSELECTED_TOKEN` is not declared")
    );
}

#[test]
fn reports_missing_required_env_values() {
    let resolution =
        resolve_install_env(&server(package(Some("npx"))), &BTreeMap::new()).expect("resolution");

    assert_eq!(resolution.missing_required, ["FILESYSTEM_TOKEN"]);
    assert_eq!(
        install_env_merge(&resolution).unwrap_err().kind(),
        "invalid_install_target"
    );
}
