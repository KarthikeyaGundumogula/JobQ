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
use tokio::sync::Mutex;
use uuid::{self, Uuid};

mod constants;
mod errors;
mod state;
mod types;
mod worker;

use errors::ApiError;
use state::AppState;
use types::{ApiResponse, Job, JobState, PostJob};

use crate::{constants::MAX_ATTEMPTS, types::Index};

#[tokio::main]
async fn main() {
    let app_state = Arc::new(Mutex::new(AppState {
        jobs: HashMap::new(),
        index: BinaryHeap::new(),
    }));

    for _ in 0..2 {
        let state = app_state.clone();
        tokio::spawn(async move { worker::worker_loop(state).await });
    }

    let app = Router::new()
        .route("/", get(health_check))
        .route("/get/{job_id}", get(get_jobs))
        .route("/post", post(post_job))
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
    State(app): State<Arc<Mutex<AppState>>>,
) -> Result<ApiResponse, ApiError> {
    let id = Uuid::from_str(&job_id)?;

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
) -> Result<ApiResponse, ApiError> {
    let id = Uuid::new_v4();

    let current_time = Utc::now().timestamp();

    if data.max_attempts > MAX_ATTEMPTS {
        Err(ApiError::InvalidArgument)
    } else {
        let mut app_state = app.lock().await;
        app_state.jobs.insert(
            id,
            Job {
                job_id: id,
                job_type: data.job_type,
                payload: data.payload,
                state: JobState::Queued,
                attempts: 0,
                max_attempts: data.max_attempts,
                run_at: current_time,
            },
        );

        app_state.index.push(Reverse(Index {
            run_at: current_time,
            uuid: id,
        }));
        Ok(ApiResponse::Created(id.to_string()))
    }
}
