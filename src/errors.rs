use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;

use crate::types::JobState;

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
    #[error("job can't be cancelled at the current state: {reason}")]
    Conflict { reason: JobState },
    #[error("Postgres Error")]
    Database(#[from] sqlx::Error),
    #[error("Serailization error")]
    Serialization(#[from] serde_json::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let message = self.to_string();
        match self {
            // Self::Failed => (
            //     StatusCode::INTERNAL_SERVER_ERROR,
            //     Json(ErrorResponse::new(
            //         message,
            //         StatusCode::INTERNAL_SERVER_ERROR,
            //     )),
            // )
            //     .into_response(),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(message, StatusCode::NOT_FOUND)),
            )
                .into_response(),
            Self::InvalidArgument => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ErrorResponse::new(
                    message,
                    StatusCode::UNPROCESSABLE_ENTITY,
                )),
            )
                .into_response(),
            Self::Uuid(err) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(err.to_string(), StatusCode::BAD_REQUEST)),
            )
                .into_response(),
            Self::Conflict { reason: _ } => (
                StatusCode::CONFLICT,
                Json(ErrorResponse::new(message, StatusCode::CONFLICT)),
            )
                .into_response(),
            Self::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    message,
                    StatusCode::INTERNAL_SERVER_ERROR,
                )),
            )
                .into_response(),
            Self::Serialization(_) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "unable to serailize".to_string(),
                    StatusCode::BAD_REQUEST,
                )),
            )
                .into_response(),
        }
    }
}
