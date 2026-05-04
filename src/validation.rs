fn is_cpu_overloaded(cpu: f32) -> bool {
    cpu > 80.0
}

fn is_memory_overloaded(memory: f32) -> bool {
    memory > 80.0
}

fn is_temperature_critical(temp: f32) -> bool {
    temp > 75.0
}

fn is_network_congested(network: &crate::device_health::NetworkStats) -> bool {
    network.in_value > 100.0 || network.out_value > 100.0
}

pub fn is_device_unhealthy(health: &crate::device_health::DeviceHealth) -> bool {
    is_cpu_overloaded(health.cpu)
        || is_memory_overloaded(health.memory)
        || is_temperature_critical(health.temp)
        || is_network_congested(&health.network)
}
