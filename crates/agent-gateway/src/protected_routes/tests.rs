use super::*;
use agent_auth::ScopeSet;
use agent_protocol::McpServerId;

fn route(name: &str, host: &str, path: &str) -> ProtectedRouteConfig {
    ProtectedRouteConfig {
        name: name.to_string(),
        enabled: true,
        public_host: host.to_string(),
        public_path: path.to_string(),
        resource_uri: format!("https://{host}{path}"),
        authorization_servers: vec!["https://auth.example.test".to_string()],
        required_scopes: ScopeSet::parse("mcp:read").expect("parse scopes"),
        target: ProtectedRouteTarget::UpstreamMcp {
            server_id: McpServerId::new("local"),
        },
    }
}

#[test]
fn resolves_by_normalized_host_and_first_path_segment() {
    let index =
        ProtectedRouteIndex::from_routes(vec![route("syslog", "MCP.Example.Test.", "/syslog")])
            .expect("build index");

    let resolved = index
        .resolve("mcp.example.test:443", "/syslog/tools/list")
        .expect("route resolved");

    assert_eq!(resolved.name, "syslog");
}

#[test]
fn rejects_duplicate_enabled_host_and_path() {
    let error = ProtectedRouteIndex::from_routes(vec![
        route("one", "mcp.example.test", "/syslog"),
        route("two", "MCP.EXAMPLE.TEST.", "/syslog/tools"),
    ])
    .expect_err("duplicate route");

    assert_eq!(
        error.to_string(),
        "duplicate protected route for host `mcp.example.test` and path segment `/syslog`"
    );
}

#[test]
fn rejects_reserved_and_ambiguous_public_paths() {
    for path in [
        "/",
        "/.well-known",
        "/v1",
        "/syslog//tools",
        "/syslog/..",
        "/syslog/%2fsecret",
    ] {
        let error = ProtectedRouteIndex::from_routes(vec![route("bad", "mcp.example.test", path)])
            .expect_err("invalid path");
        assert!(error.to_string().contains("invalid public path"));
    }
}

#[test]
fn exact_metadata_path_resolves_route() {
    let index =
        ProtectedRouteIndex::from_routes(vec![route("syslog", "mcp.example.test", "/syslog")])
            .expect("build index");

    let resolved = index
        .resolve_metadata(
            "mcp.example.test",
            "/.well-known/oauth-protected-resource/syslog",
        )
        .expect("metadata route resolved");

    assert_eq!(resolved.name, "syslog");
}

#[test]
fn projects_protected_resource_metadata() {
    let index =
        ProtectedRouteIndex::from_routes(vec![route("syslog", "mcp.example.test", "/syslog")])
            .expect("build index");
    let resolved = index
        .resolve_metadata(
            "mcp.example.test",
            "/.well-known/oauth-protected-resource/syslog",
        )
        .expect("metadata route resolved");

    let metadata = resolved.protected_resource_metadata();

    assert_eq!(metadata.resource, "https://mcp.example.test/syslog");
    assert_eq!(
        metadata.authorization_servers[0],
        "https://auth.example.test"
    );
    assert_eq!(metadata.scopes_supported.as_slice(), &["mcp:read"]);
}
