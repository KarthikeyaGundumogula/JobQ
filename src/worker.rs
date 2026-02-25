use chrono::{self, Utc};
use std::{cmp::Reverse, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};
use uuid::Uuid;

use crate::{state::AppState, types::JobState};

pub async fn worker_loop(state: Arc<Mutex<AppState>>) {
    loop {
        let now = Utc::now().timestamp();
        // get the job id that needs to be runninng
        let job_id: Option<Uuid> = {
            let mut app = state.lock().await;
            // find a job that is currently QUEUED and from the top of the queue
            if let Some(index) = app.index.peek() {
                let uuid = index.0.uuid;
                let run_at = index.0.run_at;
                if let Some(job) = app.jobs.get_mut(&uuid) {
                    if job.run_at <= now && job.state == JobState::Queued && job.run_at == run_at {
                        job.state = JobState::Running;
                        job.attempts += 1;
                        app.index.pop();
                        drop(app);
                        Some(uuid)
                    } else {
                        if job.run_at != run_at {
                            app.index.pop();
                            drop(app);
                            None
                        } else {
                            let run_at = job.run_at;
                            drop(app);
                            if run_at > now {
                                sleep(Duration::from_secs((run_at - now) as u64)).await;
                            }
                            None
                        }
                    }
                } else {
                    app.index.pop();
                    None
                }
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
                        let run = job.run_at;
                        let uuid = job.job_id;
                        app.index.push(Reverse(crate::types::Index {
                            run_at: run,
                            uuid: uuid,
                        }));
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
