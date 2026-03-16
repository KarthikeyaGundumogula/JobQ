use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use chrono::Utc;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    str::FromStr,
    sync::Arc,
};
use tokio::sync::{Mutex, Notify};
use uuid::{self, Uuid};

mod constants;
mod errors;
mod retry;
mod state;
mod types;
mod worker;

use errors::ApiError;
use state::AppState;
use types::{ApiResponse, Job, JobState, PostJob};

use crate::{constants::MAX_ATTEMPTS, types::Index};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let app_state = Arc::new(AppState {
        jobs: Mutex::new(HashMap::new()),
        index: Mutex::new(BinaryHeap::new()),
        notify: Notify::new(),
    });

    for _ in 0..2 {
        let state = app_state.clone();
        tokio::spawn(async move { worker::worker_loop(state).await });
    }

    let app = Router::new()
        .route("/", get(health_check))
        .route("/get/{job_id}", get(get_jobs))
        .route("/post", post(post_job))
        .route("/cancel/{job_id}", post(cancel_job))
        .with_state(app_state);

    let listner = tokio::net::TcpListener::bind("127.0.0.1:8484")
        .await
        .unwrap();
    println!(
        "Server started successfully at {}",
        listner.local_addr().unwrap()
    );
    axum::serve(listner, app).await.unwrap();
}

async fn health_check() -> ApiResponse {
    ApiResponse::OK
}

async fn get_jobs(
    Path(job_id): Path<String>,
    State(app): State<Arc<AppState>>,
) -> Result<ApiResponse, ApiError> {
    let id = Uuid::from_str(&job_id)?;

    let job = {
        let jobs = app.jobs.lock().await;
        jobs.get(&id).cloned()
    };

    match job {
        Some(job) => Ok(ApiResponse::JobData(job)),
        None => Err(ApiError::NotFound),
    }
}

async fn post_job(
    State(app): State<Arc<AppState>>,
    Json(data): Json<PostJob>,
) -> Result<ApiResponse, ApiError> {
    let id = Uuid::new_v4();

    let current_time = Utc::now().timestamp();

    if data.max_attempts > MAX_ATTEMPTS {
        Err(ApiError::InvalidArgument)
    } else {
        {
            let mut index = app.index.lock().await;
            let mut jobs = app.jobs.lock().await;
            jobs.insert(
                id,
                Job {
                    job_id: id,
                    job_type: data.job_type,
                    payload: data.payload,
                    state: JobState::Queued,
                    attempts: 0,
                    max_attempts: data.max_attempts,
                    run_at: current_time,
                    retry_policy: data.retry_policy,
                },
            );

            index.push(Reverse(Index {
                run_at: current_time,
                uuid: id,
            }));
        }
        app.notify.notify_one();
        Ok(ApiResponse::Created(id.to_string()))
    }
}

async fn cancel_job(
    Path(job_id): Path<String>,
    State(app): State<Arc<AppState>>,
) -> Result<ApiResponse, ApiError> {
    let id = Uuid::from_str(&job_id)?;
    let mut jobs = app.jobs.lock().await;
    let job = jobs.get_mut(&id);
    match job {
        Some(job) => match job.state {
            JobState::Queued => {
                job.state = JobState::Cancelled;
                Ok(ApiResponse::JobData(job.clone()))
            }
            _ => Err(ApiError::Conflict {
                reason: job.state.clone(),
            }),
        },
        _ => Err(ApiError::NotFound),
    }
}
