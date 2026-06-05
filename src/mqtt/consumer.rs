use std::{sync::Arc, time::Duration};

use chrono::Utc;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};

use crate::alerts::dispatcher::AlertEvent;
use crate::alerts::engine::{AlertContext, evaluate, get_critical_metrics};
use crate::app_state::AppState;
use crate::domain::{health::DeviceHealth, state::ServerState};
use crate::mqtt::topics::Topics;
use crate::validation::engine::determine_status;
use crate::websocket::dto::{WsEvent, WsServerUpdate};

pub async fn init_mqtt(
    host: &str,
    port: u16,
    client_id: &str,
    topics: &Topics,
) -> (AsyncClient, rumqttc::EventLoop) {
    let client_id = format!("{}_{}", client_id, Utc::now().timestamp_millis());
    let mut mqttoptions = MqttOptions::new(client_id, host, port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

    client
        .subscribe(topics.health_wildcard(), QoS::AtMostOnce)
        .await
        .unwrap();

    (client, eventloop)
}

pub fn spawn_event_loop(app_state: Arc<AppState>, eventloop: rumqttc::EventLoop) {
    tokio::spawn(async move {
        run_event_loop(app_state, eventloop).await;
    });
}

async fn run_event_loop(app_state: Arc<AppState>, mut eventloop: rumqttc::EventLoop) {
    loop {
        match eventloop.poll().await {
            Ok(notification) => {
                if let Event::Incoming(Packet::Publish(publish)) = notification {
                    handle_publish(&app_state, &publish.topic, &publish.payload).await;
                }
            }
            Err(e) => {
                eprintln!("MQTT error: {:?}, reconnecting in 5s...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;

                let topics = Topics::new(app_state.config.mqtt.topic_prefix.clone());
                let (new_client, new_eventloop) = init_mqtt(
                    &app_state.config.mqtt.host,
                    app_state.config.mqtt.port,
                    &app_state.config.mqtt.client_id,
                    &topics,
                )
                .await;
                println!("MQTT reconnected successfully");
                *app_state.mqtt_client.lock().await = new_client;
                eventloop = new_eventloop;
            }
        }
    }
}

pub async fn handle_publish(app_state: &AppState, topic: &str, payload: &[u8]) {
    let topics = Topics::new(app_state.config.mqtt.topic_prefix.clone());
    if let Some(server_id) = topics.parse_health_topic(topic) {
        if let Ok(health) = serde_json::from_slice::<DeviceHealth>(payload) {
            let new_status = determine_status(&health, &app_state.config.thresholds);
            let critical_metrics = get_critical_metrics(&health, &app_state.config.thresholds);

            let (old_alert_active, old_recovery_since, old_last_alert_at, old_name) =
                app_state
                    .servers
                    .get(&server_id)
                    .map(|entry| {
                        (
                            entry.alert_active,
                            entry.recovery_since,
                            entry.last_alert_at,
                            entry.name.clone(),
                        )
                    })
                    .unwrap_or((false, None, None, server_id.clone()));

            let decision = evaluate(AlertContext {
                server_id: &server_id,
                server_name: &old_name,
                old_alert_active,
                old_recovery_since,
                old_last_alert_at,
                new_status: &new_status,
                critical_metrics: &critical_metrics,
                alert_config: &app_state.config.alerts,
            });

            let server_state = ServerState {
                server_id: server_id.clone(),
                health,
                status: new_status,
                name: old_name.clone(),
                alert_active: decision.new_alert_active,
                recovery_since: decision.new_recovery_since,
                last_alert_at: decision.new_last_alert_at,
            };

            app_state
                .servers
                .insert(server_id.clone(), server_state.clone());

            for alert in decision.alerts {
                let _ = app_state.alert_sender.send(AlertEvent::New(alert)).await;
            }

            if decision.recovered {
                let _ = app_state
                    .alert_sender
                    .send(AlertEvent::Recovery {
                        server_id: server_state.server_id.clone(),
                        server_name: server_state.name.clone(),
                    })
                    .await;
            }

            let _ = app_state
                .broadcaster
                .send(WsEvent::Update(WsServerUpdate::from(&server_state)));
        } else {
            eprintln!("Failed to parse health payload for topic: {}", topic);
            println!("Payload was: {}", String::from_utf8_lossy(payload));
        }
    }
}
