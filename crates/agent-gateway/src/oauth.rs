use agent_auth::{
    OAuthCallback, OAuthClientRegistration, OAuthCredential, OAuthError, OAuthProviderMetadata,
    OAuthRefreshRequest, OAuthRefreshResult, OAuthStatus, PendingOAuthState, ScopeSet,
    validate_oauth_issuer_url,
};
use agent_store::{OAuthStore, StoreError};
use serde::Deserialize;
use std::collections::BTreeSet;
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
        let pending = self.begin_callback_exchange(&callback, now_unix)?;
        if pending.upstream_id != credential.upstream_id || pending.subject != credential.subject {
            return Err(GatewayOAuthError::CallbackCredentialMismatch);
        }
        self.store.put_credential(credential)?;
        Ok(())
    }

    pub fn begin_callback_exchange(
        &mut self,
        callback: &OAuthCallback,
        now_unix: u64,
    ) -> Result<PendingOAuthState, GatewayOAuthError> {
        let pending = self
            .store
            .consume_pending_state(&callback.state)?
            .ok_or(OAuthError::StateNotFound)?;
        pending.validate_callback(callback, now_unix)?;
        Ok(pending)
    }

    pub fn finish_callback_exchange(
        &mut self,
        pending: PendingOAuthState,
        result: OAuthRefreshResult,
    ) -> Result<(), GatewayOAuthError> {
        self.store.put_credential(OAuthCredential {
            subject: pending.subject,
            upstream_id: pending.upstream_id,
            access_token: result.access_token,
            refresh_token: result.refresh_token,
            scopes: result.scopes.unwrap_or_else(ScopeSet::empty),
            expires_at_unix: result.expires_at_unix,
            refresh_failed: false,
        })?;
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

    pub fn refresh<P>(
        &mut self,
        subject: &str,
        upstream_id: &str,
        now_unix: u64,
        provider: &mut P,
    ) -> Result<OAuthStatus, GatewayOAuthError>
    where
        P: OAuthRefreshProvider,
    {
        let Some(request) = self.begin_refresh(subject, upstream_id, now_unix)? else {
            return Ok(OAuthStatus::RefreshFailed);
        };

        match provider.refresh(request) {
            Ok(result) => self.finish_refresh_success(subject, upstream_id, result),
            Err(error) => {
                self.mark_refresh_failed(subject, upstream_id)?;
                Err(GatewayOAuthError::Refresh(error))
            }
        }
    }

    pub fn begin_refresh(
        &mut self,
        subject: &str,
        upstream_id: &str,
        now_unix: u64,
    ) -> Result<Option<OAuthRefreshRequest>, GatewayOAuthError> {
        let Some(mut credential) = self.store.credential(subject, upstream_id)? else {
            return Err(OAuthError::CredentialNotFound.into());
        };
        let Some(refresh_token) = credential.refresh_token.clone() else {
            credential.refresh_failed = true;
            self.store.put_credential(credential)?;
            return Ok(None);
        };

        Ok(Some(OAuthRefreshRequest {
            subject: subject.to_string(),
            upstream_id: upstream_id.to_string(),
            refresh_token,
            scopes: credential.scopes,
            now_unix,
        }))
    }

    pub fn finish_refresh_success(
        &mut self,
        subject: &str,
        upstream_id: &str,
        result: OAuthRefreshResult,
    ) -> Result<OAuthStatus, GatewayOAuthError> {
        let Some(mut credential) = self.store.credential(subject, upstream_id)? else {
            return Err(OAuthError::CredentialNotFound.into());
        };
        credential.access_token = result.access_token;
        credential.refresh_token = result.refresh_token.or(credential.refresh_token);
        credential.scopes = result.scopes.unwrap_or(credential.scopes);
        credential.expires_at_unix = result.expires_at_unix;
        credential.refresh_failed = false;
        self.store.put_credential(credential)?;
        Ok(OAuthStatus::Connected)
    }

    pub fn mark_refresh_failed(
        &mut self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<(), GatewayOAuthError> {
        let Some(mut credential) = self.store.credential(subject, upstream_id)? else {
            return Err(OAuthError::CredentialNotFound.into());
        };
        credential.refresh_failed = true;
        self.store.put_credential(credential)?;
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

    pub fn put_client_registration(
        &mut self,
        registration: OAuthClientRegistration,
    ) -> Result<(), GatewayOAuthError> {
        self.store.put_client_registration(registration)?;
        Ok(())
    }

    pub fn client_registration(
        &self,
        subject: &str,
        upstream_id: &str,
    ) -> Result<Option<OAuthClientRegistration>, GatewayOAuthError> {
        self.store
            .client_registration(subject, upstream_id)
            .map_err(Into::into)
    }

    pub fn reconcile_upstreams(
        &mut self,
        current_upstream_ids: impl IntoIterator<Item = String>,
        known_oauth_upstream_ids: impl IntoIterator<Item = String>,
    ) -> Result<OAuthReconciliation, GatewayOAuthError> {
        let current = current_upstream_ids.into_iter().collect::<BTreeSet<_>>();
        let mut removed_upstream_ids = Vec::new();
        for upstream_id in known_oauth_upstream_ids {
            if !current.contains(&upstream_id) {
                self.store.clear_upstream(&upstream_id)?;
                removed_upstream_ids.push(upstream_id);
            }
        }
        removed_upstream_ids.sort();
        Ok(OAuthReconciliation {
            removed_upstream_ids,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OAuthReconciliation {
    pub removed_upstream_ids: Vec<String>,
}

pub trait OAuthRefreshProvider {
    fn refresh(
        &mut self,
        request: OAuthRefreshRequest,
    ) -> Result<OAuthRefreshResult, OAuthRefreshProviderError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OAuthTokenRefreshEndpoint {
    pub token_endpoint: String,
    pub client_id: String,
    pub client_secret: Option<String>,
}

impl OAuthTokenRefreshEndpoint {
    pub fn new(
        token_endpoint: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: Option<String>,
    ) -> Result<Self, GatewayOAuthError> {
        let token_endpoint = token_endpoint.into();
        validate_oauth_issuer_url(&token_endpoint)?;
        let client_id = client_id.into();
        if client_id.trim().is_empty() {
            return Err(GatewayOAuthError::MissingClientId);
        }
        Ok(Self {
            token_endpoint,
            client_id,
            client_secret,
        })
    }

    #[cfg(test)]
    fn new_unchecked_for_tests(
        token_endpoint: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: Option<String>,
    ) -> Self {
        Self {
            token_endpoint: token_endpoint.into(),
            client_id: client_id.into(),
            client_secret,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OAuthAuthorizationCodeEndpoint {
    pub token_endpoint: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    pub code_verifier: String,
}

impl OAuthAuthorizationCodeEndpoint {
    pub fn new(
        token_endpoint: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: Option<String>,
        redirect_uri: impl Into<String>,
        code_verifier: impl Into<String>,
    ) -> Result<Self, GatewayOAuthError> {
        let token_endpoint = token_endpoint.into();
        validate_oauth_issuer_url(&token_endpoint)?;
        let client_id = client_id.into();
        if client_id.trim().is_empty() {
            return Err(GatewayOAuthError::MissingClientId);
        }
        let redirect_uri = redirect_uri.into();
        if redirect_uri.trim().is_empty() {
            return Err(GatewayOAuthError::MissingRedirectUri);
        }
        let code_verifier = code_verifier.into();
        if code_verifier.trim().is_empty() {
            return Err(GatewayOAuthError::MissingCodeVerifier);
        }
        Ok(Self {
            token_endpoint,
            client_id,
            client_secret,
            redirect_uri,
            code_verifier,
        })
    }

    #[cfg(test)]
    fn new_unchecked_for_tests(
        token_endpoint: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: Option<String>,
        redirect_uri: impl Into<String>,
        code_verifier: impl Into<String>,
    ) -> Self {
        Self {
            token_endpoint: token_endpoint.into(),
            client_id: client_id.into(),
            client_secret,
            redirect_uri: redirect_uri.into(),
            code_verifier: code_verifier.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OAuthMetadataDiscoveryEndpoint {
    pub issuer_url: String,
    pub metadata_url: String,
}

impl OAuthMetadataDiscoveryEndpoint {
    pub fn new(issuer_url: impl Into<String>) -> Result<Self, GatewayOAuthError> {
        let issuer_url = issuer_url.into();
        let issuer = validate_oauth_issuer_url(&issuer_url)?;
        let metadata_url = metadata_url_for_issuer(&issuer)?;
        Ok(Self {
            issuer_url,
            metadata_url,
        })
    }

    #[cfg(test)]
    fn new_unchecked_for_tests(
        issuer_url: impl Into<String>,
        metadata_url: impl Into<String>,
    ) -> Self {
        Self {
            issuer_url: issuer_url.into(),
            metadata_url: metadata_url.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OAuthDynamicClientRegistrationEndpoint {
    pub registration_endpoint: String,
}

impl OAuthDynamicClientRegistrationEndpoint {
    pub fn new(registration_endpoint: impl Into<String>) -> Result<Self, GatewayOAuthError> {
        let registration_endpoint = registration_endpoint.into();
        validate_oauth_issuer_url(&registration_endpoint)?;
        Ok(Self {
            registration_endpoint,
        })
    }

    #[cfg(test)]
    fn new_unchecked_for_tests(registration_endpoint: impl Into<String>) -> Self {
        Self {
            registration_endpoint: registration_endpoint.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OAuthDynamicClientRegistrationRequest {
    pub subject: String,
    pub upstream_id: String,
    pub redirect_uris: Vec<String>,
    pub client_name: Option<String>,
}

impl OAuthDynamicClientRegistrationRequest {
    fn validate(&self) -> Result<(), OAuthRefreshProviderError> {
        if self.subject.trim().is_empty()
            || self.upstream_id.trim().is_empty()
            || self.redirect_uris.is_empty()
            || self.redirect_uris.iter().any(|uri| uri.trim().is_empty())
        {
            return Err(OAuthRefreshProviderError::Failed);
        }
        Ok(())
    }
}

fn metadata_url_for_issuer(issuer: &Url) -> Result<String, GatewayOAuthError> {
    let mut metadata = issuer.clone();
    let issuer_path = issuer.path().trim_start_matches('/');
    let metadata_path = if issuer_path.is_empty() {
        "/.well-known/oauth-authorization-server".to_string()
    } else {
        format!("/.well-known/oauth-authorization-server/{issuer_path}")
    };
    metadata.set_path(&metadata_path);
    metadata.set_query(None);
    metadata.set_fragment(None);
    Ok(metadata.to_string())
}

#[derive(Clone, Debug)]
pub struct HttpOAuthTokenClient {
    client: reqwest::Client,
}

impl Default for HttpOAuthTokenClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpOAuthTokenClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn discover_metadata(
        &self,
        endpoint: &OAuthMetadataDiscoveryEndpoint,
    ) -> Result<OAuthProviderMetadata, OAuthRefreshProviderError> {
        let response = self
            .client
            .get(&endpoint.metadata_url)
            .send()
            .await
            .map_err(|_| OAuthRefreshProviderError::Failed)?;
        if !response.status().is_success() {
            return Err(OAuthRefreshProviderError::Failed);
        }
        response
            .json::<OAuthProviderMetadata>()
            .await
            .map_err(|_| OAuthRefreshProviderError::Failed)
    }

    pub async fn register_client(
        &self,
        endpoint: &OAuthDynamicClientRegistrationEndpoint,
        request: OAuthDynamicClientRegistrationRequest,
    ) -> Result<OAuthClientRegistration, OAuthRefreshProviderError> {
        request.validate()?;
        let mut body = serde_json::json!({
            "redirect_uris": request.redirect_uris,
            "grant_types": ["authorization_code", "refresh_token"],
            "response_types": ["code"],
            "token_endpoint_auth_method": "client_secret_post"
        });
        if let Some(client_name) = request.client_name.as_deref() {
            body["client_name"] = serde_json::Value::String(client_name.to_string());
        }

        let response = self
            .client
            .post(&endpoint.registration_endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|_| OAuthRefreshProviderError::Failed)?;
        if !response.status().is_success() {
            return Err(OAuthRefreshProviderError::Failed);
        }
        let registration = response
            .json::<OAuthDynamicClientRegistrationResponse>()
            .await
            .map_err(|_| OAuthRefreshProviderError::Failed)?;
        registration.into_registration(request.subject, request.upstream_id)
    }

    pub async fn refresh(
        &self,
        endpoint: &OAuthTokenRefreshEndpoint,
        request: OAuthRefreshRequest,
    ) -> Result<OAuthRefreshResult, OAuthRefreshProviderError> {
        let form = {
            let mut form = url::form_urlencoded::Serializer::new(String::new());
            form.append_pair("grant_type", "refresh_token");
            form.append_pair("refresh_token", request.refresh_token.as_str());
            form.append_pair("client_id", endpoint.client_id.as_str());
            if let Some(client_secret) = endpoint.client_secret.as_deref() {
                form.append_pair("client_secret", client_secret);
            }
            let scope = request.scopes.as_header_value();
            if !scope.is_empty() {
                form.append_pair("scope", scope.as_str());
            }
            form.finish()
        };

        let response = self
            .client
            .post(&endpoint.token_endpoint)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(form)
            .send()
            .await
            .map_err(|_| OAuthRefreshProviderError::Failed)?;
        if !response.status().is_success() {
            return Err(OAuthRefreshProviderError::Failed);
        }
        let token = response
            .json::<OAuthTokenRefreshResponse>()
            .await
            .map_err(|_| OAuthRefreshProviderError::Failed)?;
        token.into_result(request.now_unix)
    }

    pub async fn exchange_authorization_code(
        &self,
        endpoint: &OAuthAuthorizationCodeEndpoint,
        callback: &OAuthCallback,
        now_unix: u64,
    ) -> Result<OAuthRefreshResult, OAuthRefreshProviderError> {
        let form = {
            let mut form = url::form_urlencoded::Serializer::new(String::new());
            form.append_pair("grant_type", "authorization_code");
            form.append_pair("code", callback.code.as_str());
            form.append_pair("redirect_uri", endpoint.redirect_uri.as_str());
            form.append_pair("client_id", endpoint.client_id.as_str());
            form.append_pair("code_verifier", endpoint.code_verifier.as_str());
            if let Some(client_secret) = endpoint.client_secret.as_deref() {
                form.append_pair("client_secret", client_secret);
            }
            form.finish()
        };

        let response = self
            .client
            .post(&endpoint.token_endpoint)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(form)
            .send()
            .await
            .map_err(|_| OAuthRefreshProviderError::Failed)?;
        if !response.status().is_success() {
            return Err(OAuthRefreshProviderError::Failed);
        }
        let token = response
            .json::<OAuthTokenRefreshResponse>()
            .await
            .map_err(|_| OAuthRefreshProviderError::Failed)?;
        token.into_result(now_unix)
    }
}

#[derive(Debug, Deserialize)]
struct OAuthTokenRefreshResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OAuthDynamicClientRegistrationResponse {
    client_id: String,
    client_secret: Option<String>,
    client_id_issued_at: Option<u64>,
    client_secret_expires_at: Option<u64>,
}

impl OAuthDynamicClientRegistrationResponse {
    fn into_registration(
        self,
        subject: String,
        upstream_id: String,
    ) -> Result<OAuthClientRegistration, OAuthRefreshProviderError> {
        if self.client_id.trim().is_empty() {
            return Err(OAuthRefreshProviderError::Failed);
        }
        Ok(OAuthClientRegistration {
            subject,
            upstream_id,
            client_id: self.client_id,
            client_secret: self.client_secret,
            client_id_issued_at_unix: self.client_id_issued_at,
            client_secret_expires_at_unix: self.client_secret_expires_at,
        })
    }
}

impl OAuthTokenRefreshResponse {
    fn into_result(self, now_unix: u64) -> Result<OAuthRefreshResult, OAuthRefreshProviderError> {
        if self.access_token.trim().is_empty() {
            return Err(OAuthRefreshProviderError::Failed);
        }
        let scopes = self
            .scope
            .as_deref()
            .map(ScopeSet::parse)
            .transpose()
            .map_err(|_| OAuthRefreshProviderError::Failed)?;
        Ok(OAuthRefreshResult {
            access_token: self.access_token,
            refresh_token: self.refresh_token,
            scopes,
            expires_at_unix: now_unix + self.expires_in.unwrap_or(3600),
        })
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum OAuthRefreshProviderError {
    #[error("OAuth provider refresh failed")]
    Failed,
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
    #[error("OAuth client id is required")]
    MissingClientId,
    #[error("OAuth redirect URI is required")]
    MissingRedirectUri,
    #[error("OAuth code verifier is required")]
    MissingCodeVerifier,
    #[error("OAuth callback credential scope does not match pending state")]
    CallbackCredentialMismatch,
    #[error(transparent)]
    Refresh(#[from] OAuthRefreshProviderError),
}
