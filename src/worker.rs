use chrono::{self, Duration as CDuration, Utc};
use std::{sync::Arc, time::Duration};
use tokio::{select, time::sleep};
use uuid::Uuid;

use crate::{
    constants::LEASE_PERIOD,
    db,
    state::AppState,
    types::{Job, JobState, RetryPolicyConfig},
};

pub async fn worker_loop(state: Arc<AppState>, worker_id: Uuid) {
    loop {
        //-- Infinitely running worker loop --//
        // 1. get the most recent job
        // | -- Exists change the state to running
        // | -- not exists still get the job with nearest exeecution time and put a select! on notify and remaining duration for the top task to run
        let job: Option<Job> = {
            let lease_expires_at = Utc::now() + CDuration::seconds(LEASE_PERIOD);
            let db_res = db::claim_or_peek_job(&state.pool, worker_id, lease_expires_at).await;
            match db_res {
                Ok(res) => match res {
                    Some(job) => {
                        let run_at = job.run_at;
                        let now = Utc::now();
                        if run_at > now {
                            select! {
                                _ = state.notify.notified() => {continue},
                                _ = sleep(Duration::from_secs((run_at.timestamp()-now.timestamp() ) as u64)) => {continue}
                            };
                        } else {
                            Some(job)
                        }
                    }
                    None => None,
                },
                Err(e) => {
                    eprintln!("worker loop failed at db query, {}", e);
                    sleep(Duration::from_secs(2)).await;
                    None
                }
            }
        };

        if let Some(job) = job {
            println!("Processing the job with jobID: {}", job.job_id);

            sleep(Duration::from_secs(2)).await;
            let choice: u8 = rand::random();

            if choice.is_multiple_of(2) {
                println!("job witrh jobId: {} failed ", job.job_id);
                log_if_err(
                    db::update_job_status_by_id(
                        &state.pool,
                        job.job_id,
                        JobState::Failed,
                        JobState::Running,
                    )
                    .await,
                    "unable to update job status to failed",
                );

                if job.attempts < job.max_attempts {
                    let retry_policy = match RetryPolicyConfig::try_from(job.retry_policy.clone()) {
                        Ok(p) => p,
                        Err(_) => {
                            eprintln!("unable to parse the retry policy");
                            continue;
                        }
                    };
                    let delay = retry_policy.next_delay(job.attempts);
                    let new_run_at = Utc::now() + CDuration::seconds(delay);

                    log_if_err(
                        db::update_job_for_retry_by_id(&state.pool, job.job_id, new_run_at).await,
                        "unable to update job for retry",
                    );
                } else {
                    log_if_err(
                        db::update_job_status_by_id(
                            &state.pool,
                            job.job_id,
                            JobState::Dead,
                            JobState::Failed,
                        )
                        .await,
                        "unable to update job status to Dead",
                    );
                }
            } else {
                log_if_err(
                    db::update_job_status_by_id(
                        &state.pool,
                        job.job_id,
                        JobState::Succeeded,
                        JobState::Running,
                    )
                    .await,
                    "unable to update jub status to succeeded",
                );
            }
        } else {
            state.notify.notified().await;
        }
    }
}

fn log_if_err<T>(result: Result<T, impl std::fmt::Debug>, context: &str) {
    if let Err(e) = result {
        eprintln!("[worker] {}: {:?}", context, e);
    }
}

pub async fn requeue_worker(state: Arc<AppState>) {

}