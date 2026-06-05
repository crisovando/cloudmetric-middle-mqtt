use std::sync::Arc;

use crate::{
    app_state::AppState,
    domain::command::ServerCommand,
    mqtt::topics::Topics,
};
use rumqttc::{AsyncClient, QoS};
use tokio::sync::Mutex;

pub async fn send_command(
    client: &Arc<Mutex<AsyncClient>>,
    cmd: &ServerCommand,
    topics: &Topics,
) {
    let client = client.lock().await;
    match cmd {
        ServerCommand::SetFailureProbability {
            server_id,
            probability,
        } => {
            let payload = serde_json::json!({
                "server_id": server_id,
                "command": "set_failure_probability",
                "value": probability
            });
            publish_to_control(&client, server_id, &payload, topics).await;
        }
        ServerCommand::SetMetric {
            server_id,
            metric,
            value,
        } => {
            let payload = serde_json::json!({
                "server_id": server_id,
                "command": "set_metric",
                "metric": metric,
                "value": value
            });
            publish_to_control(&client, server_id, &payload, topics).await;
        }
        ServerCommand::ReleaseMetric { server_id, metric } => {
            let payload = serde_json::json!({
                "server_id": server_id,
                "command": "release_metric",
                "metric": metric
            });
            publish_to_control(&client, server_id, &payload, topics).await;
        }
        ServerCommand::CreateServer { server_id, name } => {
            let payload = serde_json::json!({
                "command": "create_server",
                "server_id": server_id,
                "name": name
            });
            publish_fleet(&client, &payload, topics).await;
        }
        ServerCommand::DeleteServer { server_id } => {
            let payload = serde_json::json!({
                "command": "delete_server",
                "server_id": server_id
            });
            publish_fleet(&client, &payload, topics).await;
        }
    }
}

async fn publish_to_control(
    client: &AsyncClient,
    server_id: &str,
    payload: &serde_json::Value,
    topics: &Topics,
) {
    let topic = topics.control_topic(server_id);
    let bytes = payload.to_string().into_bytes();
    if let Err(e) = client.publish(&topic, QoS::AtLeastOnce, false, bytes).await {
        eprintln!("Failed to publish command: {}", e);
    }
}

async fn publish_fleet(client: &AsyncClient, payload: &serde_json::Value, topics: &Topics) {
    let bytes = payload.to_string().into_bytes();
    if let Err(e) = client
        .publish(&topics.fleet_topic(), QoS::AtLeastOnce, false, bytes)
        .await
    {
        eprintln!("Failed to publish fleet command: {}", e);
    }
}

pub async fn publish_config_snapshot(state: &Arc<AppState>) {
    let client = state.mqtt_client.lock().await;
    let topics = Topics::new(state.config.mqtt.topic_prefix.clone());

    let servers: Vec<serde_json::Value> = state
        .servers
        .iter()
        .map(|entry| {
            serde_json::json!({
                "id": entry.value().server_id,
                "name": entry.value().name,
            })
        })
        .collect();

    let payload = serde_json::json!({ "servers": servers });
    let bytes = payload.to_string().into_bytes();

    if let Err(e) = client
        .publish(&topics.config_topic(), QoS::AtLeastOnce, true, bytes)
        .await
    {
        eprintln!("Failed to publish config snapshot: {}", e);
    }
}
