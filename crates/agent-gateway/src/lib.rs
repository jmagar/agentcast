pub mod catalog;
pub mod error;
pub mod exposure;
pub mod invoke;
pub mod oauth;
pub mod protected_routes;
pub mod router;

pub use catalog::{CollisionReport, GatewayCatalog, GatewaySearchDocument};
pub use error::GatewayError;
pub use exposure::GatewayExposurePolicy;
pub use invoke::GatewayService;
pub use oauth::{
    BeginAuthorization, BeginAuthorizationResult, GatewayOAuthError, GatewayOAuthService,
    HttpOAuthTokenClient, OAuthAuthorizationCodeEndpoint, OAuthDynamicClientRegistrationEndpoint,
    OAuthDynamicClientRegistrationRequest, OAuthMetadataDiscoveryEndpoint, OAuthReconciliation,
    OAuthRefreshProvider, OAuthRefreshProviderError, OAuthTokenRefreshEndpoint,
};
pub use protected_routes::{
    ProtectedRouteCollection, ProtectedRouteConfig, ProtectedRouteIndex, ProtectedRouteStatus,
    ProtectedRouteTarget, ResolvedProtectedRoute,
};
pub use router::{ActionRoute, GatewayRouter};
