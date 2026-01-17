use crate::domain::{BalanceSheet, CashFlowStatement, DailyPrice, IncomeStatement};
use crate::metrics::MetricValue;
use bigdecimal::ToPrimitive;

pub struct MetricsCalculator;

impl MetricsCalculator {
    pub fn format_currency_value(value: f64, currency: &str) -> String {
        let abs_val = value.abs();
        let (suffix, divisor) = if abs_val >= 1_000_000_000_000.0 {
            ("T", 1_000_000_000_000.0)
        } else if abs_val >= 1_000_000_000.0 {
            ("B", 1_000_000_000.0)
        } else if abs_val >= 1_000_000.0 {
            ("M", 1_000_000.0)
        } else if abs_val >= 1_000.0 {
            ("K", 1_000.0)
        } else {
            ("", 1.0)
        };

        let formatted_val = value / divisor;
        format!("{}{:.2}{}", currency, formatted_val, suffix)
    }

    pub fn calculate_yoy_change(current: f64, prior: f64) -> Option<f64> {
        if prior == 0.0 {
            return None;
        }
        Some((current - prior) / prior.abs() * 100.0)
    }

    pub fn calculate_margin(numerator: f64, denominator: f64) -> Option<f64> {
        if denominator == 0.0 {
            return None;
        }
        Some((numerator / denominator) * 100.0)
    }

    pub fn calculate_acceleration_delta(growths: &[f64]) -> Vec<Option<f64>> {
        let mut deltas = Vec::with_capacity(growths.len());
        deltas.push(None); // First period has no acceleration delta

        for i in 1..growths.len() {
            deltas.push(Some(growths[i] - growths[i - 1]));
        }
        deltas
    }

    pub fn calculate_quartiles(values: &[Option<f64>]) -> Vec<Option<i32>> {
        let clean_values: Vec<f64> = values.iter().filter_map(|&v| v).collect();
        if clean_values.len() < 2 {
            return vec![None; values.len()];
        }

        let mut sorted = clean_values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = sorted.len();
        let q1 = sorted[len / 4];
        let q2 = sorted[len / 2];
        let q3 = sorted[3 * len / 4];

        values
            .iter()
            .map(|&v| {
                v.map(|val| {
                    if val <= q1 {
                        1
                    } else if val <= q2 {
                        2
                    } else if val <= q3 {
                        3
                    } else {
                        4
                    }
                })
            })
            .collect()
    }

    pub fn calculate_revenue_metrics(
        incomes: &[IncomeStatement],
        prior_year_incomes: &[Option<IncomeStatement>],
        currency: &str,
    ) -> (Vec<MetricValue>, Vec<MetricValue>, Vec<MetricValue>) {
        let mut revenues = Vec::new();
        let mut yoy_growths = Vec::new();
        let mut qoq_growths = Vec::new();

        for (i, income) in incomes.iter().enumerate() {
            let rev_f64 = income.revenue.as_ref().and_then(|v| v.to_f64());
            
            // Revenue
            revenues.push(MetricValue {
                value: rev_f64,
                formatted_value: rev_f64.map(|v| Self::format_currency_value(v, currency)).unwrap_or_else(|| "N/A".to_string()),
                unit: currency.to_string(),
                heat_map_quartile: None,
            });

            // YoY Growth
            let prior_year_rev = prior_year_incomes.get(i).and_then(|opt| opt.as_ref()).and_then(|inc| inc.revenue.as_ref()).and_then(|v| v.to_f64());
            let yoy = match (rev_f64, prior_year_rev) {
                (Some(curr), Some(prior)) => Self::calculate_yoy_change(curr, prior),
                _ => None,
            };
            yoy_growths.push(MetricValue {
                value: yoy,
                formatted_value: yoy.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "N/A".to_string()),
                unit: "%".to_string(),
                heat_map_quartile: None,
            });

            // QoQ Growth
            let prior_period_rev = if i > 0 {
                incomes[i-1].revenue.as_ref().and_then(|v| v.to_f64())
            } else {
                None
            };
            let qoq = match (rev_f64, prior_period_rev) {
                (Some(curr), Some(prior)) => Self::calculate_yoy_change(curr, prior), // Growth is growth
                _ => None,
            };
            qoq_growths.push(MetricValue {
                value: qoq,
                formatted_value: qoq.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "N/A".to_string()),
                unit: "%".to_string(),
                heat_map_quartile: None,
            });
        }

        // Apply quartiles if needed, for now return as is
        (revenues, yoy_growths, qoq_growths)
    }

    pub fn calculate_margin_metrics(
        incomes: &[IncomeStatement],
    ) -> (Vec<MetricValue>, Vec<MetricValue>, Vec<MetricValue>) {
        let mut gross_margins = Vec::new();
        let mut operating_margins = Vec::new();
        let mut net_margins = Vec::new();

        for income in incomes {
            let rev = income.revenue.as_ref().and_then(|v| v.to_f64());
            let gp = income.gross_profit.as_ref().and_then(|v| v.to_f64());
            let op = income.operating_income.as_ref().and_then(|v| v.to_f64());
            let ni = income.net_income.as_ref().and_then(|v| v.to_f64());

            let gm = match (gp, rev) {
                (Some(n), Some(d)) => Self::calculate_margin(n, d),
                _ => None,
            };
            gross_margins.push(MetricValue {
                value: gm,
                formatted_value: gm.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "N/A".to_string()),
                unit: "%".to_string(),
                heat_map_quartile: None,
            });

            let om = match (op, rev) {
                (Some(n), Some(d)) => Self::calculate_margin(n, d),
                _ => None,
            };
            operating_margins.push(MetricValue {
                value: om,
                formatted_value: om.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "N/A".to_string()),
                unit: "%".to_string(),
                heat_map_quartile: None,
            });

            let nm = match (ni, rev) {
                (Some(n), Some(d)) => Self::calculate_margin(n, d),
                _ => None,
            };
            net_margins.push(MetricValue {
                value: nm,
                formatted_value: nm.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "N/A".to_string()),
                unit: "%".to_string(),
                heat_map_quartile: None,
            });
        }

        (gross_margins, operating_margins, net_margins)
    }

    pub fn calculate_expansion_metrics(
        margins: &[MetricValue],
    ) -> Vec<MetricValue> {
        let mut expansions = Vec::new();
        expansions.push(MetricValue {
            value: None,
            formatted_value: "N/A".to_string(),
            unit: "bps".to_string(),
            heat_map_quartile: None,
        });

        for i in 1..margins.len() {
            let expansion = match (margins[i].value, margins[i - 1].value) {
                (Some(curr), Some(prior)) => Some((curr - prior) * 100.0), // represented in basis points
                _ => None,
            };
            expansions.push(MetricValue {
                value: expansion,
                formatted_value: expansion.map(|v| format!("{:.0} bps", v)).unwrap_or_else(|| "N/A".to_string()),
                unit: "bps".to_string(),
                heat_map_quartile: None,
            });
        }

        expansions
    }

    pub fn calculate_revenue_acceleration(
        yoy_growths: &[MetricValue],
    ) -> Vec<MetricValue> {
        let mut accelerations = Vec::new();
        accelerations.push(MetricValue {
            value: None,
            formatted_value: "N/A".to_string(),
            unit: "bps".to_string(),
            heat_map_quartile: None,
        });

        for i in 1..yoy_growths.len() {
            let accel = match (yoy_growths[i].value, yoy_growths[i - 1].value) {
                (Some(curr), Some(prior)) => Some((curr - prior) * 100.0), // basis points
                _ => None,
            };
            accelerations.push(MetricValue {
                value: accel,
                formatted_value: accel.map(|v| format!("{:.0} bps", v)).unwrap_or_else(|| "N/A".to_string()),
                unit: "bps".to_string(),
                heat_map_quartile: None,
            });
        }

        accelerations
    }

    pub fn calculate_cash_metrics(
        incomes: &[IncomeStatement],
        cash_flows: &[Option<CashFlowStatement>],
    ) -> (Vec<MetricValue>, Vec<MetricValue>) {
        let mut ocf_ratios = Vec::new();
        let mut fcf_ratios = Vec::new();

        for (i, income) in incomes.iter().enumerate() {
            let rev = income.revenue.as_ref().and_then(|v| v.to_f64());
            let cf = cash_flows.get(i).and_then(|opt| opt.as_ref());
            let ocf = cf.and_then(|c| c.operating_cash_flow.as_ref()).and_then(|v| v.to_f64());
            let fcf = cf.and_then(|c| c.free_cash_flow.as_ref()).and_then(|v| v.to_f64());

            let ocf_p = match (ocf, rev) {
                (Some(n), Some(d)) => Self::calculate_margin(n, d),
                _ => None,
            };
            ocf_ratios.push(MetricValue {
                value: ocf_p,
                formatted_value: ocf_p.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "N/A".to_string()),
                unit: "%".to_string(),
                heat_map_quartile: None,
            });

            let fcf_p = match (fcf, rev) {
                (Some(n), Some(d)) => Self::calculate_margin(n, d),
                _ => None,
            };
            fcf_ratios.push(MetricValue {
                value: fcf_p,
                formatted_value: fcf_p.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "N/A".to_string()),
                unit: "%".to_string(),
                heat_map_quartile: None,
            });
        }

        (ocf_ratios, fcf_ratios)
    }

    pub fn calculate_valuation_metrics(
        incomes: &[IncomeStatement],
        prices: &[Option<DailyPrice>],
    ) -> (Vec<MetricValue>, Vec<MetricValue>, Vec<MetricValue>, Vec<MetricValue>, Vec<MetricValue>) {
        let mut open_ratios = Vec::new();
        let mut high_ratios = Vec::new();
        let mut low_ratios = Vec::new();
        let mut close_ratios = Vec::new();
        let mut pe_ratios = Vec::new();

        for (i, income) in incomes.iter().enumerate() {
            let rev = income.revenue.as_ref().and_then(|v| v.to_f64());
            let price = prices.get(i).and_then(|opt| opt.as_ref());
            let eps = income.eps.as_ref().and_then(|v| v.to_f64());

            let metrics = [
                (price.map(|p| p.open), &mut open_ratios),
                (price.map(|p| p.high), &mut high_ratios),
                (price.map(|p| p.low), &mut low_ratios),
                (price.map(|p| p.close), &mut close_ratios),
            ];

            for (val, vec) in metrics {
                let p_rev = match (val, rev) {
                    (Some(v), Some(r)) => Self::calculate_margin(v, r),
                    _ => None,
                };
                vec.push(MetricValue {
                    value: p_rev,
                    formatted_value: p_rev.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "N/A".to_string()),
                    unit: "%".to_string(),
                    heat_map_quartile: None,
                });
            }

            // P/E Ratio
            let close = price.map(|p| p.close);
            let pe = match (close, eps) {
                (Some(p), Some(e)) if e > 0.0 => Some(p / e),
                _ => None,
            };
            pe_ratios.push(MetricValue {
                value: pe,
                formatted_value: pe.map(|v| format!("{:.2}x", v)).unwrap_or_else(|| "N/A".to_string()),
                unit: "x".to_string(),
                heat_map_quartile: None,
            });
        }

        (open_ratios, high_ratios, low_ratios, close_ratios, pe_ratios)
    }

    pub fn calculate_leverage_metrics(
        incomes: &[IncomeStatement],
        balances: &[Option<BalanceSheet>],
    ) -> (Vec<MetricValue>, Vec<MetricValue>) {
        let mut revenue_minus_net_debt_ratios = Vec::new();
        let mut shares_outstanding = Vec::new();

        for (i, income) in incomes.iter().enumerate() {
            let rev = income.revenue.as_ref().and_then(|v| v.to_f64());
            let balance = balances.get(i).and_then(|opt| opt.as_ref());
            let net_debt = balance.and_then(|b| b.net_debt.as_ref()).and_then(|v| v.to_f64());
            let shares = balance.and_then(|b| b.common_stock_shares_outstanding);

            // (Revenue - Net Debt) / Revenue %
            let ratio = match (rev, net_debt) {
                (Some(r), Some(nd)) if r != 0.0 => Some(((r - nd) / r) * 100.0),
                _ => None,
            };
            revenue_minus_net_debt_ratios.push(MetricValue {
                value: ratio,
                formatted_value: ratio.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "N/A".to_string()),
                unit: "%".to_string(),
                heat_map_quartile: None,
            });

            // Total Shares Outstanding
            shares_outstanding.push(MetricValue {
                value: shares.map(|s| s as f64),
                formatted_value: shares.map(|s| format!("{:.2}M", s as f64 / 1_000_000.0)).unwrap_or_else(|| "N/A".to_string()),
                unit: "shares".to_string(),
                heat_map_quartile: None,
            });
        }

        (revenue_minus_net_debt_ratios, shares_outstanding)
    }
    
    // Additional methods will be added here
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_currency_value() {
        assert_eq!(MetricsCalculator::format_currency_value(1_500_000_000.0, "$"), "$1.50B");
        assert_eq!(MetricsCalculator::format_currency_value(950_000.0, "$"), "$950.00K");
        assert_eq!(MetricsCalculator::format_currency_value(1_200_000_000_000.0, "$"), "$1.20T");
    }

    #[test]
    fn test_calculate_yoy_change() {
        assert_eq!(MetricsCalculator::calculate_yoy_change(120.0, 100.0), Some(20.0));
        assert_eq!(MetricsCalculator::calculate_yoy_change(80.0, 100.0), Some(-20.0));
        assert_eq!(MetricsCalculator::calculate_yoy_change(100.0, 0.0), None);
    }

    #[test]
    fn test_calculate_margin() {
        assert_eq!(MetricsCalculator::calculate_margin(20.0, 100.0), Some(20.0));
        assert_eq!(MetricsCalculator::calculate_margin(50.0, 200.0), Some(25.0));
        assert_eq!(MetricsCalculator::calculate_margin(50.0, 0.0), None);
    }

    #[test]
    fn test_calculate_margin_metrics() {
        use bigdecimal::BigDecimal;
        use std::str::FromStr;
        let incomes = vec![
            IncomeStatement {
                period_end_date: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                revenue: Some(BigDecimal::from_str("1000").unwrap()),
                gross_profit: Some(BigDecimal::from_str("400").unwrap()),
                operating_income: Some(BigDecimal::from_str("200").unwrap()),
                net_income: Some(BigDecimal::from_str("100").unwrap()),
                eps: Some(BigDecimal::from_str("1.0").unwrap()),
            }
        ];
        let (gm, om, nm) = MetricsCalculator::calculate_margin_metrics(&incomes);
        assert_eq!(gm[0].value, Some(40.0));
        assert_eq!(om[0].value, Some(20.0));
        assert_eq!(nm[0].value, Some(10.0));
    }

    #[test]
    fn test_calculate_valuation_metrics() {
        use bigdecimal::BigDecimal;
        use std::str::FromStr;
        let incomes = vec![
            IncomeStatement {
                period_end_date: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                revenue: Some(BigDecimal::from_str("1000").unwrap()),
                gross_profit: None,
                operating_income: None,
                net_income: None,
                eps: Some(BigDecimal::from_str("5.0").unwrap()),
            }
        ];
        let prices = vec![
            Some(DailyPrice {
                date: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                open: 140.0,
                high: 155.0,
                low: 135.0,
                close: 150.0,
            })
        ];
        let (open, high, low, close, pe) = MetricsCalculator::calculate_valuation_metrics(&incomes, &prices);
        
        let eps = 1e-10;
        assert!((open[0].value.unwrap() - 14.0).abs() < eps);
        assert!((high[0].value.unwrap() - 15.5).abs() < eps);
        assert!((low[0].value.unwrap() - 13.5).abs() < eps);
        assert!((close[0].value.unwrap() - 15.0).abs() < eps);
        assert!((pe[0].value.unwrap() - 30.0).abs() < eps);
    }
}
