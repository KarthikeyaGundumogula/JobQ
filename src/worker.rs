use chrono::{self, Utc};
use std::{cmp::Reverse, sync::Arc, time::Duration};
use tokio::{select, time::sleep};
use uuid::Uuid;

use crate::{state::AppState, types::JobState};

pub async fn worker_loop(state: Arc<AppState>) {
    loop {
        let now = Utc::now().timestamp();

        let job_id: Option<Uuid> = {
            let mut index = state.index.lock().await;

            if let Some(index_item) = index.peek() {
                let uuid = index_item.0.uuid;
                let run_at = index_item.0.run_at;
                let mut jobs = state.jobs.lock().await;
                if let Some(job) = jobs.get_mut(&uuid) {
                    if job.run_at <= now && job.state == JobState::Queued && job.run_at == run_at {
                        job.state = JobState::Running;
                        job.attempts += 1;
                        index.pop();
                        Some(uuid)
                    } else if job.run_at != run_at {
                        index.pop();
                        None
                    } else {
                        let run_at = job.run_at;
                        drop(index);
                        drop(jobs);
                        if run_at > now {
                            select! {
                                _ = state.notify.notified() => {},
                                _ = sleep(Duration::from_secs((run_at - now) as u64)) => {}
                            }
                        }
                        None
                    }
                } else {
                    index.pop();
                    None
                }
            }
             else {
                None
            }
        };

        if let Some(id) = job_id {
            println!("Processing the job with jobID: {}", id);

            sleep(Duration::from_secs(2)).await;

            let now = Utc::now().timestamp();
            let mut jobs = state.jobs.lock().await;
            if let Some(job) = jobs.get_mut(&id) {
                let choice: u8 = rand::random();

                if choice.is_multiple_of(2) {
                    println!("job witrh jobId: {} failed ", job.job_id);

                    if job.attempts < job.max_attempts {
                        job.state = JobState::Queued;
                        let delay = job.retry_policy.next_delay(job.attempts);
                        job.run_at = now + delay;
                        let run = job.run_at;
                        let uuid = job.job_id;
                        drop(jobs);
                        let mut index = state.index.lock().await;
                        index.push(Reverse(crate::types::Index { run_at: run, uuid }));
                    } else {
                        job.state = JobState::Dead;
                    }
                } else {
                    println!("job witrh jobId: {} succeeded", job.job_id);
                    job.state = JobState::Succeeded
                }
            }
        }
        else {
            state.notify.notified().await;
        }
    }
}
