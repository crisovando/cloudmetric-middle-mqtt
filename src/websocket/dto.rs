use serde::{Deserialize, Serialize};

use crate::domain::{
    command::ServerCommand, metric::MetricName, state::ServerState, status::HealthStatus,
};
use crate::validation::engine;
use std::convert::TryFrom;

#[derive(Debug, Serialize, Clone)]
pub struct Thresholds {
    pub cpu_warning: f32,
    pub cpu_critical: f32,
    pub temp_warning: f32,
    pub temp_critical: f32,
    pub memory_warning: f32,
    pub memory_critical: f32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            cpu_warning: engine::CPU_WARNING,
            cpu_critical: engine::CPU_CRITICAL,
            temp_warning: engine::TEMP_WARNING,
            temp_critical: engine::TEMP_CRITICAL,
            memory_warning: engine::MEMORY_WARNING,
            memory_critical: engine::MEMORY_CRITICAL,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct WsServerUpdate {
    pub server_id: String,
    pub name: String,
    pub timestamp: i64,
    pub cpu: f32,
    pub memory: f32,
    pub temp: f32,
    pub network_in: f32,
    pub network_out: f32,
    pub uptime: u64,
    pub status: HealthStatus,
    pub thresholds: Thresholds,
}

#[derive(Debug, Serialize, Clone)]
pub struct WsSnapshot {
    pub servers: Vec<WsServerUpdate>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum WsEvent {
    Snapshot(WsSnapshot),
    Update(WsServerUpdate),
    Deleted { server_id: String },
}

impl From<&ServerState> for WsServerUpdate {
    fn from(state: &ServerState) -> Self {
        Self {
            server_id: state.server_id.clone(),
            name: state.name.clone(),
            timestamp: state.health.timestamp.timestamp(),
            cpu: state.health.cpu,
            memory: state.health.memory,
            temp: state.health.temp,
            network_in: state.health.network.in_value,
            network_out: state.health.network.out_value,
            uptime: state.health.uptime,
            status: state.status.clone(),
            thresholds: Thresholds::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CommandType {
    SetFailureProbability,
    SetMetric,
    ReleaseMetric,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommandPayload {
    pub server_id: String,
    pub command: CommandType,
    pub metric: Option<MetricName>,
    pub value: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WsIncoming {
    Command(CommandPayload),
}

impl TryFrom<CommandPayload> for ServerCommand {
    type Error = String;

    fn try_from(dto: CommandPayload) -> Result<Self, Self::Error> {
        match dto.command {
            CommandType::SetFailureProbability => {
                let probability = dto
                    .value
                    .ok_or("El comando SetFailureProbability requiere el campo 'value'")?;

                Ok(ServerCommand::SetFailureProbability {
                    server_id: dto.server_id,
                    probability,
                })
            }
            CommandType::SetMetric => {
                let metric = dto
                    .metric
                    .ok_or("El comando SetMetric requiere el campo 'metric'")?;
                let value = dto
                    .value
                    .ok_or("El comando SetMetric requiere el campo 'value'")?;

                Ok(ServerCommand::SetMetric {
                    server_id: dto.server_id,
                    metric,
                    value,
                })
            }
            CommandType::ReleaseMetric => {
                let metric = dto
                    .metric
                    .ok_or("El comando ReleaseMetric requiere el campo 'metric'")?;

                Ok(ServerCommand::ReleaseMetric {
                    server_id: dto.server_id,
                    metric,
                })
            }
        }
    }
}
