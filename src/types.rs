use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PostJob {
    pub job_type: String,
    pub payload: Value,
    pub max_attemps: u32,
}

#[derive(Serialize, Clone)]
pub struct Job {
    pub job_id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub state: JobState,
    pub attempts: u32,
    pub max_attemps: u32,
    pub run_at: i64,
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub struct Index {
    pub run_at: i64,
    pub uuid: Uuid,
}

#[derive(Serialize, Clone, PartialEq)]
pub enum JobState {
    Queued,
    Running,
    Succeeded,
    Failed,
    Dead,
    Cancelled,
}

pub enum ApiResponse {
    OK,
    Created(String),
    JobData(Job),
}

pub enum ApiError {
    Failed,
    NotFound,
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::OK => (StatusCode::OK).into_response(),
            Self::Created(str) => (StatusCode::CREATED, Json(str)).into_response(),
            Self::JobData(data) => (StatusCode::OK, Json(data)).into_response(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Failed => (StatusCode::BAD_REQUEST).into_response(),
            Self::NotFound => (StatusCode::NOT_FOUND).into_response(),
        }
    }
}
