use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum GatewayError {
    #[error("unknown action `{0}`")]
    UnknownAction(String),
    #[error("invalid public host `{0}`")]
    InvalidPublicHost(String),
    #[error("invalid public path `{0}`")]
    InvalidPublicPath(String),
    #[error("duplicate protected route for host `{host}` and path segment `{path}`")]
    DuplicateProtectedRoute { host: String, path: String },
}
