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

#[test]
fn static_token_verifier_maps_opaque_token_to_claims() {
    let verifier = StaticBearerTokenVerifier::new([(
        "opaque-token",
        BearerClaims {
            subject: "user-1".to_string(),
            audience: "https://mcp.example.test/syslog".to_string(),
            scopes: scopes("mcp:read"),
        },
    )]);

    let claims = verifier.verify("Bearer opaque-token").expect("claims");

    assert_eq!(claims.subject, "user-1");
    assert_eq!(
        verifier.verify("Bearer wrong").unwrap_err(),
        BearerError::InvalidToken
    );
}

#[test]
fn jwt_verifier_accepts_hs256_jwks_tokens() {
    let verifier = JwtBearerTokenVerifier::from_jwks(Jwks {
        keys: vec![Jwk {
            kty: "oct".to_string(),
            kid: Some("fixture-key".to_string()),
            alg: Some("HS256".to_string()),
            k: Some(base64url_encode(b"super-secret")),
        }],
    })
    .expect("verifier")
    .with_expected_audience("https://mcp.example.test/syslog");
    let token = signed_jwt(
        "fixture-key",
        b"super-secret",
        serde_json::json!({
            "sub": "user-1",
            "aud": ["https://mcp.example.test/syslog", "other"],
            "scope": "mcp:read mcp:write"
        }),
    );

    let claims = verifier
        .verify(&format!("Bearer {token}"))
        .expect("valid token");

    assert_eq!(claims.subject, "user-1");
    assert_eq!(claims.audience, "https://mcp.example.test/syslog");
    assert_eq!(claims.scopes.as_slice(), &["mcp:read", "mcp:write"]);
}

#[test]
fn jwt_verifier_rejects_tampered_payload() {
    let verifier = JwtBearerTokenVerifier::from_jwks(Jwks {
        keys: vec![Jwk {
            kty: "oct".to_string(),
            kid: Some("fixture-key".to_string()),
            alg: Some("HS256".to_string()),
            k: Some(base64url_encode(b"super-secret")),
        }],
    })
    .expect("verifier");
    let token = signed_jwt(
        "fixture-key",
        b"super-secret",
        serde_json::json!({
            "sub": "user-1",
            "aud": "https://mcp.example.test/syslog",
            "scp": ["mcp:read"]
        }),
    );
    let mut parts = token.split('.').map(str::to_string).collect::<Vec<_>>();
    parts[1].push('A');
    let token = parts.join(".");

    assert_eq!(
        verifier.verify(&format!("Bearer {token}")).unwrap_err(),
        BearerError::InvalidToken
    );
}

#[test]
fn jwt_verifier_rejects_wrong_audience() {
    let verifier = JwtBearerTokenVerifier::from_jwks(Jwks {
        keys: vec![Jwk {
            kty: "oct".to_string(),
            kid: Some("fixture-key".to_string()),
            alg: Some("HS256".to_string()),
            k: Some(base64url_encode(b"super-secret")),
        }],
    })
    .expect("verifier")
    .with_expected_audience("https://mcp.example.test/syslog");
    let token = signed_jwt(
        "fixture-key",
        b"super-secret",
        serde_json::json!({
            "sub": "user-1",
            "aud": "https://wrong.example.test/syslog",
            "scope": "mcp:read"
        }),
    );

    assert_eq!(
        verifier.verify(&format!("Bearer {token}")).unwrap_err(),
        BearerError::InvalidToken
    );
}

fn signed_jwt(kid: &str, secret: &[u8], payload: serde_json::Value) -> String {
    let header = base64url_encode(
        serde_json::json!({
            "alg": "HS256",
            "typ": "JWT",
            "kid": kid
        })
        .to_string()
        .as_bytes(),
    );
    let payload = base64url_encode(payload.to_string().as_bytes());
    let signing_input = format!("{header}.{payload}");
    let signature = base64url_encode(&hmac_sha256(secret, signing_input.as_bytes()));
    format!("{signing_input}.{signature}")
}

fn base64url_encode(raw: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut encoded = String::new();
    for chunk in raw.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);
        let value = ((first as u32) << 16) | ((second as u32) << 8) | third as u32;
        encoded.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
        encoded.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(TABLE[((value >> 6) & 0x3f) as usize] as char);
        }
        if chunk.len() > 2 {
            encoded.push(TABLE[(value & 0x3f) as usize] as char);
        }
    }
    encoded
}
