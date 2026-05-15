use crate::ScopeSet;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use thiserror::Error;
use url::Url;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct OAuthProviderMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    #[serde(default)]
    pub registration_endpoint: Option<String>,
    pub code_challenge_methods_supported: Vec<String>,
    pub scopes_supported: ScopeSet,
}

impl OAuthProviderMetadata {
    pub fn supports_pkce_s256(&self) -> bool {
        self.code_challenge_methods_supported
            .iter()
            .any(|method| method == "S256")
    }

    pub fn selected_scope(
        &self,
        challenge_scopes: Option<&ScopeSet>,
        protected_resource_scopes: Option<&ScopeSet>,
    ) -> Option<ScopeSet> {
        challenge_scopes
            .filter(|scopes| !scopes.is_empty())
            .cloned()
            .or_else(|| {
                protected_resource_scopes
                    .filter(|scopes| !scopes.is_empty())
                    .cloned()
            })
            .or_else(|| {
                if self.scopes_supported.is_empty() {
                    None
                } else {
                    Some(self.scopes_supported.clone())
                }
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct PendingOAuthState {
    pub state: String,
    pub subject: String,
    pub upstream_id: String,
    pub expires_at_unix: u64,
}

impl PendingOAuthState {
    pub fn validate_callback(
        &self,
        callback: &OAuthCallback,
        now_unix: u64,
    ) -> Result<(), OAuthError> {
        if now_unix > self.expires_at_unix {
            return Err(OAuthError::ExpiredState);
        }
        if self.state != callback.state {
            return Err(OAuthError::StateMismatch);
        }
        if self.subject != callback.subject {
            return Err(OAuthError::SubjectMismatch);
        }
        if callback.code.trim().is_empty() {
            return Err(OAuthError::MissingCode);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct OAuthCallback {
    pub state: String,
    pub subject: String,
    pub code: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct OAuthCredential {
    pub subject: String,
    pub upstream_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub scopes: ScopeSet,
    pub expires_at_unix: u64,
    pub refresh_failed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct OAuthRefreshRequest {
    pub subject: String,
    pub upstream_id: String,
    pub refresh_token: String,
    pub scopes: ScopeSet,
    pub now_unix: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct OAuthRefreshResult {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub scopes: Option<ScopeSet>,
    pub expires_at_unix: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct OAuthClientRegistration {
    pub subject: String,
    pub upstream_id: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub client_id_issued_at_unix: Option<u64>,
    pub client_secret_expires_at_unix: Option<u64>,
}

impl OAuthCredential {
    pub fn status(&self, now_unix: u64, expiring_window_secs: u64) -> OAuthStatus {
        if self.refresh_failed {
            return OAuthStatus::RefreshFailed;
        }
        if self.expires_at_unix <= now_unix {
            return OAuthStatus::Expired;
        }
        if self.expires_at_unix.saturating_sub(now_unix) <= expiring_window_secs {
            return OAuthStatus::Expiring;
        }
        OAuthStatus::Connected
    }

    pub fn redacted_access_token(&self) -> &'static str {
        "[REDACTED]"
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuthStatus {
    Disconnected,
    DiscoveryFailed,
    UnsupportedProvider,
    Connected,
    Expiring,
    Expired,
    RefreshFailed,
}

pub fn validate_oauth_issuer_url(raw: &str) -> Result<Url, OAuthError> {
    let url = Url::parse(raw).map_err(|_| OAuthError::UnsafeIssuerUrl)?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(OAuthError::UnsafeIssuerUrl);
    }

    let host = url.host_str().ok_or(OAuthError::UnsafeIssuerUrl)?;
    let lower_host = host.to_ascii_lowercase();
    if matches!(
        lower_host.as_str(),
        "localhost" | "metadata.google.internal" | "metadata"
    ) {
        return Err(OAuthError::UnsafeIssuerUrl);
    }

    if let Ok(ip) = lower_host.parse::<IpAddr>() {
        reject_unsafe_ip(ip)?;
    }

    Ok(url)
}

fn reject_unsafe_ip(ip: IpAddr) -> Result<(), OAuthError> {
    match ip {
        IpAddr::V4(ip) => {
            if ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_broadcast()
                || ip.is_documentation()
                || ip.octets() == [169, 254, 169, 254]
            {
                return Err(OAuthError::UnsafeIssuerUrl);
            }
        }
        IpAddr::V6(ip) => {
            if ip.is_loopback() || ip.is_unspecified() || ip.is_unique_local() {
                return Err(OAuthError::UnsafeIssuerUrl);
            }
        }
    }
    Ok(())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum OAuthError {
    #[error("OAuth issuer URL is unsafe")]
    UnsafeIssuerUrl,
    #[error("OAuth provider does not support PKCE S256")]
    UnsupportedPkce,
    #[error("OAuth callback state expired")]
    ExpiredState,
    #[error("OAuth callback state does not match")]
    StateMismatch,
    #[error("OAuth callback subject does not match")]
    SubjectMismatch,
    #[error("OAuth callback is missing authorization code")]
    MissingCode,
    #[error("OAuth state was not found")]
    StateNotFound,
    #[error("OAuth credential was not found")]
    CredentialNotFound,
    #[error("OAuth credential does not include a refresh token")]
    MissingRefreshToken,
    #[error("OAuth refresh failed")]
    RefreshFailed,
}
