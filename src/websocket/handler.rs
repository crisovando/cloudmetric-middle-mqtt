use std::sync::Arc;
use std::time::Duration;

use crate::{
    app_state::AppState,
    domain::command::ServerCommand,
    mqtt::{publisher::send_command, topics::Topics},
    websocket::dto::{WsAlert, WsEvent, WsIncoming, WsServerUpdate, WsSnapshot},
};
use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use futures::{SinkExt, stream::StreamExt};
use tokio::sync::broadcast;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    println!("WebSocket upgrade request received");
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, app_state: Arc<AppState>) {
    println!("WebSocket connection established");

    let (mut sender, mut receiver) = socket.split();

    let alerts = {
        let alerts_lock = app_state.alerts.read().await;
        alerts_lock.iter().map(WsAlert::from).collect()
    };

    let snapshot = WsSnapshot {
        servers: app_state
            .servers
            .iter()
            .map(|entry| WsServerUpdate::from(entry.value()))
            .collect(),
        alerts,
    };

    let server_count = snapshot.servers.len();
    let alert_count = snapshot.alerts.len();

    if let Ok(json) = serde_json::to_string(&WsEvent::Snapshot(snapshot))
        && sender.send(Message::Text(json.into())).await.is_err()
    {
        eprintln!("Failed to send snapshot, client disconnected immediately");
        return;
    }

    println!(
        "Snapshot sent: {} servers, {} alerts",
        server_count, alert_count
    );

    let mut rx = app_state.broadcaster.subscribe();
    let mut ping_interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(ws_event) => {
                        if let Ok(json) = serde_json::to_string(&ws_event)
                            && sender.send(Message::Text(json.into())).await.is_err()
                        {
                            println!("Client disconnected (send failed)");
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("WS client lagged, skipped {} messages", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        println!("Broadcast channel closed");
                        break;
                    }
                }
            }
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_incoming_message(&app_state, &text).await;
                    }
                    Some(Ok(Message::Close(_))) => {
                        println!("Client sent close frame");
                        break;
                    }
                    None => {
                        println!("Client disconnected (connection closed)");
                        break;
                    }
                    _ => {}
                }
            }
            _ = ping_interval.tick() => {
                if sender.send(Message::Ping(vec![].into())).await.is_err() {
                    println!("Client disconnected (ping failed)");
                    break;
                }
            }
        }
    }

    println!("WebSocket connection closed");
}

async fn handle_incoming_message(app_state: &AppState, text: &str) {
    match serde_json::from_str::<WsIncoming>(text) {
        Ok(WsIncoming::Command(cmd_dto)) => match ServerCommand::try_from(cmd_dto) {
            Ok(domain_command) => {
                let topics = Topics::new(app_state.config.mqtt.topic_prefix.clone());
                send_command(&app_state.mqtt_client, &domain_command, &topics).await;
            }
            Err(validation_error) => {
                eprintln!("Comando inválido recibido: {}", validation_error);
            }
        },
        Err(e) => eprintln!("Error parseando JSON del WS: {}", e),
    }
}
