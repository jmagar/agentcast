use super::*;
use crate::cli::Command;

#[test]
fn protected_route_index_requires_complete_route_flags() {
    let args = args_without_config();

    assert!(protected_route_index(&args).is_err());
}

#[test]
fn protected_route_index_builds_generic_upstream_route() {
    let mut args = args_without_config();
    args.protected_mcp_path = Some("/syslog".to_string());
    args.protected_mcp_resource = Some("https://mcp.example.test/syslog".to_string());

    let routes = protected_route_index(&args)
        .expect("route index")
        .expect("protected routes");
    assert!(routes.resolve("mcp.example.test", "/syslog").is_some());
}

fn args_without_config() -> crate::cli::Args {
    crate::cli::Args {
        command: None::<Command>,
        listen: "127.0.0.1:8787".parse().expect("listen"),
        mcp_config: None,
        discover_mcp: false,
        enable_imported: false,
        mcp_stdio: false,
        protected_mcp_host: Some("mcp.example.test".to_string()),
        protected_mcp_path: None,
        protected_mcp_server: Some("fixture".to_string()),
        protected_mcp_resource: None,
        protected_mcp_auth_servers: vec!["https://auth.example.test".to_string()],
        protected_mcp_scopes: "mcp:read".to_string(),
        oauth_store: None,
        oauth_key_hex: None,
    }
}
