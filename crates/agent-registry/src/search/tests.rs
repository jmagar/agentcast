use super::*;
use crate::{NormalizedRegistryMetadata, RegistryProvenance};

fn server(name: &str, description: &str) -> NormalizedMcpServer {
    NormalizedMcpServer {
        name: name.into(),
        title: None,
        description: Some(description.into()),
        latest_version: None,
        packages: vec![],
        remotes: vec![],
        repository_url: None,
        website_url: None,
        provenance: RegistryProvenance::official_mcp(),
        registry_metadata: NormalizedRegistryMetadata::default(),
    }
}

#[test]
fn search_servers_matches_name_and_description() {
    let servers = vec![
        server(
            "io.modelcontextprotocol/filesystem",
            "Filesystem MCP server",
        ),
        server("io.modelcontextprotocol/git", "Git repository tools"),
    ];

    let results = search_servers(&servers, "file", 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "io.modelcontextprotocol/filesystem");
}

#[test]
fn search_servers_honors_limit() {
    let servers = vec![server("a", "match"), server("b", "match")];
    let results = search_servers(&servers, "match", 1);
    assert_eq!(results.len(), 1);
}

#[test]
fn empty_query_returns_first_page_without_ranking() {
    let servers = vec![server("a", "one"), server("b", "two")];
    let results = search_servers(&servers, " ", 10);
    assert_eq!(
        results
            .iter()
            .map(|server| server.name.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
}
