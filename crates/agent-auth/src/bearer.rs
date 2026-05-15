use crate::{AuthChallenge, ScopeSet};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use thiserror::Error;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct BearerClaims {
    pub subject: String,
    pub audience: String,
    pub scopes: ScopeSet,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedSubject {
    pub subject: String,
}

pub trait BearerTokenVerifier: Send + Sync {
    fn verify(&self, authorization_header: &str) -> Result<BearerClaims, BearerError>;
}

#[derive(Clone, Debug, Default)]
pub struct FixtureBearerTokenVerifier;

impl BearerTokenVerifier for FixtureBearerTokenVerifier {
    fn verify(&self, authorization_header: &str) -> Result<BearerClaims, BearerError> {
        BearerClaims::from_authorization_header(authorization_header)
    }
}

#[derive(Clone, Debug, Default)]
pub struct StaticBearerTokenVerifier {
    tokens: BTreeMap<String, BearerClaims>,
}

impl StaticBearerTokenVerifier {
    pub fn new(tokens: impl IntoIterator<Item = (impl Into<String>, BearerClaims)>) -> Self {
        Self {
            tokens: tokens
                .into_iter()
                .map(|(token, claims)| (token.into(), claims))
                .collect(),
        }
    }
}

impl BearerTokenVerifier for StaticBearerTokenVerifier {
    fn verify(&self, authorization_header: &str) -> Result<BearerClaims, BearerError> {
        let token = authorization_header
            .strip_prefix("Bearer ")
            .ok_or(BearerError::MissingBearerScheme)?;
        self.tokens
            .get(token)
            .cloned()
            .ok_or(BearerError::InvalidToken)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Jwk {
    pub kty: String,
    pub kid: Option<String>,
    pub alg: Option<String>,
    pub k: Option<String>,
}

#[derive(Clone, Debug)]
pub struct JwtBearerTokenVerifier {
    keys: BTreeMap<Option<String>, Vec<u8>>,
    expected_audience: Option<String>,
}

impl JwtBearerTokenVerifier {
    pub fn from_jwks(jwks: Jwks) -> Result<Self, BearerError> {
        let mut keys = BTreeMap::new();
        for key in jwks.keys {
            if key.kty != "oct" {
                continue;
            }
            if key.alg.as_deref().is_some_and(|alg| alg != "HS256") {
                continue;
            }
            let secret = key
                .k
                .as_deref()
                .ok_or(BearerError::MissingJwkKey)
                .and_then(decode_base64url)?;
            keys.insert(key.kid, secret);
        }
        if keys.is_empty() {
            return Err(BearerError::MissingJwkKey);
        }
        Ok(Self {
            keys,
            expected_audience: None,
        })
    }

    pub fn with_expected_audience(mut self, audience: impl Into<String>) -> Self {
        self.expected_audience = Some(audience.into());
        self
    }
}

impl BearerTokenVerifier for JwtBearerTokenVerifier {
    fn verify(&self, authorization_header: &str) -> Result<BearerClaims, BearerError> {
        let token = authorization_header
            .strip_prefix("Bearer ")
            .ok_or(BearerError::MissingBearerScheme)?;
        let mut parts = token.split('.');
        let encoded_header = parts.next().ok_or(BearerError::MalformedToken)?;
        let encoded_payload = parts.next().ok_or(BearerError::MalformedToken)?;
        let encoded_signature = parts.next().ok_or(BearerError::MalformedToken)?;
        if parts.next().is_some() {
            return Err(BearerError::MalformedToken);
        }

        let header: JwtHeader = serde_json::from_slice(&decode_base64url(encoded_header)?)
            .map_err(|_| BearerError::MalformedToken)?;
        if header.alg != "HS256" {
            return Err(BearerError::UnsupportedJwtAlgorithm);
        }
        let secret = self.select_key(&header.kid)?;
        let signing_input = format!("{encoded_header}.{encoded_payload}");
        let expected_signature = hmac_sha256(secret, signing_input.as_bytes());
        let actual_signature = decode_base64url(encoded_signature)?;
        if expected_signature != actual_signature {
            return Err(BearerError::InvalidToken);
        }

        let payload: JwtPayload = serde_json::from_slice(&decode_base64url(encoded_payload)?)
            .map_err(|_| BearerError::MalformedToken)?;
        let audience = payload
            .primary_audience()
            .ok_or(BearerError::MissingAudience)?;
        if self
            .expected_audience
            .as_deref()
            .is_some_and(|expected| !payload.has_audience(expected))
        {
            return Err(BearerError::InvalidToken);
        }
        let scopes = payload.scopes()?;
        let subject = payload.sub.ok_or(BearerError::MissingSubject)?;
        Ok(BearerClaims {
            subject,
            audience,
            scopes,
        })
    }
}

impl JwtBearerTokenVerifier {
    fn select_key(&self, kid: &Option<String>) -> Result<&[u8], BearerError> {
        if let Some(secret) = self.keys.get(kid) {
            return Ok(secret);
        }
        if kid.is_none() && self.keys.len() == 1 {
            return self
                .keys
                .values()
                .next()
                .map(Vec::as_slice)
                .ok_or(BearerError::MissingJwkKey);
        }
        Err(BearerError::MissingJwkKey)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthDecision {
    Authorized(AuthorizedSubject),
    Unauthorized(AuthChallenge),
    Forbidden(AuthChallenge),
}

impl AuthDecision {
    pub fn authorize_route(
        claims: Option<&BearerClaims>,
        resource_uri: &str,
        resource_metadata: &str,
        required_scopes: &ScopeSet,
    ) -> Self {
        let Some(claims) = claims else {
            return Self::Unauthorized(AuthChallenge::unauthorized(resource_metadata.to_string()));
        };

        if claims.audience != resource_uri {
            return Self::Unauthorized(AuthChallenge::unauthorized(resource_metadata.to_string()));
        }

        if !claims.scopes.contains_all(required_scopes) {
            return Self::Forbidden(AuthChallenge::insufficient_scope(
                resource_metadata.to_string(),
                required_scopes.clone(),
            ));
        }

        Self::Authorized(AuthorizedSubject {
            subject: claims.subject.clone(),
        })
    }

    pub fn is_authorized(&self) -> bool {
        matches!(self, Self::Authorized(_))
    }
}

impl BearerClaims {
    pub fn from_authorization_header(raw: &str) -> Result<Self, BearerError> {
        let token = raw
            .strip_prefix("Bearer ")
            .ok_or(BearerError::MissingBearerScheme)?;
        Self::from_fixture_token(token)
    }

    pub fn from_fixture_token(token: &str) -> Result<Self, BearerError> {
        let mut subject = None;
        let mut audience = None;
        let mut scopes = None;

        for pair in token.split(';') {
            let (key, value) = pair.split_once('=').ok_or(BearerError::MalformedToken)?;
            match key {
                "sub" => subject = Some(value.to_string()),
                "aud" => audience = Some(value.to_string()),
                "scope" => scopes = Some(ScopeSet::parse(value)?),
                _ => {}
            }
        }

        Ok(Self {
            subject: subject.ok_or(BearerError::MissingSubject)?,
            audience: audience.ok_or(BearerError::MissingAudience)?,
            scopes: scopes.unwrap_or_else(ScopeSet::empty),
        })
    }
}

#[derive(Debug, Deserialize)]
struct JwtHeader {
    alg: String,
    kid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JwtPayload {
    sub: Option<String>,
    aud: Option<JwtAudience>,
    scope: Option<String>,
    scp: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum JwtAudience {
    One(String),
    Many(Vec<String>),
}

impl JwtPayload {
    fn primary_audience(&self) -> Option<String> {
        match &self.aud {
            Some(JwtAudience::One(audience)) => Some(audience.clone()),
            Some(JwtAudience::Many(audiences)) => audiences.first().cloned(),
            None => None,
        }
    }

    fn has_audience(&self, expected: &str) -> bool {
        match &self.aud {
            Some(JwtAudience::One(audience)) => audience == expected,
            Some(JwtAudience::Many(audiences)) => {
                audiences.iter().any(|audience| audience == expected)
            }
            None => false,
        }
    }

    fn scopes(&self) -> Result<ScopeSet, BearerError> {
        if let Some(scope) = &self.scope {
            return Ok(ScopeSet::parse(scope)?);
        }
        if let Some(scopes) = &self.scp {
            return Ok(ScopeSet::from_scopes(scopes.iter().cloned())?);
        }
        Ok(ScopeSet::empty())
    }
}

fn decode_base64url(raw: &str) -> Result<Vec<u8>, BearerError> {
    let mut bits = 0_u32;
    let mut bit_count = 0_u8;
    let mut decoded = Vec::new();
    for byte in raw.bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            b'=' => continue,
            _ => return Err(BearerError::MalformedToken),
        } as u32;
        bits = (bits << 6) | value;
        bit_count += 6;
        if bit_count >= 8 {
            bit_count -= 8;
            decoded.push((bits >> bit_count) as u8);
            bits &= (1 << bit_count) - 1;
        }
    }
    Ok(decoded)
}

fn hmac_sha256(secret: &[u8], message: &[u8]) -> Vec<u8> {
    const BLOCK_SIZE: usize = 64;
    let mut key = if secret.len() > BLOCK_SIZE {
        Sha256::digest(secret).to_vec()
    } else {
        secret.to_vec()
    };
    key.resize(BLOCK_SIZE, 0);

    let mut outer_key = [0x5c; BLOCK_SIZE];
    let mut inner_key = [0x36; BLOCK_SIZE];
    for (index, key_byte) in key.iter().enumerate() {
        outer_key[index] ^= key_byte;
        inner_key[index] ^= key_byte;
    }

    let mut inner = Sha256::new();
    inner.update(inner_key);
    inner.update(message);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(outer_key);
    outer.update(inner_hash);
    outer.finalize().to_vec()
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum BearerError {
    #[error("authorization header is missing Bearer scheme")]
    MissingBearerScheme,
    #[error("bearer token is malformed")]
    MalformedToken,
    #[error("bearer token is missing subject")]
    MissingSubject,
    #[error("bearer token is missing audience")]
    MissingAudience,
    #[error("bearer token is invalid")]
    InvalidToken,
    #[error("JWT signing algorithm is not supported")]
    UnsupportedJwtAlgorithm,
    #[error("JWKS does not contain a usable signing key")]
    MissingJwkKey,
    #[error(transparent)]
    Scope(#[from] crate::ScopeError),
}
