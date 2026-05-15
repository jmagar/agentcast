use super::*;
use agent_auth::{OAuthCallback, OAuthCredential};
use agent_store::InMemoryOAuthStore;

fn scopes(raw: &str) -> ScopeSet {
    ScopeSet::parse(raw).expect("scope")
}

fn metadata() -> OAuthProviderMetadata {
    OAuthProviderMetadata {
        issuer: "https://auth.example.test".to_string(),
        authorization_endpoint: "https://auth.example.test/authorize".to_string(),
        token_endpoint: "https://auth.example.test/token".to_string(),
        code_challenge_methods_supported: vec!["S256".to_string()],
        scopes_supported: scopes("repo"),
    }
}

fn begin() -> BeginAuthorization {
    BeginAuthorization {
        issuer_url: "https://auth.example.test".to_string(),
        subject: "user-1".to_string(),
        upstream_id: "github".to_string(),
        client_id: "agentcast".to_string(),
        redirect_uri: "https://agentcast.example.test/oauth/callback".to_string(),
        resource_uri: "https://api.github.example.test".to_string(),
        state: "state-1".to_string(),
        code_challenge: "challenge".to_string(),
        expires_at_unix: 100,
        challenge_scopes: Some(scopes("repo")),
        protected_resource_scopes: None,
    }
}

#[test]
fn probe_reports_unsupported_provider_without_live_network() {
    let service = GatewayOAuthService::new(InMemoryOAuthStore::default());
    let mut metadata = metadata();
    metadata.code_challenge_methods_supported = vec!["plain".to_string()];

    let status = service
        .probe_metadata("https://auth.example.test", &metadata)
        .expect("probe");

    assert_eq!(status, OAuthStatus::UnsupportedProvider);
}

#[test]
fn begin_authorization_records_pending_state_and_scope() {
    let mut service = GatewayOAuthService::new(InMemoryOAuthStore::default());

    let result = service
        .begin_authorization(begin(), &metadata())
        .expect("begin auth");

    assert!(result.authorization_url.contains("response_type=code"));
    assert!(
        result
            .authorization_url
            .contains("code_challenge_method=S256")
    );
    assert_eq!(result.selected_scope.expect("scope").as_slice(), &["repo"]);
}

#[test]
fn callback_consumes_state_once_and_persists_credential() {
    let mut service = GatewayOAuthService::new(InMemoryOAuthStore::default());
    service
        .begin_authorization(begin(), &metadata())
        .expect("begin auth");
    let callback = OAuthCallback {
        state: "state-1".to_string(),
        subject: "user-1".to_string(),
        code: "code".to_string(),
    };
    let credential = OAuthCredential {
        subject: "user-1".to_string(),
        upstream_id: "github".to_string(),
        access_token: "upstream-token".to_string(),
        refresh_token: Some("refresh-token".to_string()),
        scopes: scopes("repo"),
        expires_at_unix: 1000,
        refresh_failed: false,
    };

    service
        .complete_callback(callback.clone(), credential, 50)
        .expect("complete callback");
    assert_eq!(
        service.status("user-1", "github", 50).expect("status"),
        OAuthStatus::Connected
    );
    assert_eq!(
        service.complete_callback(
            callback,
            OAuthCredential {
                subject: "user-1".to_string(),
                upstream_id: "github".to_string(),
                access_token: "second-token".to_string(),
                refresh_token: None,
                scopes: scopes("repo"),
                expires_at_unix: 1000,
                refresh_failed: false,
            },
            50,
        ),
        Err(GatewayOAuthError::Auth(OAuthError::StateNotFound))
    );
}

#[test]
fn runtime_credential_lookup_uses_subject_scoped_upstream_token() {
    let mut service = GatewayOAuthService::new(InMemoryOAuthStore::default());
    service
        .store_mut()
        .put_credential(OAuthCredential {
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            access_token: "upstream-token".to_string(),
            refresh_token: None,
            scopes: scopes("repo"),
            expires_at_unix: 1000,
            refresh_failed: false,
        })
        .expect("store credential");

    let credential = service
        .credential_for_runtime("user-1", "github")
        .expect("credential")
        .expect("credential");

    assert_eq!(credential.access_token, "upstream-token");
    assert_eq!(credential.redacted_access_token(), "[REDACTED]");
}

#[test]
fn clear_removes_status_and_runtime_credentials() {
    let mut service = GatewayOAuthService::new(InMemoryOAuthStore::default());
    service
        .store_mut()
        .put_credential(OAuthCredential {
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            access_token: "upstream-token".to_string(),
            refresh_token: None,
            scopes: scopes("repo"),
            expires_at_unix: 1000,
            refresh_failed: false,
        })
        .expect("store credential");

    service.clear("user-1", "github").expect("clear");

    assert_eq!(
        service.status("user-1", "github", 50).expect("status"),
        OAuthStatus::Disconnected
    );
    assert!(
        service
            .credential_for_runtime("user-1", "github")
            .expect("credential")
            .is_none()
    );
}
