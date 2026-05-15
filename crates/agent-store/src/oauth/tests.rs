use super::*;
use agent_auth::ScopeSet;

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

    assert!(store.consume_pending_state("state-1").expect("consume").is_some());
    assert!(store.consume_pending_state("state-1").expect("consume").is_none());
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
        .clear_subject_upstream("user-1", "github")
        .expect("clear");

    assert!(
        store
            .credential("user-1", "github")
            .expect("credential")
            .is_none()
    );
    assert!(store.consume_pending_state("state-1").expect("state").is_none());
}
