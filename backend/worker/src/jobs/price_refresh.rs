use crate::jobs::Job;
use async_trait::async_trait;
use sqlx::{PgPool, FromRow};
use anyhow::Result;
use std::sync::Arc;
use domain::ports::market_data::MarketDataProvider;
use domain::domain::OutputSize;
use tracing::{info, error, warn, instrument};
use uuid::Uuid;
use chrono::{Utc, NaiveDate, Datelike, Weekday};
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
    shares_outstanding: Option<i64>,
}

pub struct PriceRefreshJob {
    db: PgPool,
    provider: Arc<dyn MarketDataProvider>,
}

impl PriceRefreshJob {
    pub fn new(db: PgPool, provider: Arc<dyn MarketDataProvider>) -> Self {
        Self { db, provider }
    }

    // Helper to check if a date is a trading day
    fn is_trading_day(&self, date: NaiveDate) -> bool {
        let weekday = date.weekday();
        weekday != Weekday::Sat && weekday != Weekday::Sun
    }
}

#[async_trait]
impl Job for PriceRefreshJob {
    fn name(&self) -> &str {
        "price_refresh"
    }

    #[instrument(skip(self, _pool))]
    async fn run(&self, _pool: &PgPool) -> Result<()> {
        info!("Starting price_refresh job");
        let start_time = Utc::now();
        let job_id = Uuid::new_v4();

        // Track job run
        let _ = sqlx::query(
            "INSERT INTO job_runs (id, job_name, status, started_at) VALUES ($1, $2, $3, $4)"
        )
        .bind(job_id)
        .bind(self.name())
        .bind("running")
        .bind(start_time)
        .execute(&self.db)
        .await
        .map_err(|e| warn!("Failed to create job_run record: {}", e));

        let mut processed = 0;
        let mut updated = 0;
        let mut errors = 0;

        // Fetch active companies
        let companies_result = sqlx::query_as::<_, CompanyRow>(
            "SELECT id, symbol, shares_outstanding FROM companies WHERE is_active = true"
        )
        .fetch_all(&self.db)
        .await;

        match companies_result {
            Ok(companies) => {
                for company in companies {
                    processed += 1;
                    
                    // Check if we need update
                    // Check if we need update
                    let max_date_result = sqlx::query_scalar::<_, Option<NaiveDate>>(
                        "SELECT MAX(price_date) FROM daily_prices WHERE company_id = $1"
                    )
                    .bind(company.id)
                    .fetch_one(&self.db)
                    .await;

                    let mut has_existing_data = false;
                    let should_update = match max_date_result {
                        Ok(Some(last_date)) => {
                            has_existing_data = true;
                            let today = Utc::now().date_naive();
                            
                            if !self.is_trading_day(today) {
                                // Skip weekends
                                false
                            } else {
                                // If today is trading day, check if we are behind
                                last_date < today
                            }
                        },
                        Ok(None) => true,
                        Err(e) => {
                            error!("Failed to check price date for {}: {}", company.symbol, e);
                            errors += 1;
                            false
                        }
                    };

                    if should_update {
                         // Determine output size
                         let output_size = if has_existing_data {
                             OutputSize::Compact
                         } else {
                             OutputSize::Full
                         };

                         info!("Fetching prices for {} ({:?})", company.symbol, output_size);
                         
                         match self.provider.get_daily_prices(&company.symbol, output_size).await {
                             Ok(prices) => {
                                 // Upsert prices
                                 let mut batch_updated = 0;
                                 for price in prices {
                                     // Insert
                                     // Note: daily_prices has volume, but domain DailyPrice might not
                                     let r = sqlx::query(
                                         r#"
                                         INSERT INTO daily_prices (company_id, price_date, open, high, low, close, volume)
                                         VALUES ($1, $2, $3, $4, $5, $6, NULL) 
                                         ON CONFLICT (company_id, price_date) 
                                         DO UPDATE SET open = EXCLUDED.open, high = EXCLUDED.high, low = EXCLUDED.low, close = EXCLUDED.close
                                         "#
                                     )
                                     .bind(company.id)
                                     .bind(price.date)
                                     .bind(price.open)
                                     .bind(price.high)
                                     .bind(price.low)
                                     .bind(price.close)
                                     .execute(&self.db)
                                     .await;

                                     if let Err(e) = r {
                                         error!("Failed to upsert price for {}: {}", company.symbol, e);
                                     } else {
                                         batch_updated += 1;
                                     }
                                 }
                                 
                                 if batch_updated > 0 {
                                     updated += 1;
                                     // Update market cap
                                     // latest close * shares
                                     let latest_close_result = sqlx::query_scalar::<_, bigdecimal::BigDecimal>(
                                         "SELECT close FROM daily_prices WHERE company_id = $1 ORDER BY price_date DESC LIMIT 1"
                                     )
                                     .bind(company.id)
                                     .fetch_optional(&self.db)
                                     .await;

                                     // Note: daily_prices close is DECIMAL, mapped to BigDecimal in sqlx usually, 
                                     // but prompt code used f64. Schema says DECIMAL(15, 4).
                                     // Rust sqlx mapping for DECIMAL is BigDecimal.
                                     // If I query_scalar, I should probably use f64 if I can, or BigDecimal. 
                                     // sqlx postgres: DECIMAL maps to BigDecimal.
                                     // If I use `query_scalar::<_, f64>`, it might fail if strict.
                                     // safer to use BigDecimal and to_f64().
                                     
                                     // Actually, let's verify if I can just use f64 for binding to DECIMAL. Yes usually.
                                     // For reading back, converting BigDecimal to f64 might be needed.
                                     
                                     if let Ok(Some(latest_close_bd)) = latest_close_result {
                                         // bigdecimal::ToPrimitive is needed
                                         use bigdecimal::ToPrimitive;
                                         if let Some(latest_close_f64) = latest_close_bd.to_f64() {
                                              if let Some(shares) = company.shares_outstanding {
                                                 let market_cap = (latest_close_f64 * shares as f64) as i64;
                                                 let _ = sqlx::query("UPDATE companies SET market_cap = $1, updated_at = NOW() WHERE id = $2")
                                                     .bind(market_cap)
                                                     .bind(company.id)
                                                     .execute(&self.db)
                                                     .await;
                                              }
                                         }
                                     }
                                 }
                             }
                             Err(e) => {
                                 error!("Failed to fetch prices for {}: {}", company.symbol, e);
                                 errors += 1;
                             }
                         }
                    }
                }
            }
            Err(e) => {
                error!("Failed to fetch companies: {}", e);
                errors += 1;
            }
        }

        // Finish job run
        let end_time = Utc::now();
        let result = JobResult { processed, updated, errors };
        let result_json = serde_json::to_value(&result).unwrap_or(serde_json::Value::Null);
        let status = if errors > 0 && updated == 0 { "failed" } else { "completed" };

        let _ = sqlx::query(
            "UPDATE job_runs SET status = $1, ended_at = $2, result = $3 WHERE id = $4"
        )
        .bind(status)
        .bind(end_time)
        .bind(result_json)
        .bind(job_id)
        .execute(&self.db)
        .await;

        info!("Price refresh job finished: {:?}", result);

        Ok(())
    }
}
