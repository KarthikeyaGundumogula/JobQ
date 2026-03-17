use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::ApiError,
    types::{Job, JobState, PostJob},
};

pub async fn get_job_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Job>, ApiError> {
    Ok(sqlx::query_as!(
        Job,
        r#"SELECT job_id, job_type, payload, state AS "state: JobState", attempts, max_attempts, run_at, retry_policy, locked_by, lease_expires_at FROM jobq WHERE job_id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn create_job(pool: &PgPool, data: &PostJob, id: Uuid) -> Result<Uuid, ApiError> {
    let retry_policy = serde_json::to_value(&data.retry_policy)?;
    Ok(
    sqlx::query_scalar!(
      r#"
      INSERT INTO jobq (job_id, job_type, payload, state, attempts, max_attempts, run_at, retry_policy, locked_by, lease_expires_at)
      VALUES ($1,$2,$3,$4,0,$5,NOW(),$6,NULL,NULL) RETURNING job_id
       "#
       ,id,data.job_type,data.payload,JobState::Queued as JobState,data.max_attempts,retry_policy)
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
        r#"UPDATE jobq SET state = $1 WHERE job_id = $2 AND state = $3 RETURNING job_id, job_type, payload, state AS "state: JobState", attempts, max_attempts, run_at, retry_policy, locked_by, lease_expires_at"#,
        new_state as JobState,
        id,
        required_state as JobState
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn update_job_for_retry_by_id(
    pool: &PgPool,
    id: Uuid,
    new_run_at: chrono::DateTime<chrono::Utc>,
) -> Result<Job, ApiError> {
    Ok(
        sqlx::query_as!(
            Job,
            r#"UPDATE jobq SET state = $1, run_at = $2 WHERE job_id = $3 RETURNING job_id, job_type, payload, state AS "state: JobState", attempts, max_attempts, run_at, retry_policy, locked_by, lease_expires_at"#,
            JobState::Queued as JobState,
            new_run_at,
            id
        ).fetch_one(pool).await?
    )
}

pub async fn claim_or_peek_job(
    pool: &PgPool,
    worker_id: Uuid,
    lease_expires_at: chrono::DateTime<chrono::Utc>,
) -> Result<Option<Job>, ApiError> {
    let mut tx = pool.begin().await?;
    let job = sqlx::query_as!(Job,
            r#"SELECT job_id, job_type, payload, state AS "state: JobState", attempts, max_attempts, run_at, retry_policy, locked_by, lease_expires_at FROM jobq WHERE state = $1 AND run_at<= NOW()
            ORDER BY run_at LIMIT 1 FOR UPDATE SKIP LOCKED;
            "#,
            JobState::Queued as JobState,
        )
        .fetch_optional(&mut *tx).await?;
    let job = match job {
        Some(job) => {
sqlx::query_as!(
            Job,
            r#"UPDATE jobq SET state = $1, attempts = attempts+1,locked_by = $2,lease_expires_at=$3 WHERE job_id = $4 RETURNING job_id, job_type, payload, state AS "state: JobState", attempts, max_attempts, run_at, retry_policy, locked_by, lease_expires_at"#,
            JobState::Running as JobState,
            worker_id,
            lease_expires_at,
            job.job_id,
        ).fetch_optional(&mut *tx).await?
        }
        None => {
sqlx::query_as!(
                Job,
                r#"SELECT job_id, job_type, payload, state AS "state: JobState", attempts, max_attempts, run_at, retry_policy, locked_by, lease_expires_at FROM jobq WHERE state = $1 ORDER BY run_at LIMIT 1"#,
                JobState::Queued as JobState
            ).fetch_optional(&mut *tx).await?
        }
    };
    tx.commit().await?;
    Ok(job)
}
