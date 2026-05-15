use agent_auth::{AuthDecision, BearerClaims, ProtectedResourceMetadata};
use agent_gateway::{ProtectedRouteIndex, ProtectedRouteTarget};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct ProtectedMcpRouteApi {
    routes: ProtectedRouteIndex,
}

impl ProtectedMcpRouteApi {
    pub fn new(routes: ProtectedRouteIndex) -> Self {
        Self { routes }
    }

    pub fn handle(&self, request: ProtectedMcpRequest) -> ProtectedMcpResponse {
        if let Some(route) = self.routes.resolve_metadata(&request.host, &request.path) {
            return ProtectedMcpResponse::Metadata {
                status: ResponseStatus::Ok,
                metadata: route.protected_resource_metadata(),
            };
        }

        let Some(route) = self.routes.resolve(&request.host, &request.path) else {
            return ProtectedMcpResponse::NotFound {
                status: ResponseStatus::NotFound,
            };
        };

        let claims = request
            .authorization
            .as_deref()
            .and_then(|header| BearerClaims::from_authorization_header(header).ok());

        match route.authorize(claims.as_ref(), &request.public_origin) {
            AuthDecision::Authorized(subject) => ProtectedMcpResponse::DispatchAllowed {
                status: ResponseStatus::Accepted,
                subject: subject.subject,
                target: route.target.clone(),
            },
            AuthDecision::Unauthorized(challenge) => ProtectedMcpResponse::Challenge {
                status: ResponseStatus::Unauthorized,
                www_authenticate: challenge.www_authenticate(),
            },
            AuthDecision::Forbidden(challenge) => ProtectedMcpResponse::Challenge {
                status: ResponseStatus::Forbidden,
                www_authenticate: challenge.www_authenticate(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedMcpRequest {
    pub host: String,
    pub path: String,
    pub public_origin: String,
    pub authorization: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtectedMcpResponse {
    Metadata {
        status: ResponseStatus,
        metadata: ProtectedResourceMetadata,
    },
    DispatchAllowed {
        status: ResponseStatus,
        subject: String,
        target: ProtectedRouteTarget,
    },
    Challenge {
        status: ResponseStatus,
        www_authenticate: String,
    },
    NotFound {
        status: ResponseStatus,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseStatus {
    Ok,
    Accepted,
    Unauthorized,
    Forbidden,
    NotFound,
}
