use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::time::{Duration, interval};

use crate::alerts::dispatcher::AlertEvent;
use crate::alerts::engine::{AlertContext, evaluate, get_critical_metrics};
use crate::app_state::AppState;
use crate::domain::status::HealthStatus;
use crate::websocket::dto::{WsEvent, WsServerUpdate};

const CHECK_INTERVAL_SECS: u64 = 5;
const OFFLINE_THRESHOLD_SECS: i64 = 10;

pub fn run_offline_checker(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(CHECK_INTERVAL_SECS));
        loop {
            ticker.tick().await;

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            let mut servers_went_offline = Vec::new();

            let mut alerts_to_send = Vec::new();

            for mut entry in state.servers.iter_mut() {
                let server_state = entry.value_mut();
                let elapsed = now - server_state.health.timestamp.timestamp();

                if elapsed > OFFLINE_THRESHOLD_SECS && server_state.status != HealthStatus::Offline
                {
                    server_state.status = HealthStatus::Offline;

                    let critical_metrics = get_critical_metrics(
                        &server_state.health,
                        &state.config.thresholds,
                    );

                    let decision = evaluate(AlertContext {
                        server_id: &server_state.server_id,
                        server_name: &server_state.name,
                        old_alert_active: server_state.alert_active,
                        old_recovery_since: server_state.recovery_since,
                        old_last_alert_at: server_state.last_alert_at,
                        new_status: &HealthStatus::Offline,
                        critical_metrics: &critical_metrics,
                        alert_config: &state.config.alerts,
                    });

                    server_state.alert_active = decision.new_alert_active;
                    server_state.recovery_since = decision.new_recovery_since;
                    server_state.last_alert_at = decision.new_last_alert_at;

                    for alert in decision.alerts {
                        alerts_to_send.push(AlertEvent::New(alert));
                    }

                    servers_went_offline.push(server_state.clone());
                }
            }

            for alert_event in alerts_to_send {
                let _ = state.alert_sender.send(alert_event).await;
            }

            for server in servers_went_offline {
                let _ = state
                    .broadcaster
                    .send(WsEvent::Update(WsServerUpdate::from(&server)));
            }
        }
    });
}
