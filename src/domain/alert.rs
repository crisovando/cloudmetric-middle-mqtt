use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlertMetric {
    Cpu,
    Temp,
    Memory,
    Network,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub server_id: String,
    pub server_name: String,
    pub timestamp: DateTime<Utc>,
    pub metric: AlertMetric,
    pub value: Option<f32>,
    pub resolved_at: Option<DateTime<Utc>>,
}
