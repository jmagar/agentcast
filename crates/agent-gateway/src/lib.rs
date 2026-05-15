pub mod catalog;
pub mod error;
pub mod protected_routes;
pub mod router;

pub use catalog::{CollisionReport, GatewayCatalog};
pub use error::GatewayError;
pub use protected_routes::{
    ProtectedRouteConfig, ProtectedRouteIndex, ProtectedRouteTarget, ResolvedProtectedRoute,
};
pub use router::{ActionRoute, GatewayRouter};
