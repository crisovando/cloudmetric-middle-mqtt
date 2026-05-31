use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    #[default]
    Offline,
}
