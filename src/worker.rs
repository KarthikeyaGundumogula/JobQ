use chrono::{self, Utc};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};
use uuid::Uuid;

use crate::{state::AppState, types::JobState};

pub async fn worker_loop(state: Arc<Mutex<AppState>>) {
  loop {
      let now = Utc::now().timestamp();
        // get the job id that needs to be runninng
        let job_id: Option<Uuid> = {
            let mut app = state.lock().await;
            // find a job that is currently QUEUED
            if let Some((id, job)) = app
                .jobs
                .iter_mut()
                .find(|(_, job)| matches!(job.state, JobState::Queued) && now > job.run_at)
            {
                job.state = JobState::Running;
                job.attempts += 1;
                Some(*id)
            } else {
                None
            }
        };
        // run the job
        if let Some(id) = job_id {
            println!("Processing the job with jobID: {}", id);

            // Simulation
            sleep(Duration::from_secs(2)).await;

            // acquire lock
            let mut app = state.lock().await;
            if let Some(job) = app.jobs.get_mut(&id) {
                let choice: u8 = rand::random();
                // Randomly change the job state
                if choice % 2 == 0 {
                  // this mean the simulation is failed 
                    if job.attempts < job.max_attemps {
                        job.state = JobState::Queued;
                        // Increment the run at constrait so that it wont run repeatedly
                        job.run_at = Utc::now().timestamp() + 60;
                    } else {
                        job.state = JobState::Dead;
                    }
                } else {
                    job.state = JobState::Succeeded
                }
            }
        } else {
            // if there is no job is running currently
            sleep(Duration::from_millis(500)).await
        }
    }
}
