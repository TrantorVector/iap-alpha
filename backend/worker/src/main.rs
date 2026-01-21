use anyhow::{Context, Result};
use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tracing::{error, info};

mod jobs;
use jobs::{Job, EarningsPollingJob, PriceRefreshJob, FxRefresh, DocumentRefresh, MetricsRecalc};
use providers::mock::MockMarketDataProvider;
use std::sync::Arc;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    job: Option<String>,

    #[arg(long)]
    all: bool,
}

#[derive(Debug)]
struct Config {
    database_url: String,
}

impl Config {
    fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL must be set")?;
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
    
    // We only need config and DB if we are running jobs
    if !args.all && args.job.is_none() {
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

    let jobs: Vec<Box<dyn Job>> = vec![
        Box::new(EarningsPollingJob::new(pool.clone(), provider.clone())),
        Box::new(PriceRefreshJob::new(pool.clone(), provider.clone())),
        Box::new(FxRefresh),
        Box::new(DocumentRefresh),
        Box::new(MetricsRecalc),
    ];

    let mut failed = false;

    if args.all {
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
        let job = jobs.iter().find(|j| j.name() == job_name)
            .context(format!("Job '{}' not found. Available jobs: {:?}", job_name, jobs.iter().map(|j| j.name()).collect::<Vec<_>>()))?;
        
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
