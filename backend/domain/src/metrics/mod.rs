pub mod calculator;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetricValue {
    pub value: Option<f64>,
    pub formatted_value: String,
    pub unit: String,
    pub heat_map_quartile: Option<i32>,
}
