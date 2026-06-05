#[derive(Debug, Clone)]
pub struct Topics {
    pub prefix: String,
}

impl Topics {
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }

    pub fn health_wildcard(&self) -> String {
        format!("{}/simulator/health/+", self.prefix)
    }

    pub fn config_topic(&self) -> String {
        format!("{}/simulator/config", self.prefix)
    }

    pub fn fleet_topic(&self) -> String {
        format!("{}/simulator/fleet", self.prefix)
    }

    pub fn control_topic(&self, server_id: &str) -> String {
        format!("{}/simulator/control/{}", self.prefix, server_id)
    }

    pub fn parse_health_topic(&self, topic: &str) -> Option<String> {
        let parts: Vec<&str> = topic.split('/').collect();
        match parts.as_slice() {
            [prefix, "simulator", "health", server_id] if *prefix == self.prefix => {
                Some(server_id.to_string())
            }
            _ => None,
        }
    }
}
