use std::sync::Arc;

use crate::{
    app_state::AppState,
    domain::command::ServerCommand,
    mqtt::topics::{CONFIG_TOPIC, FLEET_TOPIC, control_topic},
};
use rumqttc::{AsyncClient, QoS};

pub async fn send_command(client: &AsyncClient, cmd: &ServerCommand) {
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
            publish_to_control(client, server_id, &payload).await;
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
            publish_to_control(client, server_id, &payload).await;
        }
        ServerCommand::ReleaseMetric { server_id, metric } => {
            let payload = serde_json::json!({
                "server_id": server_id,
                "command": "release_metric",
                "metric": metric
            });
            publish_to_control(client, server_id, &payload).await;
        }
        ServerCommand::CreateServer { server_id, name } => {
            let payload = serde_json::json!({
                "command": "create_server",
                "server_id": server_id,
                "name": name
            });
            publish_fleet(client, &payload).await;
        }
        ServerCommand::DeleteServer { server_id } => {
            let payload = serde_json::json!({
                "command": "delete_server",
                "server_id": server_id
            });
            publish_fleet(client, &payload).await;
        }
    }
}

async fn publish_to_control(client: &AsyncClient, server_id: &str, payload: &serde_json::Value) {
    let topic = control_topic(server_id);
    let bytes = payload.to_string().into_bytes();
    if let Err(e) = client.publish(&topic, QoS::AtLeastOnce, false, bytes).await {
        eprintln!("Failed to publish command: {}", e);
    }
}

async fn publish_fleet(client: &AsyncClient, payload: &serde_json::Value) {
    let bytes = payload.to_string().into_bytes();
    if let Err(e) = client
        .publish(FLEET_TOPIC, QoS::AtLeastOnce, false, bytes)
        .await
    {
        eprintln!("Failed to publish fleet command: {}", e);
    }
}

pub async fn publish_config_snapshot(state: &Arc<AppState>) {
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

    if let Err(e) = state
        .mqtt_client
        .publish(CONFIG_TOPIC, QoS::AtLeastOnce, true, bytes)
        .await
    {
        eprintln!("Failed to publish config snapshot: {}", e);
    }
}
