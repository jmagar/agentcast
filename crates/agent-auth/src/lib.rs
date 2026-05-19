pub mod bearer;
pub mod oauth;
pub mod protected_resource;
pub mod scope;

#[cfg(any(test, feature = "fixture-auth"))]
pub use bearer::FixtureBearerTokenVerifier;
pub use bearer::{
    AuthDecision, AuthorizedSubject, BearerClaims, BearerError, BearerTokenVerifier, Jwk, Jwks,
    JwtBearerTokenVerifier, StaticBearerTokenVerifier,
};
pub use oauth::{
    OAuthCallback, OAuthClientRegistration, OAuthCredential, OAuthError, OAuthProviderMetadata,
    OAuthRefreshRequest, OAuthRefreshResult, OAuthStatus, PendingOAuthState,
    validate_oauth_issuer_url,
};
pub use protected_resource::{AuthChallenge, ProtectedResourceMetadata};
pub use scope::{ScopeError, ScopeSet};
