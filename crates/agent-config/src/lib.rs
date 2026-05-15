pub mod discovery;
pub mod env_merge;
pub mod error;
pub mod load;
pub mod mcp;
pub mod mcp_json;
pub mod mutation;
pub mod paths;

pub use discovery::{DiscoveredMcpServer, discover_known_mcp_configs};
pub use env_merge::{EnvMerge, EnvMergeResult, merge_env_file, validate_env_key_for_merge};
pub use error::{ConfigError, ConfigResult};
pub use load::{AgentConfig, load_from_path, load_from_str, write_to_path};
pub use mcp::{
    EnvBinding, McpConfig, McpTransport, McpUpstreamConfig, StdioUpstreamConfig,
    StreamableHttpUpstreamConfig,
};
pub use mcp_json::{parse_mcp_json, parse_mcp_json_with_options};
pub use mutation::{add_or_replace_upstream, remove_upstream};
pub use paths::{AgentPaths, PathResolution};
