pub mod oauth;
pub mod protected_mcp;

pub use oauth::{OAuthApiStatus, OAuthStatusResponse};
pub use protected_mcp::{
    ProtectedMcpRequest, ProtectedMcpResponse, ProtectedMcpRouteApi, ResponseStatus,
};
