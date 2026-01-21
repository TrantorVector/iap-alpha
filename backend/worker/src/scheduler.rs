use anyhow::Result;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time;
use tracing::{info, error};
use crate::jobs::Job;

pub struct Scheduler {
    pool: PgPool,
    jobs: Vec<Box<dyn Job>>,
}

impl Scheduler {
    pub fn new(pool: PgPool, jobs: Vec<Box<dyn Job>>) -> Self {
        Self { pool, jobs }
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting worker scheduler...");
        info!("Schedule: running all jobs every 15 minutes");

        loop {
            info!("--- Starting scheduled job run ---");
            for job in &self.jobs {
                info!("Running job: {}", job.name());
                if let Err(e) = job.run(&self.pool).await {
                    error!("Job {} failed: {:?}", job.name(), e);
                } else {
                    info!("Job {} completed successfully", job.name());
                }
            }
            info!("--- Scheduled run complete ---");

            time::sleep(Duration::from_secs(15 * 60)).await;
        }
    }
}
