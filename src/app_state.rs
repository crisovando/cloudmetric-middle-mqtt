use std::collections::VecDeque;
use std::sync::Arc;

use dashmap::DashMap;
use libsql::Connection;
use tokio::sync::broadcast;
use tokio::sync::{mpsc, Mutex, RwLock};

use crate::alerts::dispatcher::AlertEvent;
use crate::domain::alert::Alert;
use crate::domain::state::ServerState;
use crate::infrastructure::config::AppConfig;
use crate::websocket::dto::WsEvent;

#[derive(Clone)]
pub struct AppState {
    pub servers: Arc<DashMap<String, ServerState>>,
    pub alerts: Arc<RwLock<VecDeque<Alert>>>,
    pub broadcaster: broadcast::Sender<WsEvent>,
    pub mqtt_client: Arc<Mutex<rumqttc::AsyncClient>>,
    pub db: Connection,
    pub config: Arc<AppConfig>,
    pub alert_sender: mpsc::Sender<AlertEvent>,
}
