use crate::{
    app_state::AppState,
    domain::{command::ServerCommand, state::ServerState, status::HealthStatus},
    mqtt::{publisher::{publish_config_snapshot, send_command}, topics::Topics},
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
    let new_id = crate::state::repository::add_server(&state.db, &payload.name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let new_state = ServerState {
        server_id: new_id.to_string(),
        name: payload.name,
        status: HealthStatus::Offline,
        ..Default::default()
    };

    state
        .servers
        .insert(new_state.server_id.clone(), new_state.clone());

    let update_dto = WsServerUpdate::from(&new_state);
    let _ = state.broadcaster.send(WsEvent::Update(update_dto));

    let cmd = ServerCommand::CreateServer {
        server_id: new_state.server_id.clone(),
        name: new_state.name.clone(),
    };
    let topics = Topics::new(state.config.mqtt.topic_prefix.clone());
    send_command(&state.mqtt_client, &cmd, &topics).await;
    publish_config_snapshot(&state).await;

    Ok(StatusCode::CREATED)
}

pub async fn delete_server(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    crate::state::repository::delete_server(&state.db, &server_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state.servers.remove(&server_id);

    let _ = state.broadcaster.send(WsEvent::Deleted {
        server_id: server_id.clone(),
    });

    let cmd = ServerCommand::DeleteServer {
        server_id: server_id.clone(),
    };
    let topics = Topics::new(state.config.mqtt.topic_prefix.clone());
    send_command(&state.mqtt_client, &cmd, &topics).await;
    publish_config_snapshot(&state).await;

    Ok(StatusCode::NO_CONTENT)
}
