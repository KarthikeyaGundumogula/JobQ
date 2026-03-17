use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use sqlx::postgres::PgPoolOptions;
use std::{str::FromStr, sync::Arc};
use tokio::sync::Notify;
use uuid::{self, Uuid};

mod constants;
mod db;
mod errors;
mod retry;
mod state;
mod types;
mod worker;

use errors::ApiError;
use state::AppState;
use types::{ApiResponse, JobState, PostJob};

use crate::constants::MAX_ATTEMPTS;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("database url must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("failed to connect to the database");
    let app_state = Arc::new(AppState {
        notify: Notify::new(),
        pool,
    });

    for _ in 0..2 {
        let state = app_state.clone();
        tokio::spawn(async move { worker::worker_loop(state).await });
    }

    let app = Router::new()
        .route("/", get(health_check))
        .route("/jobs", post(post_job))
        .route("/jobs/{job_id}", get(get_jobs))
        .route("/jobs/{job_id}/cancel", post(cancel_job))
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

    let job = db::get_job_by_id(&app.pool, id).await?;
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
    if data.max_attempts > MAX_ATTEMPTS {
        Err(ApiError::InvalidArgument)
    } else {
        let id = db::create_job(&app.pool, &data, id).await?;
        app.notify.notify_one();
        Ok(ApiResponse::Created(id.to_string()))
    }
}

async fn cancel_job(
    Path(job_id): Path<String>,
    State(app): State<Arc<AppState>>,
) -> Result<ApiResponse, ApiError> {
    let id = Uuid::from_str(&job_id)?;
    let job =
        db::update_job_status_by_id(&app.pool, id, JobState::Cancelled, JobState::Queued).await?;
    match job {
        Some(job) => Ok(ApiResponse::JobData(job.clone())),
        None => {
            let job = db::get_job_by_id(&app.pool, id).await?;
            match job {
                Some(job) => Err(ApiError::Conflict { reason: job.state }),
                None => Err(ApiError::NotFound),
            }
        }
    }
}
