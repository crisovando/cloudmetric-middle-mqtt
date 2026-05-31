use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::time::{Duration, interval};

use crate::app_state::AppState;
use crate::domain::status::HealthStatus;
use crate::websocket::dto::WsEvent;

const CHECK_INTERVAL_SECS: u64 = 5;
const OFFLINE_THRESHOLD_SECS: i64 = 10;

pub fn run_offline_checkers(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(CHECK_INTERVAL_SECS));
        loop {
            ticker.tick().await;

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            let mut servers_went_offline = Vec::new();

            for mut entry in state.servers.iter_mut() {
                let server_state = entry.value_mut();
                let elapsed = now - server_state.health.timestamp.timestamp();

                if elapsed > OFFLINE_THRESHOLD_SECS && server_state.status != HealthStatus::Offline
                {
                    server_state.status = HealthStatus::Offline;
                    servers_went_offline.push(server_state.clone());
                }
            }

            for server in servers_went_offline {
                let update_dto = crate::websocket::dto::WsServerUpdate::from(&server);
                if let Err(e) = state.broadcaster.send(WsEvent::Update(update_dto)) {
                    eprintln!("Error emitiendo desconexión: {}", e);
                }
            }
        }
    });
}
