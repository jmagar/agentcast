use super::*;

fn scopes(raw: &str) -> ScopeSet {
    ScopeSet::parse(raw).expect("valid scopes")
}

#[test]
fn missing_bearer_returns_unauthorized_challenge() {
    let decision = AuthDecision::authorize_route(
        None,
        "https://mcp.example.test/syslog",
        "https://mcp.example.test/.well-known/oauth-protected-resource/syslog",
        &scopes("mcp:read"),
    );

    let AuthDecision::Unauthorized(challenge) = decision else {
        panic!("expected unauthorized");
    };
    assert!(challenge.www_authenticate().contains("resource_metadata="));
}

#[test]
fn invalid_audience_is_unauthorized() {
    let claims = BearerClaims {
        subject: "user-1".to_string(),
        audience: "https://wrong.example.test/syslog".to_string(),
        scopes: scopes("mcp:read"),
    };

    let decision = AuthDecision::authorize_route(
        Some(&claims),
        "https://mcp.example.test/syslog",
        "https://mcp.example.test/.well-known/oauth-protected-resource/syslog",
        &scopes("mcp:read"),
    );

    assert!(matches!(decision, AuthDecision::Unauthorized(_)));
}

#[test]
fn missing_scope_is_forbidden_with_insufficient_scope() {
    let claims = BearerClaims {
        subject: "user-1".to_string(),
        audience: "https://mcp.example.test/syslog".to_string(),
        scopes: scopes("mcp:read"),
    };

    let decision = AuthDecision::authorize_route(
        Some(&claims),
        "https://mcp.example.test/syslog",
        "https://mcp.example.test/.well-known/oauth-protected-resource/syslog",
        &scopes("mcp:write"),
    );

    let AuthDecision::Forbidden(challenge) = decision else {
        panic!("expected forbidden");
    };
    assert_eq!(challenge.error(), Some("insufficient_scope"));
    assert!(challenge.www_authenticate().contains("scope=\"mcp:write\""));
}

#[test]
fn matching_audience_and_scope_authorizes_subject() {
    let claims = BearerClaims {
        subject: "user-1".to_string(),
        audience: "https://mcp.example.test/syslog".to_string(),
        scopes: scopes("mcp:read mcp:write"),
    };

    let decision = AuthDecision::authorize_route(
        Some(&claims),
        "https://mcp.example.test/syslog",
        "https://mcp.example.test/.well-known/oauth-protected-resource/syslog",
        &scopes("mcp:read"),
    );

    assert_eq!(
        decision,
        AuthDecision::Authorized(AuthorizedSubject {
            subject: "user-1".to_string()
        })
    );
}

#[test]
fn parses_fixture_authorization_header_without_logging_token() {
    let claims = BearerClaims::from_authorization_header(
        "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read",
    )
    .expect("fixture claims");

    assert_eq!(claims.subject, "user-1");
    assert_eq!(claims.scopes.as_slice(), &["mcp:read"]);
}
