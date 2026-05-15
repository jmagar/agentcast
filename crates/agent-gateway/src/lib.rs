pub mod catalog;
pub mod error;
pub mod invoke;
pub mod oauth;
pub mod protected_routes;
pub mod router;

pub use catalog::{CollisionReport, GatewayCatalog, GatewaySearchDocument};
pub use error::GatewayError;
pub use invoke::GatewayService;
pub use oauth::{
    BeginAuthorization, BeginAuthorizationResult, GatewayOAuthError, GatewayOAuthService,
};
pub use protected_routes::{
    ProtectedRouteConfig, ProtectedRouteIndex, ProtectedRouteTarget, ResolvedProtectedRoute,
};
pub use router::{ActionRoute, GatewayRouter};
