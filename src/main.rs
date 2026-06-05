mod alerts;
mod api;
mod app_state;
mod domain;
mod infrastructure;
mod state;
mod validation;

mod mqtt;
mod websocket;

use std::collections::VecDeque;
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

use app_state::AppState;

use crate::alerts::dispatcher::{self, AlertEvent};
use crate::infrastructure::config;
use crate::mqtt::{consumer, publisher::publish_config_snapshot};
use crate::websocket::dto::WsEvent;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting CloudMetric MQTT middleware...");
    
    dotenvy::dotenv().ok();
    println!("Environment variables loaded");

    let app_config = config::load();
    println!("Configuration loaded");

    let (tx, _) = broadcast::channel::<WsEvent>(1024);
    let (alert_tx, alert_rx) = mpsc::channel::<AlertEvent>(1024);
    println!("Broadcast channels created");

    println!("Connecting to MQTT broker at {}:{}...", app_config.mqtt.host, app_config.mqtt.port);
    let topics = mqtt::topics::Topics::new(app_config.mqtt.topic_prefix.clone());
    let (mqtt_client, eventloop) = consumer::init_mqtt(
        &app_config.mqtt.host,
        app_config.mqtt.port,
        &app_config.mqtt.client_id,
        &topics,
    )
    .await;
    println!("MQTT client connected (topic prefix: {})", app_config.mqtt.topic_prefix);

    println!("Connecting to database...");
    let db_conn = infrastructure::db::init_db()
        .await
        .expect("Error al conectar a la db");
    println!("Database connected");

    println!("Loading alerts from database...");
    let alerts_buffer = {
        match alerts::repository::load_recent_alerts(&db_conn, app_config.alerts.buffer_size).await {
            Ok(mut alerts) => {
                let count = alerts.len();
                alerts.reverse();
                println!("✅ Loaded {} alerts", count);
                alerts.into_iter().collect::<VecDeque<_>>()
            }
            Err(e) => {
                eprintln!("Error loading alerts from DB: {}", e);
                VecDeque::new()
            }
        }
    };

    let state = Arc::new(AppState {
        servers: Arc::new(DashMap::new()),
        alerts: Arc::new(tokio::sync::RwLock::new(alerts_buffer)),
        broadcaster: tx,
        mqtt_client: Arc::new(Mutex::new(mqtt_client)),
        db: db_conn,
        config: Arc::new(app_config),
        alert_sender: alert_tx,
    });

    println!("Loading servers from database...");
    if let Ok(servers_from_db) = crate::state::repository::load_all_servers(&state.db).await {
        let count = servers_from_db.len();
        for server in servers_from_db {
            state.servers.insert(server.server_id.clone(), server);
        }
        println!("Loaded {} servers", count);
    }

    println!("Publishing config snapshot to MQTT...");
    publish_config_snapshot(&state).await;
    println!("Config snapshot published");

    println!("Starting alert dispatcher...");
    dispatcher::start(state.clone(), alert_rx);
    println!("Alert dispatcher started");

    println!("Starting offline checker...");
    state::offline_checker::run_offline_checker(state.clone());
    println!("Offline checker started");

    println!("Starting MQTT event loop...");
    consumer::spawn_event_loop(state.clone(), eventloop);
    println!("MQTT event loop started");

    println!("Starting WebSocket server...");
    websocket::gateway::start(state.clone()).await?;

    Ok(())
}
