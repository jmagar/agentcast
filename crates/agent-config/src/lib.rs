pub mod discovery;
pub mod env_file;
pub mod env_merge;
pub mod error;
pub mod load;
pub mod mcp;
pub mod mcp_json;
pub mod mutation;
pub mod operations;
pub mod paths;
pub mod value_path;

pub use discovery::{DiscoveredMcpServer, discover_known_mcp_configs};
pub use env_file::{EnvWriteResult, get_env_value, list_env_keys, set_env_value, unset_env_value};
pub use env_merge::{EnvMerge, EnvMergeResult, merge_env_file, validate_env_key_for_merge};
pub use error::{ConfigError, ConfigResult};
pub use load::{AgentConfig, load_from_path, load_from_str, write_to_path};
pub use mcp::{
    EnvBinding, McpConfig, McpTransport, McpUpstreamConfig, StdioUpstreamConfig,
    StreamableHttpUpstreamConfig,
};
pub use mcp_json::{parse_mcp_json, parse_mcp_json_with_options};
pub use mutation::{add_or_replace_upstream, remove_upstream};
pub use operations::{
    ConfigPathsRecord, ConfigUnsetRecord, ConfigValueRecord, ConfigWriteRecord, EnvListRecord,
    EnvUnsetRecord, EnvValueRecord, EnvWriteRecord, config_get, config_list, config_paths,
    config_set, config_unset, config_validate, default_paths, env_get, env_list, env_set,
    env_unset,
};
pub use paths::{AgentPaths, PathResolution};
pub use value_path::{get_value, set_value, unset_value, validate_as_agent_config};
