use super::*;
use agent_auth::ScopeSet;

fn test_key() -> [u8; 32] {
    [7; 32]
}

#[test]
fn pending_state_is_consumed_once() {
    let mut store = InMemoryOAuthStore::default();
    store
        .put_pending_state(PendingOAuthState {
            state: "state-1".to_string(),
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            expires_at_unix: 100,
        })
        .expect("put state");

    assert!(
        store
            .consume_pending_state("state-1")
            .expect("consume")
            .is_some()
    );
    assert!(
        store
            .consume_pending_state("state-1")
            .expect("consume")
            .is_none()
    );
}

#[test]
fn clear_removes_credentials_and_pending_state_for_scope() {
    let mut store = InMemoryOAuthStore::default();
    store
        .put_pending_state(PendingOAuthState {
            state: "state-1".to_string(),
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            expires_at_unix: 100,
        })
        .expect("put state");
    store
        .put_credential(OAuthCredential {
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            access_token: "secret".to_string(),
            refresh_token: None,
            scopes: ScopeSet::parse("repo").expect("scope"),
            expires_at_unix: 200,
            refresh_failed: false,
        })
        .expect("put credential");
    store
        .put_client_registration(OAuthClientRegistration {
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            client_id: "client-1".to_string(),
            client_secret: Some("client-secret".to_string()),
            client_id_issued_at_unix: Some(10),
            client_secret_expires_at_unix: Some(1000),
        })
        .expect("put registration");

    store
        .clear_subject_upstream("user-1", "github")
        .expect("clear");

    assert!(
        store
            .credential("user-1", "github")
            .expect("credential")
            .is_none()
    );
    assert!(
        store
            .consume_pending_state("state-1")
            .expect("state")
            .is_none()
    );
    assert!(
        store
            .client_registration("user-1", "github")
            .expect("registration")
            .is_none()
    );
}

#[test]
fn sqlite_pending_state_is_consumed_once() {
    let mut store = SqliteOAuthStore::in_memory(test_key()).expect("store");
    store
        .put_pending_state(PendingOAuthState {
            state: "state-1".to_string(),
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            expires_at_unix: 100,
        })
        .expect("put state");

    assert!(
        store
            .consume_pending_state("state-1")
            .expect("consume")
            .is_some()
    );
    assert!(
        store
            .consume_pending_state("state-1")
            .expect("consume")
            .is_none()
    );
}

#[test]
fn sqlite_credentials_round_trip_without_plaintext_storage() {
    let mut store = SqliteOAuthStore::in_memory(test_key()).expect("store");
    store
        .put_credential(OAuthCredential {
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            access_token: "secret-access".to_string(),
            refresh_token: Some("secret-refresh".to_string()),
            scopes: ScopeSet::parse("repo").expect("scope"),
            expires_at_unix: 200,
            refresh_failed: false,
        })
        .expect("put credential");

    let ciphertext = store
        .raw_access_token_ciphertext("user-1", "github")
        .expect("ciphertext")
        .expect("ciphertext");
    assert!(!String::from_utf8_lossy(&ciphertext).contains("secret-access"));

    let credential = store
        .credential("user-1", "github")
        .expect("credential")
        .expect("credential");
    assert_eq!(credential.access_token, "secret-access");
    assert_eq!(credential.refresh_token.as_deref(), Some("secret-refresh"));
    assert_eq!(credential.scopes.as_slice(), &["repo"]);
}

#[test]
fn sqlite_credential_rewrite_uses_fresh_nonce() {
    let mut store = SqliteOAuthStore::in_memory(test_key()).expect("store");
    let credential = OAuthCredential {
        subject: "user-1".to_string(),
        upstream_id: "github".to_string(),
        access_token: "secret-access".to_string(),
        refresh_token: None,
        scopes: ScopeSet::parse("repo").expect("scope"),
        expires_at_unix: 200,
        refresh_failed: false,
    };

    store
        .put_credential(credential.clone())
        .expect("put credential");
    let first_nonce = store
        .raw_access_token_nonce("user-1", "github")
        .expect("nonce")
        .expect("nonce");

    store.put_credential(credential).expect("put credential");
    let second_nonce = store
        .raw_access_token_nonce("user-1", "github")
        .expect("nonce")
        .expect("nonce");

    assert_ne!(first_nonce, second_nonce);
}

#[test]
fn sqlite_client_registration_round_trip_without_plaintext_secret() {
    let mut store = SqliteOAuthStore::in_memory(test_key()).expect("store");
    store
        .put_client_registration(OAuthClientRegistration {
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            client_id: "client-1".to_string(),
            client_secret: Some("client-secret".to_string()),
            client_id_issued_at_unix: Some(10),
            client_secret_expires_at_unix: Some(1000),
        })
        .expect("put registration");

    let registration = store
        .client_registration("user-1", "github")
        .expect("registration")
        .expect("registration");

    assert_eq!(registration.client_id, "client-1");
    assert_eq!(registration.client_secret.as_deref(), Some("client-secret"));
    assert_eq!(registration.client_id_issued_at_unix, Some(10));
    assert_eq!(registration.client_secret_expires_at_unix, Some(1000));
}

#[test]
fn sqlite_clear_removes_credentials_and_pending_state_for_scope() {
    let mut store = SqliteOAuthStore::in_memory(test_key()).expect("store");
    store
        .put_pending_state(PendingOAuthState {
            state: "state-1".to_string(),
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            expires_at_unix: 100,
        })
        .expect("put state");
    store
        .put_credential(OAuthCredential {
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            access_token: "secret".to_string(),
            refresh_token: None,
            scopes: ScopeSet::parse("repo").expect("scope"),
            expires_at_unix: 200,
            refresh_failed: false,
        })
        .expect("put credential");
    store
        .put_client_registration(OAuthClientRegistration {
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            client_id: "client-1".to_string(),
            client_secret: Some("client-secret".to_string()),
            client_id_issued_at_unix: None,
            client_secret_expires_at_unix: None,
        })
        .expect("put registration");

    store
        .clear_subject_upstream("user-1", "github")
        .expect("clear");

    assert!(
        store
            .credential("user-1", "github")
            .expect("credential")
            .is_none()
    );
    assert!(
        store
            .consume_pending_state("state-1")
            .expect("state")
            .is_none()
    );
    assert!(
        store
            .client_registration("user-1", "github")
            .expect("registration")
            .is_none()
    );
}

#[test]
fn sqlite_clear_upstream_removes_all_subject_state_for_upstream() {
    let mut store = SqliteOAuthStore::in_memory(test_key()).expect("store");
    for subject in ["user-1", "user-2"] {
        store
            .put_pending_state(PendingOAuthState {
                state: format!("state-{subject}"),
                subject: subject.to_string(),
                upstream_id: "github".to_string(),
                expires_at_unix: 100,
            })
            .expect("put state");
        store
            .put_credential(OAuthCredential {
                subject: subject.to_string(),
                upstream_id: "github".to_string(),
                access_token: "secret".to_string(),
                refresh_token: None,
                scopes: ScopeSet::parse("repo").expect("scope"),
                expires_at_unix: 200,
                refresh_failed: false,
            })
            .expect("put credential");
        store
            .put_client_registration(OAuthClientRegistration {
                subject: subject.to_string(),
                upstream_id: "github".to_string(),
                client_id: format!("client-{subject}"),
                client_secret: Some("client-secret".to_string()),
                client_id_issued_at_unix: None,
                client_secret_expires_at_unix: None,
            })
            .expect("put registration");
    }

    store.clear_upstream("github").expect("clear upstream");

    for subject in ["user-1", "user-2"] {
        assert!(
            store
                .credential(subject, "github")
                .expect("credential")
                .is_none()
        );
        assert!(
            store
                .client_registration(subject, "github")
                .expect("registration")
                .is_none()
        );
        assert!(
            store
                .consume_pending_state(&format!("state-{subject}"))
                .expect("state")
                .is_none()
        );
    }
}
