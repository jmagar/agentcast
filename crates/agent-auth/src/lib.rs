pub mod protected_resource;
pub mod scope;

pub use protected_resource::{AuthChallenge, ProtectedResourceMetadata};
pub use scope::{ScopeError, ScopeSet};
