use crate::jobs::Job;
use async_trait::async_trait;
use sqlx::{PgPool, FromRow};
use anyhow::Result;
use std::sync::Arc;
use domain::ports::market_data::MarketDataProvider;
use tracing::{info, error, instrument};
use uuid::Uuid;
use chrono::{Utc, NaiveDate};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct JobResult {
    processed: usize,
    updated: usize,
    errors: usize,
}

#[derive(FromRow)]
struct CompanyRow {
    id: Uuid,
    symbol: String,
    latest_quarter: Option<NaiveDate>,
}

pub struct EarningsPollingJob {
    db: PgPool,
    provider: Arc<dyn MarketDataProvider>,
}

impl EarningsPollingJob {
    pub fn new(db: PgPool, provider: Arc<dyn MarketDataProvider>) -> Self {
        Self { db, provider }
    }
}

#[async_trait]
impl Job for EarningsPollingJob {
    fn name(&self) -> &str {
        "earnings_poll"
    }

    #[instrument(skip(self, _pool))]
    async fn run(&self, _pool: &PgPool) -> Result<()> {
        info!("Starting earnings_poll job");
        let start_time = Utc::now();
        let job_id = Uuid::new_v4();

        // 2a. Create job_run record
        let insert_result = sqlx::query(
            "INSERT INTO job_runs (id, job_name, status, started_at) VALUES ($1, $2, $3, $4)"
        )
        .bind(job_id)
        .bind(self.name())
        .bind("running")
        .bind(start_time)
        .execute(&self.db)
        .await;

        if let Err(e) = insert_result {
            error!("Failed to create job_run record: {}. Continuing execution but this likely points to missing migration.", e);
        }

        let mut processed = 0;
        let mut updated = 0;
        let mut errors = 0;

        // 2c. Fetch earnings calendar
        info!("Fetching earnings calendar...");
        let calendar_result = self.provider.get_earnings_calendar().await;
        
        match calendar_result {
            Ok(events) => {
                info!("Fetched {} earnings events", events.len());

                // 2b. Get list of active tracked companies
                // Using generic query_as to avoid compile-time DB checks
                let active_companies_result = sqlx::query_as::<_, CompanyRow>(
                    "SELECT id, symbol, latest_quarter FROM companies WHERE is_active = true"
                )
                .fetch_all(&self.db)
                .await;

                match active_companies_result {
                    Ok(companies) => {
                        for company in companies {
                            processed += 1;
                            
                            // 3. Handle rate limiting (simple delay per active company processing to share load if we did fetches here)
                            // Since we fetched calendar already, this loop is fast. No delay needed.
                            
                            if let Some(event) = events.iter().find(|e| e.symbol == company.symbol) {
                                let mut needs_update = false;
                                let old_date = company.latest_quarter;
                                let new_date = event.fiscal_date_ending;

                                if let Some(new_d) = new_date {
                                    if let Some(old_d) = old_date {
                                        if new_d > old_d {
                                            needs_update = true;
                                        }
                                    } else {
                                        needs_update = true;
                                    }
                                }
                                
                                info!(
                                    company_symbol = %company.symbol,
                                    report_date = %event.report_date,
                                    "Found earnings event"
                                );

                                if needs_update {
                                     info!(
                                         company_id = %company.id,
                                         old_date = ?old_date,
                                         new_date = ?new_date,
                                         "Updating company earnings data"
                                     );

                                     let update_result = sqlx::query(
                                         "UPDATE companies SET latest_quarter = $1, updated_at = NOW() WHERE id = $2"
                                     )
                                     .bind(new_date)
                                     .bind(company.id)
                                     .execute(&self.db)
                                     .await;

                                     match update_result {
                                         Ok(_) => updated += 1,
                                         Err(e) => {
                                             error!(company_id = %company.id, error = %e, "Failed to update company");
                                             errors += 1;
                                         }
                                     }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch active companies: {}", e);
                        errors += 1;
                    }
                }
            }
            Err(e) => {
                error!("Failed to fetch earnings calendar: {}", e);
                errors += 1;
            }
        }

        // 2d. Update job_run record with results
        let end_time = Utc::now();
        let result = JobResult { processed, updated, errors };
        let result_json = serde_json::to_value(&result).unwrap_or(serde_json::Value::Null);
        let status = if errors > 0 && processed == 0 { "failed" } else { "completed" };

        let _ = sqlx::query(
            "UPDATE job_runs SET status = $1, ended_at = $2, result = $3 WHERE id = $4"
        )
        .bind(status)
        .bind(end_time)
        .bind(result_json)
        .bind(job_id)
        .execute(&self.db)
        .await;

        info!("Earnings poll job finished: {:?}", result);
        
        Ok(())
    }
}
