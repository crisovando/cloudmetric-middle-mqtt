mod api;
mod app_state;
mod domain;
mod infrastructure;
mod state;
mod validation;

mod mqtt;
mod websocket;

use anyhow::Result;
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::broadcast;

use app_state::AppState;

use crate::{
    mqtt::{consumer, publisher::publish_config_snapshot},
    websocket::dto::WsEvent,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let (tx, _) = broadcast::channel::<WsEvent>(1024);
    let (mqtt_client, eventloop) = consumer::init_mqtt().await;
    let db_conn = infrastructure::db::init_db()
        .await
        .expect("Error al conectar a la db");

    //
    // SHARED APPLICATION STATE
    //
    let state = Arc::new(AppState {
        servers: Arc::new(DashMap::new()),
        broadcaster: tx,
        mqtt_client,
        db: db_conn,
    });

    if let Ok(servers_from_db) = crate::state::repository::load_all_servers(&state.db).await {
        for server in servers_from_db {
            state.servers.insert(server.server_id.clone(), server);
        }
        println!("✅ Servidores cargados en memoria.");
    }

    publish_config_snapshot(&state).await;

    consumer::spawn_event_loop(state.clone(), eventloop);
    websocket::gateway::start(state.clone()).await?;

    //
    // OFFLINE CHECKERS
    //
    state::offline_checkers::run_offline_checkers(state.clone());

    Ok(())
}
