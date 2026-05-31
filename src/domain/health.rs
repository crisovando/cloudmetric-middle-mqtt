use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkStats {
    pub in_value: f32,
    pub out_value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceHealth {
    // Mapea el camelCase del JSON al snake_case de Rust
    #[serde(rename = "serverId")]
    pub server_id: String,

    // Le indica a Serde que el JSON trae un número Unix en segundos, no un String
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,

    pub cpu: f32,
    pub memory: f32,
    pub temp: f32,
    pub network: NetworkStats,
    pub uptime: u64,

    // Opcional: Puedes capturar el estado que envía el simulador
    pub status: String,
}
