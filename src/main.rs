use std::{collections::HashMap, str::FromStr, sync::Arc};
mod types;
use types::{ApiError, ApiResponse, Job, JobState};

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use tokio::sync::Mutex;
use uuid::{self, Uuid};

use crate::types::PostJob;

pub struct AppState {
    jobs: HashMap<Uuid, Job>,
}

#[tokio::main]
async fn main() {
    let app_state = Arc::new(Mutex::new(AppState {
        jobs: HashMap::new(),
    }));

    let app = Router::new()
        .route("/get/{job_id}", get(get_jobs))
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
