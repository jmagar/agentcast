use std::collections::BTreeMap;

use crate::NormalizedMcpServer;

#[cfg(test)]
mod tests;

pub trait RegistryCache {
    fn put(&mut self, server: NormalizedMcpServer);
    fn put_fetched(&mut self, server: NormalizedMcpServer, fetched_at_unix: u64);
    fn put_many(&mut self, servers: impl IntoIterator<Item = NormalizedMcpServer>) {
        for server in servers {
            self.put(server);
        }
    }
    fn get(&self, cache_key: &str) -> Option<&NormalizedMcpServer>;
    fn get_cached(&self, cache_key: &str) -> Option<&CachedRegistryServer>;
    fn list(&self) -> Vec<NormalizedMcpServer>;
    fn list_cached(&self) -> Vec<CachedRegistryServer>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedRegistryServer {
    pub server: NormalizedMcpServer,
    pub fetched_at_unix: Option<u64>,
}

#[derive(Debug, Default)]
pub struct InMemoryRegistryCache {
    servers: BTreeMap<String, CachedRegistryServer>,
}

impl RegistryCache for InMemoryRegistryCache {
    fn put(&mut self, server: NormalizedMcpServer) {
        self.put_cached(server, None);
    }

    fn put_fetched(&mut self, server: NormalizedMcpServer, fetched_at_unix: u64) {
        self.put_cached(server, Some(fetched_at_unix));
    }

    fn get(&self, cache_key: &str) -> Option<&NormalizedMcpServer> {
        self.get_cached(cache_key).map(|cached| &cached.server)
    }

    fn get_cached(&self, cache_key: &str) -> Option<&CachedRegistryServer> {
        self.servers.get(cache_key)
    }

    fn list(&self) -> Vec<NormalizedMcpServer> {
        self.servers
            .values()
            .map(|cached| cached.server.clone())
            .collect()
    }

    fn list_cached(&self) -> Vec<CachedRegistryServer> {
        self.servers.values().cloned().collect()
    }
}

impl InMemoryRegistryCache {
    fn put_cached(&mut self, server: NormalizedMcpServer, fetched_at_unix: Option<u64>) {
        self.servers.insert(
            server.cache_key(),
            CachedRegistryServer {
                server,
                fetched_at_unix,
            },
        );
    }
}
