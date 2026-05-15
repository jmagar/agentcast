use agent_auth::{
    OAuthCallback, OAuthCredential, OAuthError, OAuthProviderMetadata, OAuthStatus,
    PendingOAuthState, ScopeSet, validate_oauth_issuer_url,
};
use agent_store::{OAuthStore, StoreError};
use thiserror::Error;
use url::Url;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct GatewayOAuthService<S> {
    store: S,
}

impl<S> GatewayOAuthService<S>
where
    S: OAuthStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &S {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut S {
        &mut self.store
    }

    pub fn probe_metadata(
        &self,
        issuer_url: &str,
        metadata: &OAuthProviderMetadata,
    ) -> Result<OAuthStatus, GatewayOAuthError> {
        let issuer = validate_oauth_issuer_url(issuer_url)?;
        if metadata.issuer != issuer.as_str().trim_end_matches('/') {
            return Ok(OAuthStatus::DiscoveryFailed);
        }
        if !metadata.supports_pkce_s256() {
            return Ok(OAuthStatus::UnsupportedProvider);
        }
        validate_oauth_issuer_url(&metadata.authorization_endpoint)?;
        validate_oauth_issuer_url(&metadata.token_endpoint)?;
        Ok(OAuthStatus::Disconnected)
    }

    pub fn begin_authorization(
        &mut self,
        request: BeginAuthorization,
        metadata: &OAuthProviderMetadata,
    ) -> Result<BeginAuthorizationResult, GatewayOAuthError> {
        if self.probe_metadata(&request.issuer_url, metadata)? != OAuthStatus::Disconnected {
            return Err(GatewayOAuthError::UnsupportedProvider);
        }

        let scope = metadata.selected_scope(
            request.challenge_scopes.as_ref(),
            request.protected_resource_scopes.as_ref(),
        );
        let pending = PendingOAuthState {
            state: request.state.clone(),
            subject: request.subject.clone(),
            upstream_id: request.upstream_id.clone(),
            expires_at_unix: request.expires_at_unix,
        };
        self.store.put_pending_state(pending)?;

        let mut url = Url::parse(&metadata.authorization_endpoint)
            .map_err(|_| GatewayOAuthError::UnsafeMetadata)?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("response_type", "code");
            query.append_pair("client_id", &request.client_id);
            query.append_pair("redirect_uri", &request.redirect_uri);
            query.append_pair("state", &request.state);
            query.append_pair("resource", &request.resource_uri);
            query.append_pair("code_challenge_method", "S256");
            query.append_pair("code_challenge", &request.code_challenge);
            if let Some(scope) = &scope {
                query.append_pair("scope", &scope.as_header_value());
            }
        }

        Ok(BeginAuthorizationResult {
            authorization_url: url.to_string(),
            selected_scope: scope,
        })
    }

    pub fn complete_callback(
        &mut self,
        callback: OAuthCallback,
        credential: OAuthCredential,
        now_unix: u64,
    ) -> Result<(), GatewayOAuthError> {
        let pending = self
            .store
            .consume_pending_state(&callback.state)?
            .ok_or(OAuthError::StateNotFound)?;
        pending.validate_callback(&callback, now_unix)?;
        if pending.upstream_id != credential.upstream_id || pending.subject != credential.subject {
            return Err(GatewayOAuthError::CallbackCredentialMismatch);
        }
        self.store.put_credential(credential)?;
        Ok(())
    }

    pub fn status(
        &self,
        subject: &str,
        upstream_id: &str,
        now_unix: u64,
    ) -> Result<OAuthStatus, GatewayOAuthError> {
        let Some(credential) = self.store.credential(subject, upstream_id)? else {
            return Ok(OAuthStatus::Disconnected);
        };
        Ok(credential.status(now_unix, 300))
    }

    pub fn clear(&mut self, subject: &str, upstream_id: &str) -> Result<(), GatewayOAuthError> {
        self.store.clear_subject_upstream(subject, upstream_id)?;
        Ok(())
    }

    pub fn credential_for_runtime(
        &self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<Option<OAuthCredential>, GatewayOAuthError> {
        self.store
            .credential(subject, upstream_id)
            .map_err(Into::into)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BeginAuthorization {
    pub issuer_url: String,
    pub subject: String,
    pub upstream_id: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub resource_uri: String,
    pub state: String,
    pub code_challenge: String,
    pub expires_at_unix: u64,
    pub challenge_scopes: Option<ScopeSet>,
    pub protected_resource_scopes: Option<ScopeSet>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BeginAuthorizationResult {
    pub authorization_url: String,
    pub selected_scope: Option<ScopeSet>,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum GatewayOAuthError {
    #[error(transparent)]
    Auth(#[from] OAuthError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("OAuth provider is unsupported")]
    UnsupportedProvider,
    #[error("OAuth metadata URL is unsafe")]
    UnsafeMetadata,
    #[error("OAuth callback credential scope does not match pending state")]
    CallbackCredentialMismatch,
}
