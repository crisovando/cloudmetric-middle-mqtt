use std::sync::Arc;
use tokio::sync::mpsc;

use crate::alerts::{repository, telegram};
use crate::app_state::AppState;
use crate::domain::alert::Alert;
use crate::websocket::dto::{WsAlert, WsEvent, WsRecovery};
use chrono::Utc;

pub enum AlertEvent {
    New(Alert),
    Recovery { server_id: String, server_name: String },
}

pub fn start(state: Arc<AppState>, mut rx: mpsc::Receiver<AlertEvent>) {
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                AlertEvent::New(alert) => {
                    handle_new_alert(&state, &alert).await;
                }
                AlertEvent::Recovery { server_id, server_name } => {
                    handle_recovery(&state, &server_id, &server_name).await;
                }
            }
        }
    });
}

async fn handle_new_alert(state: &AppState, alert: &Alert) {
    if let Err(e) = repository::insert_alert(&state.db, alert).await {
        eprintln!("Failed to insert alert to DB: {}", e);
    }

    {
        let mut alerts = state.alerts.write().await;
        alerts.push_back(alert.clone());
        while alerts.len() > state.config.alerts.buffer_size {
            alerts.pop_front();
        }
    }

    let _ = state.broadcaster.send(WsEvent::Alert(WsAlert::from(alert)));

    if state.config.telegram.enabled {
        let config = state.config.clone();
        let alert = alert.clone();
        tokio::spawn(async move {
            if let Err(e) = telegram::send_alert(&config.telegram, &alert).await {
                eprintln!("Failed to send Telegram alert: {}", e);
            }
        });
    }
}

async fn handle_recovery(state: &AppState, server_id: &str, server_name: &str) {
    let now = Utc::now();

    if let Err(e) = repository::resolve_alerts_by_server(&state.db, server_id, now).await {
        eprintln!("Failed to resolve alerts in DB: {}", e);
    }

    {
        let mut alerts = state.alerts.write().await;
        for alert in alerts.iter_mut() {
            if alert.server_id == server_id && alert.resolved_at.is_none() {
                alert.resolved_at = Some(now);
            }
        }
    }

    let _ = state.broadcaster.send(WsEvent::Recovery(WsRecovery {
        server_id: server_id.to_string(),
        server_name: server_name.to_string(),
    }));
}
