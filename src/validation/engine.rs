use crate::domain::{
    health::DeviceHealth,
    status::HealthStatus,
};

pub const CPU_WARNING: f32 = 70.0;
pub const CPU_CRITICAL: f32 = 90.0;
pub const MEMORY_WARNING: f32 = 70.0;
pub const MEMORY_CRITICAL: f32 = 90.0;
pub const TEMP_WARNING: f32 = 75.0;
pub const TEMP_CRITICAL: f32 = 85.0;

pub fn determine_status(
    health: &DeviceHealth,
) -> HealthStatus {
    if is_critical(health) {
        return HealthStatus::Critical;
    }

    if is_warning(health) {
        return HealthStatus::Warning;
    }

    HealthStatus::Healthy
}

fn is_critical(
    health: &DeviceHealth,
) -> bool {
    health.cpu >= CPU_CRITICAL
        || health.memory >= MEMORY_CRITICAL
        || health.temp >= TEMP_CRITICAL
}

fn is_warning(
    health: &DeviceHealth,
) -> bool {
    health.cpu >= CPU_WARNING
        || health.memory >= MEMORY_WARNING
        || health.temp >= TEMP_WARNING
}