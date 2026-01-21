use crate::jobs::Job;
use anyhow::Result;
use async_trait::async_trait;
use bigdecimal::{BigDecimal, ToPrimitive, FromPrimitive};
use chrono::{NaiveDate, Datelike};
use db::models::financials::{
    IncomeStatement as DbIncome, 
    BalanceSheet as DbBalance, 
    CashFlowStatement as DbCashFlow
};
use db::models::DailyPrice as DbPrice;
use domain::domain::{
    IncomeStatement as DomainIncome, 
    BalanceSheet as DomainBalance, 
    CashFlowStatement as DomainCashFlow, 
    DailyPrice as DomainPrice
};
use domain::metrics::calculator::MetricsCalculator;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{info, error};

pub struct MetricsRecalculationJob;

#[async_trait]
impl Job for MetricsRecalculationJob {
    fn name(&self) -> &str {
        "metrics_recalc"
    }

    async fn run(&self, pool: &PgPool) -> Result<()> {
        info!("Starting metrics recalculation job");

        // Fetch active companies
        let companies = sqlx::query!(
            "SELECT id, symbol, currency FROM companies WHERE is_active = true"
        )
        .fetch_all(pool)
        .await?;

        info!("Found {} active companies to process", companies.len());

        let mut success_count = 0;
        let mut fail_count = 0;

        for company in companies {
            let currency = company.currency.unwrap_or_else(|| "USD".to_string());
            match process_company(pool, company.id, &company.symbol, &currency).await {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    error!("Failed to calculate metrics for {}: {:?}", company.symbol, e);
                    fail_count += 1;
                }
            }
        }

        info!(
            "Metrics recalculation completed. Success: {}, Failed: {}",
            success_count, fail_count
        );

        Ok(())
    }
}

async fn process_company(
    pool: &PgPool,
    company_id: uuid::Uuid,
    _symbol: &str,
    currency: &str,
) -> Result<()> {
    // 1. Fetch all financial statements sorted by period_end_date ASC
    let incomes = sqlx::query_as!(
        DbIncome,
        "SELECT * FROM income_statements WHERE company_id = $1 ORDER BY period_end_date ASC",
        company_id
    )
    .fetch_all(pool)
    .await?;

    if incomes.is_empty() {
        return Ok(());
    }

    let balances = sqlx::query_as!(
        DbBalance,
        "SELECT * FROM balance_sheets WHERE company_id = $1 ORDER BY period_end_date ASC",
        company_id
    )
    .fetch_all(pool)
    .await?;

    let cash_flows = sqlx::query_as!(
        DbCashFlow,
        "SELECT * FROM cash_flow_statements WHERE company_id = $1 ORDER BY period_end_date ASC",
        company_id
    )
    .fetch_all(pool)
    .await?;

    // Create lookups for balance and cashflow
    let bal_map: HashMap<(NaiveDate, String), DbBalance> = balances
        .into_iter()
        .map(|b| ((b.period_end_date, b.period_type.clone()), b))
        .collect();
    
    let cf_map: HashMap<(NaiveDate, String), DbCashFlow> = cash_flows
        .into_iter()
        .map(|c| ((c.period_end_date, c.period_type.clone()), c))
        .collect();

    // 2. Align data
    let mut domain_incomes = Vec::new();
    let mut aligned_balances = Vec::new();
    let mut aligned_cash_flows = Vec::new();
    let mut dates = Vec::new();

    for income in &incomes {
        let key = (income.period_end_date, income.period_type.clone());
        dates.push(income.period_end_date);

        // Convert Income
        domain_incomes.push(DomainIncome {
            period_end_date: income.period_end_date,
            revenue: income.total_revenue.clone(),
            gross_profit: income.gross_profit.clone(),
            operating_income: income.operating_income.clone(),
            net_income: income.net_income.clone(),
            eps: income.basic_eps.clone(), 
        });

        // Align Balance
        let mut domain_bal = bal_map.get(&key).map(|b| DomainBalance {
            period_end_date: b.period_end_date,
            total_assets: b.total_assets.clone(),
            total_liabilities: b.total_liabilities.clone(),
            total_equity: b.total_equity.clone(),
            cash_and_equivalents: b.cash_and_equivalents.clone(),
            short_term_investments: b.short_term_investments.clone(),
            short_term_debt: b.short_term_debt.clone(),
            long_term_debt: b.long_term_debt.clone(),
            net_debt: b.net_debt.clone(),
            common_stock_shares_outstanding: None, 
        });

        // Try to patch shares from income if available
        if let Some(ref mut db) = domain_bal {
             if let Some(shares) = income.shares_outstanding {
                 db.common_stock_shares_outstanding = Some(shares);
             }
        }
        aligned_balances.push(domain_bal);

        // Align CashFlow
        let cf_opt = cf_map.get(&key).map(|c| DomainCashFlow {
            period_end_date: c.period_end_date,
            operating_cash_flow: c.operating_cash_flow.clone(),
            capital_expenditures: c.capital_expenditures.clone(),
            free_cash_flow: c.free_cash_flow.clone(),
        });
        aligned_cash_flows.push(cf_opt);
    }

    // 3. Prepare prior year incomes
    let mut prior_year_incomes = Vec::new();
    for income in &incomes {
        let target_date = income.period_end_date;
        let prior = incomes.iter().find(|i| {
            i.period_type == income.period_type && 
            i.period_end_date < target_date && 
            (target_date.year() - i.period_end_date.year() == 1) &&
             (target_date.month() as i32 - i.period_end_date.month() as i32).abs() <= 1
        });
        
        let prior_domain = prior.map(|p| DomainIncome {
             period_end_date: p.period_end_date,
             revenue: p.total_revenue.clone(),
             gross_profit: p.gross_profit.clone(),
             operating_income: p.operating_income.clone(),
             net_income: p.net_income.clone(),
             eps: p.basic_eps.clone(),
        });
        prior_year_incomes.push(prior_domain);
    }

    // 4. Fetch Prices
    let mut aligned_prices = Vec::new();
    for date in &dates {
        let price = sqlx::query_as!(
            DbPrice,
            "SELECT * FROM daily_prices WHERE company_id = $1 AND price_date <= $2 ORDER BY price_date DESC LIMIT 1",
            company_id,
            *date
        )
        .fetch_optional(pool)
        .await?;

        let domain_price = price.map(|p| DomainPrice {
            date: p.price_date,
            open: p.open.and_then(|v| v.to_f64()).unwrap_or(0.0),
            high: p.high.and_then(|v| v.to_f64()).unwrap_or(0.0),
            low: p.low.and_then(|v| v.to_f64()).unwrap_or(0.0),
            close: p.close.and_then(|v| v.to_f64()).unwrap_or(0.0),
        });
        aligned_prices.push(domain_price);
    }

    // 5. Run Calculations
    
    // Revenue Metrics
    let (_, yoy_growths, qoq_growths) = MetricsCalculator::calculate_revenue_metrics(
        &domain_incomes, &prior_year_incomes, currency
    );
    
    // Margins
    let (gross_margins, op_margins, net_margins) = MetricsCalculator::calculate_margin_metrics(
        &domain_incomes
    );
    
    // Acceleration
    let accels = MetricsCalculator::calculate_revenue_acceleration(&yoy_growths);

    // Cash Metrics
    let (ocf_ratios, fcf_ratios) = MetricsCalculator::calculate_cash_metrics(
        &domain_incomes, &aligned_cash_flows
    );

    // Leverage
    let (rev_net_debt, _shares) = MetricsCalculator::calculate_leverage_metrics(
        &domain_incomes, &aligned_balances
    );

    // Valuation (Point-in-time)
    let val_metrics = MetricsCalculator::calculate_valuation_metrics(
        &domain_incomes, &aligned_prices
    );

    // 6. Save Metrics
    for i in 0..domain_incomes.len() {
        let period_end = domain_incomes[i].period_end_date;
        let period_type = incomes[i].period_type.clone(); 
        
        let mut metrics_to_save: HashMap<String, Option<f64>> = HashMap::new();
        
        metrics_to_save.insert("yoy_revenue_growth_pct".to_string(), yoy_growths[i].value);
        metrics_to_save.insert("qoq_revenue_growth_pct".to_string(), qoq_growths[i].value);
        metrics_to_save.insert("growth_acceleration".to_string(), accels[i].value);
        
        metrics_to_save.insert("gross_margin_pct".to_string(), gross_margins[i].value);
        metrics_to_save.insert("operating_margin_pct".to_string(), op_margins[i].value);
        metrics_to_save.insert("net_margin_pct".to_string(), net_margins[i].value);
        
        metrics_to_save.insert("ocf_revenue_pct".to_string(), ocf_ratios[i].value);
        metrics_to_save.insert("fcf_revenue_pct".to_string(), fcf_ratios[i].value);
        
        metrics_to_save.insert("revenue_minus_net_debt_pct".to_string(), rev_net_debt[i].value);
        
        metrics_to_save.insert("pe_ratio_historical".to_string(), val_metrics.pe_ratios[i].value);
        
        for (name, val_opt) in metrics_to_save {
            if let Some(val) = val_opt {
                insert_metric(pool, company_id, period_end, &period_type, &name, val).await?;
            }
        }
    }
    
    // 7. Latest Price Metrics
    if let Some(latest_idx) = domain_incomes.len().checked_sub(1) {
        let latest_income = &domain_incomes[latest_idx];
        let latest_period_end = latest_income.period_end_date;
        let latest_period_type = incomes[latest_idx].period_type.clone();
        
        let latest_price_row = sqlx::query_as!(
            DbPrice,
            "SELECT * FROM daily_prices WHERE company_id = $1 ORDER BY price_date DESC LIMIT 1",
            company_id
        )
        .fetch_optional(pool)
        .await?;

        if let Some(lp) = latest_price_row {
            let close = lp.close.and_then(|v| v.to_f64()).unwrap_or(0.0);
            
            // P/E (Latest)
            if let Some(eps) = latest_income.eps.as_ref().and_then(|v| v.to_f64()) {
                if eps > 0.0 {
                    let pe = close / eps;
                    insert_metric(pool, company_id, latest_period_end, &latest_period_type, "pe_ratio_ttm", pe).await?;
                }
            }
            
            // Momentum
            let dates = [
                ("momentum_1m", 30),
                ("momentum_3m", 90),
                ("momentum_6m", 180),
            ];
            
            for (name, days) in dates {
                 let target_date = lp.price_date - chrono::Duration::days(days);
                 let hist_price = sqlx::query_as!(
                    DbPrice,
                    "SELECT * FROM daily_prices WHERE company_id = $1 AND price_date <= $2 ORDER BY price_date DESC LIMIT 1",
                    company_id,
                    target_date
                )
                .fetch_optional(pool)
                .await?;
                
                if let Some(hp) = hist_price {
                    let hp_close = hp.close.and_then(|v| v.to_f64()).unwrap_or(0.0);
                    if hp_close > 0.0 {
                        let mom = (close / hp_close - 1.0) * 100.0;
                         insert_metric(pool, company_id, latest_period_end, &latest_period_type, name, mom).await?;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn insert_metric(
    pool: &PgPool, 
    company_id: uuid::Uuid, 
    date: NaiveDate, 
    ptype: &str, 
    name: &str, 
    val: f64
) -> Result<()> {
    let val_bd = BigDecimal::from_f64(val).unwrap_or_default();
    sqlx::query!(
        "INSERT INTO derived_metrics 
        (company_id, period_end_date, period_type, metric_name, metric_value)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (company_id, period_end_date, period_type, metric_name)
        DO UPDATE SET metric_value = $5, created_at = NOW()",
        company_id,
        date,
        ptype,
        name,
        val_bd
    )
    .execute(pool)
    .await?;
    Ok(())
}
