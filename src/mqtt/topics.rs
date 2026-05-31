pub const ROOT_TOPIC: &str = "cloudmetric";

pub const HEALTH_WILDCARD: &str = "cloudmetric/simulator/health/+";

pub const CONFIG_TOPIC: &str = "cloudmetric/simulator/config";

pub const FLEET_TOPIC: &str = "cloudmetric/simulator/fleet";

pub fn control_topic(server_id: &str) -> String {
    format!("{ROOT_TOPIC}/simulator/control/{server_id}")
}

pub fn parse_health_topic(topic: &str) -> Option<String> {
    let parts: Vec<&str> = topic.split('/').collect();
    match parts.as_slice() {
        ["cloudmetric", "simulator", "health", server_id] => Some(server_id.to_string()),
        _ => None,
    }
}
