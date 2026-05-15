mod cache;
mod client;
mod error;
mod mcp;
mod search;

pub use cache::{CachedRegistryServer, InMemoryRegistryCache, RegistryCache};
pub use client::{DEFAULT_MCP_REGISTRY_BASE_URL, McpRegistryClient};
pub use error::{RegistryError, RegistryResult};
pub use mcp::{
    McpEnvironmentVariable, McpRegistryHeader, McpRegistryIcon, McpRegistryPackage,
    McpRegistryRemote, McpRegistryRepository, McpRegistryResponse, McpRegistryServer,
    McpRegistryServerResponse, McpRegistryTransport, NormalizedMcpEnvVar, NormalizedMcpPackage,
    NormalizedMcpRemote, NormalizedMcpServer, NormalizedRegistryMetadata, RegistryPagination,
    RegistryProvenance,
};
pub use search::search_servers;
