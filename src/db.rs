use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::ApiError,
    types::{Job, JobState, PostJob},
};

pub async fn get_job_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Job>, ApiError> {
    Ok(sqlx::query_as!(
        Job,
        r#"SELECT job_id, job_type, payload, state AS "state: JobState", attempts, max_attempts, run_at, retry_policy FROM jobq WHERE job_id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn create_job(pool: &PgPool, data: &PostJob, id: Uuid) -> Result<Uuid, ApiError> {
    let current_time = Utc::now();
    let retry_policy = serde_json::to_value(&data.retry_policy)?;
    Ok(
    sqlx::query_scalar!(
      r#"INSERT INTO jobq (job_id, job_type, payload, state, attempts, max_attempts, run_at, retry_policy)
       VALUES ($1,$2,$3,$4,0,$5,$6,$7) RETURNING job_id"#,id,data.job_type,data.payload,JobState::Queued as JobState,data.max_attempts,current_time,retry_policy)
       .fetch_one(pool)
       .await?
  )
}

pub async fn update_job_status_by_id(
    pool: &PgPool,
    id: Uuid,
    new_state: JobState,
    required_state: JobState,
) -> Result<Option<Job>, ApiError> {
    Ok(sqlx::query_as!(
      Job,
        r#"UPDATE jobq SET state = $1 WHERE job_id = $2 AND state = $3 RETURNING job_id, job_type, payload, state AS "state: JobState", attempts, max_attempts, run_at, retry_policy"#,
        new_state as JobState,
        id,
        required_state as JobState
    )
    .fetch_optional(pool)
    .await?)
}
