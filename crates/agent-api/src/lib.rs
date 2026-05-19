pub mod gateway;
pub mod http;
pub mod oauth;
pub mod protected_mcp;

pub use gateway::{
    GatewayApi, GatewayApiAction, GatewayApiPrompt, GatewayApiResource, GatewayApiSearchResult,
    GatewayApiServer, GatewayApiStatus,
};
pub use http::{
    ApiErrorBody, ApiErrorEnvelope, CallActionRequest, MarketplaceMcpApplyRequest,
    MarketplaceMcpApplyResponse, MarketplaceMcpPlanRequest, OAuthAuthorizeRequest,
    OAuthAuthorizeResponse, OAuthCallbackRequest, OAuthClientRegistrationResponse,
    OAuthProbeRequest, OAuthProbeResponse, OAuthRefreshHttpRequest, OAuthRegisterHttpRequest,
    ReadResourceRequest, gateway_router, marketplace_router, oauth_router, oauth_router_with_store,
    protected_mcp_router, protected_mcp_router_with_oauth_store,
    protected_mcp_router_with_oauth_store_and_verifier, protected_mcp_router_with_verifier,
    protected_route_admin_router, registry_router, registry_router_from_cache,
};
pub use oauth::{OAuthApiStatus, OAuthStatusResponse};
pub use protected_mcp::{
    ProtectedMcpJsonRpcRequest, ProtectedMcpJsonRpcResponse, ProtectedMcpRequest,
    ProtectedMcpResponse, ProtectedMcpRouteApi, ResponseStatus,
};
