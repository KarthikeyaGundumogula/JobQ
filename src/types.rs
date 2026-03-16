use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx;
use uuid::Uuid;

use crate::retry::{ExponentialBackoff, LinearBackoff, RetryPolicy};

#[derive(Deserialize, Clone, Serialize)]
#[serde(tag = "type")]
pub enum RetryPolicyConfig {
    Exponential { base: i64, max_delay: i64 },
    Linear { step: i64, max_delay: i64 },
}

impl RetryPolicyConfig {
    pub fn next_delay(&self, attempts: u32) -> i64 {
        match self {
            Self::Exponential { base, max_delay } => ExponentialBackoff {
                base: *base,
                max_delay: *max_delay,
            }
            .next_delay(attempts),
            Self::Linear { step, max_delay } => LinearBackoff {
                step: *step,
                max_delay: *max_delay,
            }
            .next_delay(attempts),
        }
    }
}

#[derive(Deserialize)]
pub struct PostJob {
    pub job_type: String,
    pub payload: Value,
    pub max_attempts: i16,
    pub retry_policy: RetryPolicyConfig,
}

#[derive(sqlx::FromRow, Serialize, Clone)]
pub struct Job {
    pub job_id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub state: JobState,
    pub attempts: i16,
    pub max_attempts: i16,
    pub run_at: chrono::DateTime<chrono::Utc>,
    pub retry_policy: serde_json::Value,
}

#[derive(sqlx::Type,Serialize, Clone, PartialEq, Debug)]
#[sqlx(type_name = "job_state", rename_all = "PascalCase")]
pub enum JobState {
    Queued,
    Running,
    Succeeded,
    Failed,
    Dead,
    Cancelled,
}

impl std::fmt::Display for JobState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            JobState::Queued => write!(f, "Queued"),
            JobState::Running => write!(f, "Running"),
            JobState::Succeeded => write!(f, "Succeeded"),
            JobState::Failed => write!(f, "Failed"),
            JobState::Dead => write!(f, "Dead"),
            JobState::Cancelled => write!(f, "Cancelled"),
        }
    }
}

pub enum ApiResponse {
    OK,
    Created(String),
    JobData(Job),
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
