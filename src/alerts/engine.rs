use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::alert::{Alert, AlertMetric};
use crate::domain::health::DeviceHealth;
use crate::domain::status::HealthStatus;
use crate::infrastructure::config::{AlertConfig, ThresholdsConfig};

pub struct CriticalMetric {
    pub metric: AlertMetric,
    pub value: f32,
}

pub struct AlertDecision {
    pub alerts: Vec<Alert>,
    pub recovered: bool,
    pub new_alert_active: bool,
    pub new_recovery_since: Option<DateTime<Utc>>,
    pub new_last_alert_at: Option<DateTime<Utc>>,
}

pub struct AlertContext<'a> {
    pub server_id: &'a str,
    pub server_name: &'a str,
    pub old_alert_active: bool,
    pub old_recovery_since: Option<DateTime<Utc>>,
    pub old_last_alert_at: Option<DateTime<Utc>>,
    pub new_status: &'a HealthStatus,
    pub critical_metrics: &'a [CriticalMetric],
    pub alert_config: &'a AlertConfig,
}

pub fn get_critical_metrics(health: &DeviceHealth, config: &ThresholdsConfig) -> Vec<CriticalMetric> {
    let mut metrics = Vec::new();

    if health.cpu >= config.cpu_critical {
        metrics.push(CriticalMetric {
            metric: AlertMetric::Cpu,
            value: health.cpu,
        });
    }
    if health.temp >= config.temp_critical {
        metrics.push(CriticalMetric {
            metric: AlertMetric::Temp,
            value: health.temp,
        });
    }
    if health.memory >= config.memory_critical {
        metrics.push(CriticalMetric {
            metric: AlertMetric::Memory,
            value: health.memory,
        });
    }

    metrics
}

pub fn evaluate(ctx: AlertContext) -> AlertDecision {
    let now = Utc::now();

    match ctx.new_status {
        HealthStatus::Critical => {
            let alerts: Vec<Alert> = ctx
                .critical_metrics
                .iter()
                .map(|cm| Alert {
                    id: Uuid::new_v4(),
                    server_id: ctx.server_id.to_string(),
                    server_name: ctx.server_name.to_string(),
                    timestamp: now,
                    metric: cm.metric.clone(),
                    value: Some(cm.value),
                    resolved_at: None,
                })
                .collect();

            if ctx.old_alert_active {
                AlertDecision {
                    alerts: vec![],
                    recovered: false,
                    new_alert_active: true,
                    new_recovery_since: None,
                    new_last_alert_at: ctx.old_last_alert_at,
                }
            } else {
                AlertDecision {
                    alerts,
                    recovered: false,
                    new_alert_active: true,
                    new_recovery_since: None,
                    new_last_alert_at: Some(now),
                }
            }
        }
        HealthStatus::Offline => {
            if ctx.old_alert_active {
                AlertDecision {
                    alerts: vec![],
                    recovered: false,
                    new_alert_active: true,
                    new_recovery_since: None,
                    new_last_alert_at: ctx.old_last_alert_at,
                }
            } else {
                let alert = Alert {
                    id: Uuid::new_v4(),
                    server_id: ctx.server_id.to_string(),
                    server_name: ctx.server_name.to_string(),
                    timestamp: now,
                    metric: AlertMetric::Offline,
                    value: None,
                    resolved_at: None,
                };

                AlertDecision {
                    alerts: vec![alert],
                    recovered: false,
                    new_alert_active: true,
                    new_recovery_since: None,
                    new_last_alert_at: Some(now),
                }
            }
        }
        HealthStatus::Healthy | HealthStatus::Warning => {
            if !ctx.old_alert_active {
                AlertDecision {
                    alerts: vec![],
                    recovered: false,
                    new_alert_active: false,
                    new_recovery_since: None,
                    new_last_alert_at: ctx.old_last_alert_at,
                }
            } else {
                let recovery_since = ctx.old_recovery_since.unwrap_or(now);
                let elapsed = (now - recovery_since).num_seconds();

                if elapsed >= ctx.alert_config.recovery_seconds {
                    AlertDecision {
                        alerts: vec![],
                        recovered: true,
                        new_alert_active: false,
                        new_recovery_since: None,
                        new_last_alert_at: ctx.old_last_alert_at,
                    }
                } else {
                    AlertDecision {
                        alerts: vec![],
                        recovered: false,
                        new_alert_active: true,
                        new_recovery_since: Some(recovery_since),
                        new_last_alert_at: ctx.old_last_alert_at,
                    }
                }
            }
        }
    }
}
