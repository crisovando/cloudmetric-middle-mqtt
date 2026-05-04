use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkStats {
    pub in_value: f32,
    pub out_value: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceHealth {
    pub timestamp: String,
    pub cpu: f32,
    pub temp: f32,
    pub memory: f32,
    pub network: NetworkStats,
    pub uptime: u64,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerHealth {
    pub serverId: String,
    pub health: DeviceHealth,
}
impl DeviceHealth {
    pub fn mark_unhealthy(&mut self) {
        self.status = "unhealthy".to_string();
    }
}
