use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents a fiscal period with consistent labeling
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct FiscalPeriod {
    pub period_end_date: NaiveDate,
    pub period_type: PeriodType,
    pub fiscal_year: i32,
    pub fiscal_quarter: Option<i32>,  // None for annual
    pub display_label: String,        // e.g., "FY2024", "Q3 2024"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PeriodType {
    Annual,
    Quarterly,
}

/// Generates consistent period windows for both metrics and documents
pub struct PeriodWindowGenerator {
    fiscal_year_end_month: u32,  // e.g., 3 for March, 12 for December
}

impl PeriodWindowGenerator {
    pub fn new(fiscal_year_end_month: u32) -> Self {
        Self { fiscal_year_end_month }
    }

    /// Generate periods for display (4-10 periods based on user selection)
    pub fn generate_periods(
        &self,
        period_count: usize,
        period_type: PeriodType,
        as_of_date: NaiveDate,
    ) -> Vec<FiscalPeriod> {
        let mut periods = Vec::with_capacity(period_count);
        
        for i in 0..period_count {
            let period = match period_type {
                PeriodType::Annual => self.calculate_annual_period(as_of_date, i),
                PeriodType::Quarterly => self.calculate_quarterly_period(as_of_date, i),
            };
            periods.push(period);
        }
        
        periods
    }

    fn calculate_annual_period(&self, as_of_date: NaiveDate, offset: usize) -> FiscalPeriod {
        let mut year = as_of_date.year();
        let month = as_of_date.month();
        
        // If we haven't reached the fiscal year end yet, the current year's fiscal year is last year
        if month < self.fiscal_year_end_month {
            year -= 1;
        }
        
        let fiscal_year = year - (offset as i32);
        
        // Calculate the end date of that fiscal year
        let end_date = self.get_fiscal_year_end(fiscal_year);
        
        let mut period = FiscalPeriod {
            period_end_date: end_date,
            period_type: PeriodType::Annual,
            fiscal_year,
            fiscal_quarter: None,
            display_label: String::new(),
        };
        period.display_label = self.format_label(&period);
        period
    }

    fn calculate_quarterly_period(&self, as_of_date: NaiveDate, offset: usize) -> FiscalPeriod {
        // Start from the most recent completed quarter
        let (mut year, mut quarter) = self.get_fiscal_quarter(as_of_date);
        
        // Apply offset
        for _ in 0..offset {
            if quarter == 1 {
                quarter = 4;
                year -= 1;
            } else {
                quarter -= 1;
            }
        }
        
        let end_date = self.get_quarter_end(year, quarter);
        
        let mut period = FiscalPeriod {
            period_end_date: end_date,
            period_type: PeriodType::Quarterly,
            fiscal_year: year,
            fiscal_quarter: Some(quarter),
            display_label: String::new(),
        };
        period.display_label = self.format_label(&period);
        period
    }

    fn get_fiscal_year_end(&self, fiscal_year: i32) -> NaiveDate {
        let month = self.fiscal_year_end_month;
        let day = if month == 2 {
            if (fiscal_year % 4 == 0 && fiscal_year % 100 != 0) || (fiscal_year % 400 == 0) { 29 } else { 28 }
        } else if [4, 6, 9, 11].contains(&month) {
            30
        } else {
            31
        };
        
        NaiveDate::from_ymd_opt(fiscal_year, month, day).unwrap()
    }

    fn get_fiscal_quarter(&self, date: NaiveDate) -> (i32, i32) {
        let year = date.year();
        let month = date.month();
        
        // Fiscal year ends at self.fiscal_year_end_month
        // Q4: [FYE-2, FYE]
        // Q3: [FYE-5, FYE-3]
        // Q2: [FYE-8, FYE-6]
        // Q1: [FYE-11, FYE-9]
        
        // Re-calculate quarter based on f_year start
        let start_month = (self.fiscal_year_end_month % 12) + 1;
        
        let f_year = if month <= self.fiscal_year_end_month {
            year
        } else {
            year + 1
        };
        
        let diff = if month >= start_month {
            month - start_month
        } else {
            month + 12 - start_month
        };
        let f_quarter = (diff / 3) + 1;
        
        (f_year, f_quarter as i32)
    }

    fn get_quarter_end(&self, fiscal_year: i32, fiscal_quarter: i32) -> NaiveDate {
        // Let's redo it properly.
        let target_month = ((self.fiscal_year_end_month + (fiscal_quarter as u32 * 3) - 1) % 12) + 1;
        // If FYE is Dec (12): Q1=3, Q2=6, Q3=9, Q4=12.
        // If FYE is Mar (3): Q1=6, Q2=9, Q3=12, Q4=3.
        
        let target_year = if target_month <= self.fiscal_year_end_month {
            fiscal_year
        } else {
            fiscal_year - 1
        };
        
        let day = match target_month {
            2 => if (target_year % 4 == 0 && target_year % 100 != 0) || (target_year % 400 == 0) { 29 } else { 28 },
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };
        
        NaiveDate::from_ymd_opt(target_year, target_month, day).unwrap()
    }

    fn format_label(&self, period: &FiscalPeriod) -> String {
        match period.period_type {
            PeriodType::Annual => format!("FY{}", period.fiscal_year),
            PeriodType::Quarterly => {
                let q = period.fiscal_quarter.unwrap_or(4);
                if q == 4 {
                    format!("FY{}", period.fiscal_year)
                } else {
                    format!("Q{} {}", q, period.fiscal_year)
                }
            }
        }
    }

    fn _is_fy_aligned(&self, date: &NaiveDate) -> bool {
        date.month() == self.fiscal_year_end_month
    }
}
