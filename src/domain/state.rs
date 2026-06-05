use chrono::{DateTime, Utc};
use crate::domain::health::DeviceHealth;
use crate::domain::status::HealthStatus;

#[derive(Clone, Debug, Default)]
pub struct ServerState {
    pub server_id: String,
    pub name: String,
    pub health: DeviceHealth,
    pub status: HealthStatus,

    pub alert_active: bool,
    pub recovery_since: Option<DateTime<Utc>>,
    pub last_alert_at: Option<DateTime<Utc>>,
}
