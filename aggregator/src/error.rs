use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error as ThisError;
use validator::ValidationErrors;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Resource not found")]
    NotFound,
    #[error("Internal server error")]
    #[allow(clippy::enum_variant_names)]
    InternalServerError,
    #[error("Invalid Json Request: {0}")]
    JsonRejection(#[from] JsonRejection),
    #[error("Validation Error: {0}")]
    Validation(#[from] ValidationErrors),
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            Self::JsonRejection(rejection) => (rejection.status(), rejection.body_text()),
            Self::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };
        (status, Json(ErrorResponse { message })).into_response()
    }
}
