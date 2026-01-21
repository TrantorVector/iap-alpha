use anyhow::{Context, Result};
use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tracing::{error, info};

use providers::mock::MockMarketDataProvider;
use std::sync::Arc;
use worker::jobs::{
    DocumentRefresh, EarningsPollingJob, FxRefresh, Job, MetricsRecalculationJob, PriceRefreshJob,
};
use worker::scheduler::Scheduler;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run a specific job by name
    #[arg(long)]
    job: Option<String>,

    /// Run all jobs immediately once
    #[arg(long)]
    all: bool,

    /// Run all jobs immediately once (alias for --all)
    #[arg(long)]
    run_now: bool,

    /// Run in scheduler mode (continuous loop)
    #[arg(long)]
    schedule: bool,
}

#[derive(Debug)]
struct Config {
    database_url: String,
}

impl Config {
    fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
        Ok(Self { database_url })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file (try from root first, then current)
    if let Err(_) = dotenvy::from_path("../.env") {
        dotenvy::dotenv().ok();
    }

    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // We only need config and DB if we are running jobs or scheduler
    if !args.all && args.job.is_none() && !args.run_now && !args.schedule {
        use clap::CommandFactory;
        Args::command().print_help()?;
        return Ok(());
    }

    let config = Config::from_env().context("Failed to load configuration")?;

    info!("Starting worker...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .context("Failed to connect to database")?;

    // Initialize provider (using Mock for now as per build plan context)
    let provider = Arc::new(MockMarketDataProvider::new());

    // Helper to create job list
    let create_jobs = |pool: &sqlx::PgPool| -> Vec<Box<dyn Job>> {
        vec![
            Box::new(EarningsPollingJob::new(pool.clone(), provider.clone())),
            Box::new(PriceRefreshJob::new(pool.clone(), provider.clone())),
            Box::new(FxRefresh),
            Box::new(DocumentRefresh),
            Box::new(MetricsRecalculationJob),
        ]
    };

    let jobs = create_jobs(&pool);
    let mut failed = false;

    if args.schedule {
        let scheduler = Scheduler::new(pool.clone(), jobs);
        scheduler.run().await?;
    } else if args.all || args.run_now {
        info!("Running all jobs immediately...");
        for job in jobs {
            info!("Running job: {}", job.name());
            if let Err(e) = job.run(&pool).await {
                error!("Job {} failed: {:?}", job.name(), e);
                failed = true;
            } else {
                info!("Job {} completed successfully", job.name());
            }
        }
    } else if let Some(job_name) = args.job {
        let job = jobs
            .into_iter()
            .find(|j| j.name() == job_name)
            .context(format!("Job '{}' not found.", job_name))?;

        info!("Running job: {}", job.name());
        if let Err(e) = job.run(&pool).await {
            error!("Job {} failed: {:?}", job.name(), e);
            failed = true;
        } else {
            info!("Job {} completed successfully", job.name());
        }
    }

    if failed {
        std::process::exit(1);
    }

    Ok(())
}
