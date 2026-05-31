use std::sync::Arc;

use dashmap::DashMap;
use libsql::Connection;
use tokio::sync::broadcast;

use crate::{domain::state::ServerState, websocket::dto::WsEvent};

#[derive(Clone)]
pub struct AppState {
    pub servers: Arc<DashMap<String, ServerState>>,
    pub broadcaster: broadcast::Sender<WsEvent>,
    pub mqtt_client: rumqttc::AsyncClient,
    pub db: Connection,
}
