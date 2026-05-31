use crate::{
    app_state::AppState,
    domain::{command::ServerCommand, state::ServerState, status::HealthStatus},
    mqtt::publisher::{publish_config_snapshot, send_command},
    websocket::dto::{WsEvent, WsServerUpdate},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CreateServerReq {
    name: String,
}

pub async fn create_server(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateServerReq>,
) -> Result<StatusCode, (StatusCode, String)> {
    // 1. Guardar en Turso y obtener el ID
    let new_id = crate::state::repository::add_server(&state.db, &payload.name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 2. Armar el dominio
    let mut new_state = ServerState::default();
    new_state.server_id = new_id.to_string();
    new_state.name = payload.name;
    new_state.status = HealthStatus::Offline;

    // 3. Guardar en memoria (DashMap)
    state
        .servers
        .insert(new_state.server_id.clone(), new_state.clone());

    // 4. Avisar a todos los clientes por WebSocket
    let update_dto = WsServerUpdate::from(&new_state);
    let _ = state.broadcaster.send(WsEvent::Update(update_dto));

    // 5. Notificar a simulators via MQTT (fleet + config snapshot)
    let cmd = ServerCommand::CreateServer {
        server_id: new_state.server_id.clone(),
        name: new_state.name.clone(),
    };
    send_command(&state.mqtt_client, &cmd).await;
    publish_config_snapshot(&state).await;

    Ok(StatusCode::CREATED)
}

pub async fn delete_server(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    // 1. Borrar de Turso
    crate::state::repository::delete_server(&state.db, &server_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 2. Borrar de memoria
    state.servers.remove(&server_id);

    // 3. Avisar a todos por WebSocket (Evento Deleted)
    let _ = state.broadcaster.send(WsEvent::Deleted {
        server_id: server_id.clone(),
    });

    // 4. Notificar a simulators via MQTT (fleet + config snapshot)
    let cmd = ServerCommand::DeleteServer {
        server_id: server_id.clone(),
    };
    send_command(&state.mqtt_client, &cmd).await;
    publish_config_snapshot(&state).await;

    Ok(StatusCode::NO_CONTENT)
}
