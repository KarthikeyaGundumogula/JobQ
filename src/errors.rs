use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
    code: String,
}

impl ErrorResponse {
    pub fn new(error: String, code: StatusCode) -> Self {
        ErrorResponse {
            error,
            code: code.to_string(),
        }
    }
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("UUID Parsing error")]
    Uuid(#[from] uuid::Error),
    // #[error("API Request Failed")]
    // Failed,
    #[error("Job not found")]
    NotFound,
    #[error("Invalid Argument (max_attempts)")]
    InvalidArgument,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            // Self::Failed => (
            //     StatusCode::INTERNAL_SERVER_ERROR,
            //     Json(ErrorResponse::new(
            //         self.to_string(),
            //         StatusCode::INTERNAL_SERVER_ERROR,
            //     )),
            // )
            //     .into_response(),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(self.to_string(), StatusCode::NOT_FOUND)),
            )
                .into_response(),
            Self::InvalidArgument => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ErrorResponse::new(
                    self.to_string(),
                    StatusCode::UNPROCESSABLE_ENTITY,
                )),
            )
                .into_response(),
            Self::Uuid(err) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(err.to_string(), StatusCode::BAD_REQUEST)),
            )
                .into_response(),
        }
    }
}
