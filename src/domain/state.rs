use crate::domain::health::DeviceHealth;
use crate::domain::status::HealthStatus;

#[derive(Clone, Debug, Default)]
pub struct ServerState {
    pub server_id: String,
    pub name: String,
    pub health: DeviceHealth,
    pub status: HealthStatus,
}
