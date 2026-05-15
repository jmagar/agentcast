pub mod discovery;
pub mod error;
pub mod mcp_json;

pub use discovery::{DiscoveredMcpServer, discover_known_mcp_configs};
pub use error::ConfigError;
pub use mcp_json::{parse_mcp_json, parse_mcp_json_with_options};
