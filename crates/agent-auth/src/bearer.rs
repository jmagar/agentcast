use crate::{AuthChallenge, ScopeSet};
use serde::{Deserialize, Serialize};
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
    #[error(transparent)]
    Scope(#[from] crate::ScopeError),
}
