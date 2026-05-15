use agent_auth::OAuthStatus;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OAuthStatusResponse {
    pub subject: String,
    pub upstream_id: String,
    pub status: OAuthApiStatus,
}

impl OAuthStatusResponse {
    pub fn from_status(subject: &str, upstream_id: &str, status: OAuthStatus) -> Self {
        Self {
            subject: subject.to_string(),
            upstream_id: upstream_id.to_string(),
            status: status.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OAuthApiStatus {
    Disconnected,
    DiscoveryFailed,
    UnsupportedProvider,
    Connected,
    Expiring,
    Expired,
    RefreshFailed,
}

impl From<OAuthStatus> for OAuthApiStatus {
    fn from(status: OAuthStatus) -> Self {
        match status {
            OAuthStatus::Disconnected => Self::Disconnected,
            OAuthStatus::DiscoveryFailed => Self::DiscoveryFailed,
            OAuthStatus::UnsupportedProvider => Self::UnsupportedProvider,
            OAuthStatus::Connected => Self::Connected,
            OAuthStatus::Expiring => Self::Expiring,
            OAuthStatus::Expired => Self::Expired,
            OAuthStatus::RefreshFailed => Self::RefreshFailed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_response_contains_no_tokens() {
        let response =
            OAuthStatusResponse::from_status("user-1", "github", OAuthStatus::RefreshFailed);

        assert_eq!(response.subject, "user-1");
        assert_eq!(response.upstream_id, "github");
        assert_eq!(response.status, OAuthApiStatus::RefreshFailed);
    }
}
