use super::*;
use crate::{NormalizedRegistryMetadata, RegistryProvenance};

fn server(name: &str, version: &str) -> NormalizedMcpServer {
    NormalizedMcpServer {
        name: name.into(),
        title: None,
        description: None,
        latest_version: Some(version.into()),
        packages: vec![],
        remotes: vec![],
        repository_url: None,
        website_url: None,
        provenance: RegistryProvenance::official_mcp(),
        registry_metadata: NormalizedRegistryMetadata::default(),
    }
}

#[test]
fn cache_round_trips_normalized_server_by_cache_key() {
    let mut cache = InMemoryRegistryCache::default();
    let server = server("io.modelcontextprotocol/filesystem", "0.6.2");
    let key = server.cache_key();

    cache.put(server);
    assert_eq!(
        cache.get(&key).unwrap().latest_version.as_deref(),
        Some("0.6.2")
    );
}

#[test]
fn cache_records_fetch_freshness_metadata() {
    let mut cache = InMemoryRegistryCache::default();
    cache.put_fetched(
        server("io.modelcontextprotocol/filesystem", "0.6.2"),
        1_779_000_000,
    );

    let cached = cache
        .get_cached("mcp:io.modelcontextprotocol/filesystem")
        .expect("cached server");

    assert_eq!(cached.fetched_at_unix, Some(1_779_000_000));
    assert_eq!(
        cache.list_cached()[0].server.name,
        "io.modelcontextprotocol/filesystem"
    );
}

#[test]
fn cache_replaces_existing_server_by_cache_key_and_lists_in_key_order() {
    let mut cache = InMemoryRegistryCache::default();
    cache.put(server("z", "1.0.0"));
    cache.put(server("a", "1.0.0"));
    cache.put(server("z", "2.0.0"));

    let servers = cache.list();
    assert_eq!(
        servers
            .iter()
            .map(|server| server.name.as_str())
            .collect::<Vec<_>>(),
        ["a", "z"]
    );
    assert_eq!(
        cache.get("mcp:z").unwrap().latest_version.as_deref(),
        Some("2.0.0")
    );
}
