use super::*;
use agent_auth::OAuthClientRegistration;
use clap::Parser;
use std::path::PathBuf;

#[test]
fn parses_gateway_cli_subcommands() {
    let args = Args::try_parse_from([
        "agentcast",
        "gateway",
        "call",
        "--mcp-config",
        "mcp.json",
        "--enable-imported",
        "mcp:fixture:echo",
        "--arguments",
        r#"{"message":"hi"}"#,
    ])
    .expect("parse");

    let Some(Command::Gateway { command }) = args.command else {
        panic!("gateway call command");
    };
    let GatewayCommand::Call {
        config,
        action_id,
        arguments,
    } = *command
    else {
        panic!("gateway call command");
    };

    assert_eq!(config.mcp_config, Some(PathBuf::from("mcp.json")));
    assert!(config.enable_imported);
    assert_eq!(action_id, "mcp:fixture:echo");
    assert_eq!(arguments, r#"{"message":"hi"}"#);
}

#[test]
fn parses_protected_mcp_server_bearer_token_flag() {
    let args = Args::try_parse_from([
        "agentcast",
        "--protected-mcp-host",
        "mcp.example.test",
        "--protected-mcp-path",
        "/syslog",
        "--protected-mcp-server",
        "syslog",
        "--protected-mcp-auth-server",
        "https://auth.example.test",
        "--protected-mcp-bearer-token",
        "secret-token",
    ])
    .expect("parse");

    assert_eq!(
        args.protected_mcp_bearer_token,
        Some("secret-token".to_string())
    );
}

#[test]
fn parses_protected_route_cli_subcommands() {
    let args = Args::try_parse_from([
        "agentcast",
        "gateway",
        "protected-route",
        "metadata",
        "--host",
        "mcp.example.test",
        "--path",
        "/syslog",
        "--server",
        "syslog",
        "--auth-server",
        "https://auth.example.test",
    ])
    .expect("parse");

    let Some(Command::Gateway { command }) = args.command else {
        panic!("protected route metadata command");
    };
    let GatewayCommand::ProtectedRoute {
        command: ProtectedRouteCommand::Metadata(route),
    } = *command
    else {
        panic!("protected route metadata command");
    };

    assert_eq!(route.host, "mcp.example.test");
    assert_eq!(route.path, "/syslog");
    assert_eq!(route.server, "syslog");
    assert_eq!(route.auth_servers, vec!["https://auth.example.test"]);
}

#[test]
fn parses_oauth_cli_subcommands_without_exposing_secret_result() {
    let args = Args::try_parse_from([
        "agentcast",
        "gateway",
        "oauth",
        "register",
        "--registration",
        r#"{"subject":"user-1","upstream_id":"github","client_id":"client-1","client_secret":"secret","client_id_issued_at_unix":1,"client_secret_expires_at_unix":2}"#,
    ])
    .expect("parse");

    let Some(Command::Gateway { command }) = args.command else {
        panic!("oauth register command");
    };
    let GatewayCommand::OAuth {
        command: OAuthCommand::Register { registration, .. },
    } = *command
    else {
        panic!("oauth register command");
    };

    let registration = parse_json::<OAuthClientRegistration>(&registration).expect("json");
    let output = OAuthClientRegistrationOutput::from(registration);
    assert_eq!(output.client_id, "client-1");
    assert!(output.has_client_secret);
}

#[test]
fn parses_registry_search_with_offline_response() {
    let args = Args::try_parse_from([
        "agentcast",
        "registry",
        "search",
        "filesystem",
        "--limit",
        "5",
        "--registry-response",
        r#"{"servers":[]}"#,
    ])
    .expect("parse");

    let Some(Command::Registry {
        command:
            RegistryCommand::Search {
                q,
                limit,
                registry_response,
            },
    }) = args.command
    else {
        panic!("registry search command");
    };

    assert_eq!(q, "filesystem");
    assert_eq!(limit, 5);
    assert_eq!(registry_response.as_deref(), Some(r#"{"servers":[]}"#));
}

#[test]
fn parses_marketplace_plan_and_apply_commands() {
    let server_json = normalized_server_json();
    let args = Args::try_parse_from([
        "agentcast",
        "marketplace",
        "plan-mcp",
        "--server-json",
        &server_json,
    ])
    .expect("parse");

    assert!(matches!(
        args.command,
        Some(Command::Marketplace {
            command: MarketplaceCommand::PlanMcp { .. }
        })
    ));

    let args = Args::try_parse_from([
        "agentcast",
        "marketplace",
        "apply-mcp",
        "--server-json",
        &server_json,
        "--config",
        "agentcast.toml",
        "--write",
    ])
    .expect("parse");

    let Some(Command::Marketplace {
        command: MarketplaceCommand::ApplyMcp { config, write, .. },
    }) = args.command
    else {
        panic!("marketplace apply command");
    };
    assert_eq!(config, PathBuf::from("agentcast.toml"));
    assert!(write);
}

fn normalized_server_json() -> String {
    serde_json::json!({
        "name": "io.modelcontextprotocol/filesystem",
        "title": null,
        "description": "Filesystem MCP server",
        "latest_version": "0.6.2",
        "packages": [{
            "registry_type": "npm",
            "identifier": "@modelcontextprotocol/server-filesystem",
            "version": "0.6.2",
            "runtime_hint": "npx",
            "transport": "stdio",
            "runtime_arguments": ["-y"],
            "package_arguments": ["/tmp"],
            "environment_variables": []
        }],
        "remotes": [],
        "repository_url": null,
        "website_url": null,
        "provenance": {
            "source": "mcp_registry",
            "source_url": "https://registry.modelcontextprotocol.io"
        },
        "registry_metadata": {}
    })
    .to_string()
}
