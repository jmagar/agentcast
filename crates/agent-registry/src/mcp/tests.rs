use super::*;

const MCP_REGISTRY_SERVERS_FIXTURE: &str = r#"{
  "servers": [{
    "server": {
      "name": "io.modelcontextprotocol/filesystem",
      "title": "Filesystem",
      "description": "Filesystem MCP server",
      "version": "0.6.2",
      "packages": [{
        "registryType": "npm",
        "identifier": "@modelcontextprotocol/server-filesystem",
        "version": "0.6.2",
        "runtimeHint": "npx",
        "transport": { "type": "stdio" }
      }],
      "remotes": [{
        "type": "http",
        "url": "https://filesystem.example.test/mcp"
      }],
      "repository": {
        "url": "https://github.com/modelcontextprotocol/servers"
      },
      "websiteUrl": "https://modelcontextprotocol.io"
    },
    "meta": {
      "official": {
        "isLatest": true,
        "status": "active",
        "publishedAt": "2026-05-01T00:00:00Z",
        "updatedAt": "2026-05-02T00:00:00Z"
      }
    }
  }],
  "metadata": {
    "next_cursor": "next-page"
  }
}"#;

const FLAT_REGISTRY_SERVERS_FIXTURE: &str = r#"{
  "servers": [{
    "name": "io.modelcontextprotocol/git",
    "description": "Git repository tools",
    "version": "0.6.2",
    "packages": [{
      "registry_type": "npm",
      "identifier": "@modelcontextprotocol/server-git",
      "version": "0.6.2",
      "runtime_hint": "npx",
      "transport": "stdio"
    }]
  }]
}"#;

#[test]
fn normalizes_mcp_registry_response() {
    let response: McpRegistryResponse = serde_json::from_str(MCP_REGISTRY_SERVERS_FIXTURE).unwrap();
    let servers = response.normalize().unwrap();

    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "io.modelcontextprotocol/filesystem");
    assert_eq!(servers[0].title.as_deref(), Some("Filesystem"));
    assert_eq!(servers[0].latest_version.as_deref(), Some("0.6.2"));
    assert_eq!(
        servers[0].packages[0].identifier,
        "@modelcontextprotocol/server-filesystem"
    );
    assert_eq!(servers[0].packages[0].transport.as_deref(), Some("stdio"));
    assert_eq!(servers[0].remotes[0].transport_type, "http");
    assert_eq!(servers[0].registry_metadata.is_latest, Some(true));
    assert_eq!(
        servers[0].registry_metadata.status.as_deref(),
        Some("active")
    );
    assert_eq!(
        servers[0].registry_metadata.published_at.as_deref(),
        Some("2026-05-01T00:00:00Z")
    );
    assert_eq!(servers[0].provenance.source, "mcp_registry");
}

#[test]
fn normalizes_flat_fixture_shape_for_local_tests() {
    let response: McpRegistryResponse =
        serde_json::from_str(FLAT_REGISTRY_SERVERS_FIXTURE).unwrap();
    let servers = response.normalize().unwrap();

    assert_eq!(servers[0].name, "io.modelcontextprotocol/git");
    assert_eq!(servers[0].packages[0].runtime_hint.as_deref(), Some("npx"));
    assert_eq!(servers[0].packages[0].transport.as_deref(), Some("stdio"));
}

#[test]
fn normalized_server_has_stable_cache_key() {
    let server = NormalizedMcpServer {
        name: "io.modelcontextprotocol/filesystem".into(),
        title: None,
        description: Some("Filesystem MCP server".into()),
        latest_version: Some("0.6.2".into()),
        packages: vec![],
        remotes: vec![],
        repository_url: None,
        website_url: None,
        provenance: RegistryProvenance::official_mcp(),
        registry_metadata: NormalizedRegistryMetadata::default(),
    };

    assert_eq!(server.cache_key(), "mcp:io.modelcontextprotocol/filesystem");
}

#[test]
fn rejects_blank_server_names() {
    let raw = r#"{"servers":[{"name":" ","description":"bad"}]}"#;
    let response: McpRegistryResponse = serde_json::from_str(raw).unwrap();
    let err = response.normalize().unwrap_err();

    assert_eq!(err.kind(), "registry_invalid_data");
}
