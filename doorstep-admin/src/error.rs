use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use displaydoc::Display;
use eyre::Report;
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum DoorstepError {
    /// Not found: {0}
    NotFound(String),
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
    fn into_response(self) -> Response<axum::body::Body> {
        match self {
            Self::NotFound(message) => (StatusCode::NOT_FOUND, message).into_response(),
            Self::InvalidRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            Self::InternalError(error) => {
                let message = format!("Error while processing request: {}", error);

                (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
            }
        }
    }
}
