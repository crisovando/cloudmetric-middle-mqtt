use crate::domain::{health::DeviceHealth, status::HealthStatus};
use crate::infrastructure::config::ThresholdsConfig;

pub fn determine_status(health: &DeviceHealth, config: &ThresholdsConfig) -> HealthStatus {
    if is_critical(health, config) {
        return HealthStatus::Critical;
    }

    if is_warning(health, config) {
        return HealthStatus::Warning;
    }

    HealthStatus::Healthy
}

fn is_critical(health: &DeviceHealth, config: &ThresholdsConfig) -> bool {
    health.cpu >= config.cpu_critical
        || health.memory >= config.memory_critical
        || health.temp >= config.temp_critical
}

fn is_warning(health: &DeviceHealth, config: &ThresholdsConfig) -> bool {
    health.cpu >= config.cpu_warning
        || health.memory >= config.memory_warning
        || health.temp >= config.temp_warning
}
