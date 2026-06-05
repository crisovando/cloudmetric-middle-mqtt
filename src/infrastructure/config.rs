use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub mqtt: MqttConfig,
    pub thresholds: ThresholdsConfig,
    pub alerts: AlertConfig,
    pub telegram: TelegramConfig,
}

#[derive(Debug, Clone)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub client_id: String,
    pub topic_prefix: String,
}

#[derive(Debug, Clone)]
pub struct ThresholdsConfig {
    pub cpu_warning: f32,
    pub cpu_critical: f32,
    pub memory_warning: f32,
    pub memory_critical: f32,
    pub temp_warning: f32,
    pub temp_critical: f32,
}

#[derive(Debug, Clone)]
pub struct AlertConfig {
    pub recovery_seconds: i64,
    pub buffer_size: usize,
}

#[derive(Debug, Clone)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub bot_token: String,
    pub chat_id: String,
}

pub fn load() -> AppConfig {
    AppConfig {
        mqtt: MqttConfig {
            host: env::var("MQTT_HOST").unwrap_or_else(|_| "broker.hivemq.com".to_string()),
            port: env::var("MQTT_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1883),
            client_id: env::var("MQTT_CLIENT_ID")
                .unwrap_or_else(|_| "cloudmetric-middle".to_string()),
            topic_prefix: env::var("MQTT_TOPIC_PREFIX")
                .unwrap_or_else(|_| "cloudmetric".to_string()),
        },
        thresholds: ThresholdsConfig {
            cpu_warning: env::var("THRESHOLD_CPU_WARNING")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(70.0),
            cpu_critical: env::var("THRESHOLD_CPU_CRITICAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(90.0),
            memory_warning: env::var("THRESHOLD_MEMORY_WARNING")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(70.0),
            memory_critical: env::var("THRESHOLD_MEMORY_CRITICAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(90.0),
            temp_warning: env::var("THRESHOLD_TEMP_WARNING")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(75.0),
            temp_critical: env::var("THRESHOLD_TEMP_CRITICAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(85.0),
        },
        alerts: AlertConfig {
            recovery_seconds: env::var("ALERT_RECOVERY_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(120),
            buffer_size: env::var("ALERT_BUFFER_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
        },
        telegram: TelegramConfig {
            enabled: env::var("TELEGRAM_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
            bot_token: env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default(),
            chat_id: env::var("TELEGRAM_CHAT_ID").unwrap_or_default(),
        },
    }
}
