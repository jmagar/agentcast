use super::*;
use agent_auth::{
    OAuthCallback, OAuthClientRegistration, OAuthCredential, OAuthRefreshRequest,
    OAuthRefreshResult,
};
use agent_store::InMemoryOAuthStore;
use std::io::{Read, Write};

fn scopes(raw: &str) -> ScopeSet {
    ScopeSet::parse(raw).expect("scope")
}

fn metadata() -> OAuthProviderMetadata {
    OAuthProviderMetadata {
        issuer: "https://auth.example.test".to_string(),
        authorization_endpoint: "https://auth.example.test/authorize".to_string(),
        token_endpoint: "https://auth.example.test/token".to_string(),
        registration_endpoint: Some("https://auth.example.test/register".to_string()),
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

#[test]
fn refresh_updates_credential_and_clears_failure_state() {
    let mut service = GatewayOAuthService::new(InMemoryOAuthStore::default());
    service
        .store_mut()
        .put_credential(OAuthCredential {
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            access_token: "old-access".to_string(),
            refresh_token: Some("old-refresh".to_string()),
            scopes: scopes("repo"),
            expires_at_unix: 1000,
            refresh_failed: true,
        })
        .expect("store credential");
    let mut provider = FixtureRefreshProvider {
        result: Ok(OAuthRefreshResult {
            access_token: "new-access".to_string(),
            refresh_token: Some("new-refresh".to_string()),
            scopes: Some(scopes("repo read:org")),
            expires_at_unix: 5000,
        }),
        seen_request: None,
    };

    let status = service
        .refresh("user-1", "github", 900, &mut provider)
        .expect("refresh");

    assert_eq!(status, OAuthStatus::Connected);
    assert_eq!(
        provider.seen_request.expect("request").refresh_token,
        "old-refresh"
    );
    let credential = service
        .credential_for_runtime("user-1", "github")
        .expect("credential")
        .expect("credential");
    assert_eq!(credential.access_token, "new-access");
    assert_eq!(credential.refresh_token.as_deref(), Some("new-refresh"));
    assert_eq!(credential.scopes.as_slice(), &["read:org", "repo"]);
    assert!(!credential.refresh_failed);
}

#[test]
fn refresh_failure_marks_status_refresh_failed_without_exposing_tokens() {
    let mut service = GatewayOAuthService::new(InMemoryOAuthStore::default());
    service
        .store_mut()
        .put_credential(OAuthCredential {
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            access_token: "old-access".to_string(),
            refresh_token: Some("old-refresh".to_string()),
            scopes: scopes("repo"),
            expires_at_unix: 1000,
            refresh_failed: false,
        })
        .expect("store credential");
    let mut provider = FixtureRefreshProvider {
        result: Err(OAuthRefreshProviderError::Failed),
        seen_request: None,
    };

    assert_eq!(
        service.refresh("user-1", "github", 900, &mut provider),
        Err(GatewayOAuthError::Refresh(
            OAuthRefreshProviderError::Failed
        ))
    );
    assert_eq!(
        service.status("user-1", "github", 900).expect("status"),
        OAuthStatus::RefreshFailed
    );
}

#[test]
fn refresh_without_refresh_token_marks_refresh_failed() {
    let mut service = GatewayOAuthService::new(InMemoryOAuthStore::default());
    service
        .store_mut()
        .put_credential(OAuthCredential {
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            access_token: "old-access".to_string(),
            refresh_token: None,
            scopes: scopes("repo"),
            expires_at_unix: 1000,
            refresh_failed: false,
        })
        .expect("store credential");
    let mut provider = FixtureRefreshProvider {
        result: Ok(OAuthRefreshResult {
            access_token: "unused".to_string(),
            refresh_token: None,
            scopes: None,
            expires_at_unix: 5000,
        }),
        seen_request: None,
    };

    let status = service
        .refresh("user-1", "github", 900, &mut provider)
        .expect("refresh");

    assert_eq!(status, OAuthStatus::RefreshFailed);
    assert!(provider.seen_request.is_none());
    assert_eq!(
        service.status("user-1", "github", 900).expect("status"),
        OAuthStatus::RefreshFailed
    );
}

#[tokio::test]
async fn http_refresh_client_exchanges_refresh_token_without_live_provider() {
    let server = TokenFixtureServer::start(
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\r\n\
         {\"access_token\":\"new-access\",\"refresh_token\":\"new-refresh\",\"expires_in\":120,\"scope\":\"repo read:org\"}",
    );
    let endpoint = OAuthTokenRefreshEndpoint::new_unchecked_for_tests(
        server.url(),
        "agentcast",
        Some("client-secret".to_string()),
    );
    let client = HttpOAuthTokenClient::new();

    let result = client
        .refresh(
            &endpoint,
            OAuthRefreshRequest {
                subject: "user-1".to_string(),
                upstream_id: "github".to_string(),
                refresh_token: "old-refresh".to_string(),
                scopes: scopes("repo"),
                now_unix: 1000,
            },
        )
        .await
        .expect("refresh");

    assert_eq!(result.access_token, "new-access");
    assert_eq!(result.refresh_token.as_deref(), Some("new-refresh"));
    assert_eq!(result.expires_at_unix, 1120);
    assert_eq!(
        result.scopes.expect("scope").as_slice(),
        &["read:org", "repo"]
    );
    let request = server.request();
    assert!(request.contains("grant_type=refresh_token"));
    assert!(request.contains("refresh_token=old-refresh"));
    assert!(request.contains("client_id=agentcast"));
    assert!(request.contains("client_secret=client-secret"));
}

#[tokio::test]
async fn http_token_client_discovers_oauth_metadata_without_live_provider() {
    let server = TokenFixtureServer::start(
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\r\n\
         {\"issuer\":\"https://auth.example.test\",\"authorization_endpoint\":\"https://auth.example.test/authorize\",\"token_endpoint\":\"https://auth.example.test/token\",\"code_challenge_methods_supported\":[\"S256\"],\"scopes_supported\":[\"repo\"]}",
    );
    let endpoint = OAuthMetadataDiscoveryEndpoint::new_unchecked_for_tests(
        "https://auth.example.test",
        server.url(),
    );
    let client = HttpOAuthTokenClient::new();

    let metadata = client.discover_metadata(&endpoint).await.expect("metadata");

    assert_eq!(metadata.issuer, "https://auth.example.test");
    assert_eq!(metadata.scopes_supported.as_slice(), &["repo"]);
    let request = server.request();
    assert!(request.starts_with("GET /token HTTP/1.1"));
}

#[tokio::test]
async fn http_token_client_registers_dynamic_client_without_live_provider() {
    let server = TokenFixtureServer::start(
        "HTTP/1.1 201 Created\r\ncontent-type: application/json\r\n\r\n\
         {\"client_id\":\"dynamic-client\",\"client_secret\":\"dynamic-secret\",\"client_id_issued_at\":100,\"client_secret_expires_at\":5000}",
    );
    let endpoint = OAuthDynamicClientRegistrationEndpoint::new_unchecked_for_tests(server.url());
    let client = HttpOAuthTokenClient::new();

    let registration = client
        .register_client(
            &endpoint,
            OAuthDynamicClientRegistrationRequest {
                subject: "user-1".to_string(),
                upstream_id: "github".to_string(),
                redirect_uris: vec!["https://agentcast.example.test/oauth/callback".to_string()],
                client_name: Some("AgentCast".to_string()),
            },
        )
        .await
        .expect("registration");

    assert_eq!(registration.client_id, "dynamic-client");
    assert_eq!(
        registration.client_secret.as_deref(),
        Some("dynamic-secret")
    );
    assert_eq!(registration.client_id_issued_at_unix, Some(100));
    assert_eq!(registration.client_secret_expires_at_unix, Some(5000));
    let request = server.request();
    assert!(request.starts_with("POST /token HTTP/1.1"));
    assert!(request.contains("\"redirect_uris\""));
    assert!(request.contains("\"authorization_code\""));
}

#[tokio::test]
async fn http_token_client_exchanges_authorization_code_without_live_provider() {
    let server = TokenFixtureServer::start(
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\r\n\
         {\"access_token\":\"code-access\",\"refresh_token\":\"code-refresh\",\"expires_in\":300,\"scope\":\"repo\"}",
    );
    let endpoint = OAuthAuthorizationCodeEndpoint::new_unchecked_for_tests(
        server.url(),
        "agentcast",
        Some("client-secret".to_string()),
        "https://agentcast.example.test/oauth/callback",
        "verifier",
    );
    let client = HttpOAuthTokenClient::new();

    let result = client
        .exchange_authorization_code(
            &endpoint,
            &OAuthCallback {
                state: "state-1".to_string(),
                subject: "user-1".to_string(),
                code: "auth-code".to_string(),
            },
            1000,
        )
        .await
        .expect("exchange");

    assert_eq!(result.access_token, "code-access");
    assert_eq!(result.refresh_token.as_deref(), Some("code-refresh"));
    assert_eq!(result.expires_at_unix, 1300);
    assert_eq!(result.scopes.expect("scope").as_slice(), &["repo"]);
    let request = server.request();
    assert!(request.contains("grant_type=authorization_code"));
    assert!(request.contains("code=auth-code"));
    assert!(
        request.contains("redirect_uri=https%3A%2F%2Fagentcast.example.test%2Foauth%2Fcallback")
    );
    assert!(request.contains("client_id=agentcast"));
    assert!(request.contains("code_verifier=verifier"));
}

#[test]
fn callback_exchange_consumes_state_and_persists_token_response() {
    let mut service = GatewayOAuthService::new(InMemoryOAuthStore::default());
    service
        .begin_authorization(begin(), &metadata())
        .expect("begin auth");
    let callback = OAuthCallback {
        state: "state-1".to_string(),
        subject: "user-1".to_string(),
        code: "code".to_string(),
    };

    let pending = service
        .begin_callback_exchange(&callback, 50)
        .expect("begin callback exchange");
    service
        .finish_callback_exchange(
            pending,
            OAuthRefreshResult {
                access_token: "access-from-code".to_string(),
                refresh_token: Some("refresh-from-code".to_string()),
                scopes: Some(scopes("repo")),
                expires_at_unix: 5000,
            },
        )
        .expect("finish callback exchange");

    let credential = service
        .credential_for_runtime("user-1", "github")
        .expect("credential")
        .expect("credential");
    assert_eq!(credential.access_token, "access-from-code");
    assert_eq!(
        credential.refresh_token.as_deref(),
        Some("refresh-from-code")
    );
    assert_eq!(credential.scopes.as_slice(), &["repo"]);
    assert_eq!(
        service.begin_callback_exchange(&callback, 50),
        Err(GatewayOAuthError::Auth(OAuthError::StateNotFound))
    );
}

#[test]
fn dynamic_client_registration_persists_in_gateway_store() {
    let mut service = GatewayOAuthService::new(InMemoryOAuthStore::default());
    service
        .put_client_registration(OAuthClientRegistration {
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            client_id: "dynamic-client".to_string(),
            client_secret: Some("dynamic-secret".to_string()),
            client_id_issued_at_unix: Some(100),
            client_secret_expires_at_unix: Some(5000),
        })
        .expect("put registration");

    let registration = service
        .client_registration("user-1", "github")
        .expect("registration")
        .expect("registration");

    assert_eq!(registration.client_id, "dynamic-client");
    assert_eq!(
        registration.client_secret.as_deref(),
        Some("dynamic-secret")
    );
}

#[test]
fn reconcile_upstreams_clears_removed_oauth_state() {
    let mut service = GatewayOAuthService::new(InMemoryOAuthStore::default());
    for upstream_id in ["github", "slack"] {
        service
            .store_mut()
            .put_credential(OAuthCredential {
                subject: "user-1".to_string(),
                upstream_id: upstream_id.to_string(),
                access_token: "access".to_string(),
                refresh_token: None,
                scopes: scopes("repo"),
                expires_at_unix: 100,
                refresh_failed: false,
            })
            .expect("credential");
        service
            .put_client_registration(OAuthClientRegistration {
                subject: "user-1".to_string(),
                upstream_id: upstream_id.to_string(),
                client_id: format!("client-{upstream_id}"),
                client_secret: None,
                client_id_issued_at_unix: None,
                client_secret_expires_at_unix: None,
            })
            .expect("registration");
    }

    let reconciliation = service
        .reconcile_upstreams(
            vec!["github".to_string()],
            vec!["github".to_string(), "slack".to_string()],
        )
        .expect("reconcile");

    assert_eq!(reconciliation.removed_upstream_ids, &["slack"]);
    assert!(
        service
            .credential_for_runtime("user-1", "slack")
            .expect("credential")
            .is_none()
    );
    assert!(
        service
            .client_registration("user-1", "slack")
            .expect("registration")
            .is_none()
    );
    assert!(
        service
            .credential_for_runtime("user-1", "github")
            .expect("credential")
            .is_some()
    );
}

#[test]
fn refresh_can_be_split_around_async_provider_call() {
    let mut service = GatewayOAuthService::new(InMemoryOAuthStore::default());
    service
        .store_mut()
        .put_credential(OAuthCredential {
            subject: "user-1".to_string(),
            upstream_id: "github".to_string(),
            access_token: "old-access".to_string(),
            refresh_token: Some("old-refresh".to_string()),
            scopes: scopes("repo"),
            expires_at_unix: 1000,
            refresh_failed: true,
        })
        .expect("store credential");

    let request = service
        .begin_refresh("user-1", "github", 900)
        .expect("begin refresh")
        .expect("request");
    assert_eq!(request.refresh_token, "old-refresh");
    let status = service
        .finish_refresh_success(
            "user-1",
            "github",
            OAuthRefreshResult {
                access_token: "new-access".to_string(),
                refresh_token: None,
                scopes: None,
                expires_at_unix: 5000,
            },
        )
        .expect("finish refresh");

    assert_eq!(status, OAuthStatus::Connected);
    let credential = service
        .credential_for_runtime("user-1", "github")
        .expect("credential")
        .expect("credential");
    assert_eq!(credential.access_token, "new-access");
    assert_eq!(credential.refresh_token.as_deref(), Some("old-refresh"));
    assert!(!credential.refresh_failed);
}

struct FixtureRefreshProvider {
    result: Result<OAuthRefreshResult, OAuthRefreshProviderError>,
    seen_request: Option<OAuthRefreshRequest>,
}

struct TokenFixtureServer {
    url: String,
    request: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl TokenFixtureServer {
    fn start(response: &'static str) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("listener");
        let url = format!("http://{}/token", listener.local_addr().expect("addr"));
        let request = std::sync::Arc::new(std::sync::Mutex::new(None));
        let request_for_thread = request.clone();
        let thread = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut raw = [0_u8; 4096];
            let read = stream.read(&mut raw).expect("read");
            *request_for_thread.lock().expect("request lock") =
                Some(String::from_utf8_lossy(&raw[..read]).to_string());
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        });
        Self {
            url,
            request,
            thread: Some(thread),
        }
    }

    fn url(&self) -> String {
        self.url.clone()
    }

    fn request(mut self) -> String {
        if let Some(thread) = self.thread.take() {
            thread.join().expect("server thread");
        }
        self.request
            .lock()
            .expect("request lock")
            .clone()
            .expect("request")
    }
}

impl OAuthRefreshProvider for FixtureRefreshProvider {
    fn refresh(
        &mut self,
        request: OAuthRefreshRequest,
    ) -> Result<OAuthRefreshResult, OAuthRefreshProviderError> {
        self.seen_request = Some(request);
        self.result.clone()
    }
}
