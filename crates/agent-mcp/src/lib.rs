pub mod client;
pub mod error;
pub mod metadata;
pub mod result;
pub mod stdio;

pub use client::{McpClient, McpClientOptions};
pub use error::{McpError, McpResult};
pub use metadata::{
    McpPromptMetadata, McpReadResourceResult, McpResourceContent, McpResourceMetadata,
    McpResourceTemplateMetadata, McpToolMetadata,
};
pub use result::{McpToolContent, McpToolResult};
pub use stdio::StdioConnection;
