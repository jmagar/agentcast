use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("unknown action `{0}`")]
    UnknownAction(String),
}
