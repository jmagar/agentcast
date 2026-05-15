use super::*;

fn scopes(raw: &str) -> ScopeSet {
    ScopeSet::parse(raw).expect("scope")
}

#[test]
fn oauth_probe_rejects_unsafe_urls() {
    for raw in [
        "http://auth.example.test",
        "https://user@auth.example.test",
        "https://auth.example.test/path?x=1",
        "https://auth.example.test/path#fragment",
        "https://localhost",
        "https://127.0.0.1",
        "https://10.0.0.1",
        "https://169.254.169.254",
        "https://metadata.google.internal",
    ] {
        assert_eq!(
            validate_oauth_issuer_url(raw).expect_err("unsafe"),
            OAuthError::UnsafeIssuerUrl
        );
    }
}

#[test]
fn oauth_probe_accepts_https_public_issuer() {
    let url = validate_oauth_issuer_url("https://auth.example.test/oauth").expect("safe url");

    assert_eq!(url.host_str(), Some("auth.example.test"));
}

#[test]
fn provider_requires_pkce_s256_and_selects_scope_by_priority() {
    let metadata = OAuthProviderMetadata {
        issuer: "https://auth.example.test".to_string(),
        authorization_endpoint: "https://auth.example.test/authorize".to_string(),
        token_endpoint: "https://auth.example.test/token".to_string(),
        registration_endpoint: None,
        code_challenge_methods_supported: vec!["plain".to_string(), "S256".to_string()],
        scopes_supported: scopes("fallback"),
    };

    assert!(metadata.supports_pkce_s256());
    assert_eq!(
        metadata
            .selected_scope(Some(&scopes("challenge")), Some(&scopes("resource")))
            .expect("scope")
            .as_slice(),
        &["challenge"]
    );
}

#[test]
fn callback_validation_rejects_replay_shape() {
    let pending = PendingOAuthState {
        state: "state-1".to_string(),
        subject: "user-1".to_string(),
        upstream_id: "github".to_string(),
        expires_at_unix: 100,
    };

    let wrong_subject = OAuthCallback {
        state: "state-1".to_string(),
        subject: "user-2".to_string(),
        code: "code".to_string(),
    };

    assert_eq!(
        pending.validate_callback(&wrong_subject, 50),
        Err(OAuthError::SubjectMismatch)
    );
    assert_eq!(
        pending.validate_callback(
            &OAuthCallback {
                state: "state-1".to_string(),
                subject: "user-1".to_string(),
                code: "code".to_string(),
            },
            101,
        ),
        Err(OAuthError::ExpiredState)
    );
}

#[test]
fn credential_status_is_redacted_and_time_based() {
    let credential = OAuthCredential {
        subject: "user-1".to_string(),
        upstream_id: "github".to_string(),
        access_token: "secret".to_string(),
        refresh_token: Some("refresh-secret".to_string()),
        scopes: scopes("repo"),
        expires_at_unix: 120,
        refresh_failed: false,
    };

    assert_eq!(credential.status(50, 60), OAuthStatus::Connected);
    assert_eq!(credential.status(70, 60), OAuthStatus::Expiring);
    assert_eq!(credential.status(121, 60), OAuthStatus::Expired);
    assert_eq!(credential.redacted_access_token(), "[REDACTED]");
}
