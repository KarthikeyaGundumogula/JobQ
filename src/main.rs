use std::{collections::HashMap, str::FromStr, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
use uuid::{self, Uuid};

#[derive(Serialize, Clone)]
pub struct Job {
    job_id: Uuid,
    job_type: String,
    payload: serde_json::Value,
    state: JobState,
    attempts: u32,
}

#[derive(Serialize, Clone)]
pub enum JobState {
    Queued,
    Running,
    Succeeded,
    Failed,
    Dead,
    Cancelled,
}

enum ApiResponse {
    OK,
    Created(String),
    JobData(Job),
}

enum ApiError {
    Failed,
    NotFound,
}
pub struct AppState {
    jobs: HashMap<Uuid, Job>,
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

#[tokio::main]
async fn main() {
    let app_state = Arc::new(Mutex::new(AppState {
        jobs: HashMap::new(),
    }));

    let app = Router::new()
        .route("/get/:job_id", get(get_jobs))
        .route("/post", post(post_job))
        .with_state(app_state);

    let listner = tokio::net::TcpListener::bind("0.0.0.0:8484").await.unwrap();
    println!(
        "Server started successfully at {}",
        listner.local_addr().unwrap()
    );
    axum::serve(listner, app).await.unwrap();
}

async fn get_jobs(
    Path(job_id): Path<String>,
    State(app): State<Arc<Mutex<AppState>>>,
) -> Result<ApiResponse, ApiError> {
    let id = match Uuid::from_str(&job_id) {
        Ok(id) => id,
        Err(_) => return Err(ApiError::Failed),
    };

    let job = {
        let state = app.lock().await;
        state.jobs.get(&id).cloned()
    };

    match job {
        Some(job) => Ok(ApiResponse::JobData(job)),
        None => Err(ApiError::NotFound),
    }
}

#[derive(Deserialize)]
pub struct PostJob {
    pub job_type: String,
    pub payload: Value,
}
async fn post_job(
    State(app): State<Arc<Mutex<AppState>>>,
    Json(data): Json<PostJob>,
) -> ApiResponse {
    let id = Uuid::new_v4();
    {
        let mut app_state = app.lock().await;
        app_state.jobs.insert(
            id,
            Job {
                job_id: id,
                job_type: data.job_type,
                payload: data.payload,
                state: JobState::Queued,
                attempts: 0,
            },
        );
    }
    ApiResponse::Created(id.to_string())
}
