use std::sync::Arc;

use crate::{
    app_state::AppState,
    domain::command::ServerCommand,
    mqtt::publisher::send_command,
    websocket::dto::{WsEvent, WsIncoming, WsServerUpdate, WsSnapshot},
};
use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use futures::{SinkExt, stream::StreamExt};

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, app_state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    let snapshot = WsSnapshot {
        servers: app_state
            .servers
            .iter()
            .map(|entry| WsServerUpdate::from(entry.value()))
            .collect(),
    };

    if let Ok(json) = serde_json::to_string(&WsEvent::Snapshot(snapshot)) {
        let _ = sender.send(Message::Text(json.into())).await;
    }

    let mut rx = app_state.broadcaster.subscribe();

    let send_task = tokio::spawn(async move {
        // rx.recv() ahora devuelve un WsEvent puro
        while let Ok(ws_event) = rx.recv().await {
            // Serializamos directamente el evento (sea Update o Deleted)
            if let Ok(json) = serde_json::to_string(&ws_event) {
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            handle_incoming_message(&app_state, &text).await;
        }
    }

    send_task.abort();
}

async fn handle_incoming_message(app_state: &AppState, text: &str) {
    match serde_json::from_str::<WsIncoming>(text) {
        Ok(WsIncoming::Command(cmd_dto)) => match ServerCommand::try_from(cmd_dto) {
            Ok(domain_command) => {
                send_command(&app_state.mqtt_client, &domain_command).await;
            }
            Err(validation_error) => {
                eprintln!("Comando inválido recibido: {}", validation_error);
            }
        },
        Err(e) => eprintln!("Error parseando JSON del WS: {}", e),
    }
}
