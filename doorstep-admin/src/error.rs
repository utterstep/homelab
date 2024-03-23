use axum::response::IntoResponse;
use displaydoc::Display;
use eyre::Report;
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum DoorstepError {
    /// Internal error: {0}
    InternalError(#[from] Report),
    /// Invalid request: {0}
    InvalidRequest(String),
}

impl From<&str> for DoorstepError {
    fn from(message: &str) -> Self {
        Self::InvalidRequest(message.to_string())
    }
}

impl IntoResponse for DoorstepError {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        let message = format!("Error while processing request: {}", self);

        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
    }
}
