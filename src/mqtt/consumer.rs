use std::{sync::Arc, time::Duration};

use chrono::Utc;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};

use crate::{
    app_state::AppState,
    domain::{health::DeviceHealth, state::ServerState},
    mqtt::topics::{HEALTH_WILDCARD, parse_health_topic},
    validation::engine,
    websocket::dto::{WsEvent, WsServerUpdate},
};

pub async fn init_mqtt() -> (AsyncClient, rumqttc::EventLoop) {
    let client_id = format!("mqtt_consumer_{}", Utc::now().timestamp_millis());
    let mut mqttoptions = MqttOptions::new(client_id, "broker.hivemq.com", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

    client
        .subscribe(HEALTH_WILDCARD, QoS::AtMostOnce)
        .await
        .unwrap();

    (client, eventloop)
}

pub fn spawn_event_loop(app_state: Arc<AppState>, mut eventloop: rumqttc::EventLoop) {
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(notification) => {
                    if let Event::Incoming(Packet::Publish(publish)) = notification {
                        handle_publish(&app_state, &publish.topic, &publish.payload).await;
                    }
                }
                Err(e) => {
                    eprintln!("MQTT error: {:?}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });
}

pub async fn handle_publish(app_state: &AppState, topic: &str, payload: &[u8]) {
    if let Some(server_id) = parse_health_topic(topic) {
        if let Ok(health) = serde_json::from_slice::<DeviceHealth>(payload) {
            let status = engine::determine_status(&health);

            let server_state = ServerState {
                server_id: server_id.clone(),
                health: health,
                status: status,
                name: server_id.clone(),
            };

            app_state.servers.insert(server_id, server_state.clone());

            let _ = app_state
                .broadcaster
                .send(WsEvent::Update(WsServerUpdate::from(&server_state)));
        } else {
            eprintln!("Failed to parse health payload for topic: {}", topic);
            println!("Payload was: {}", String::from_utf8_lossy(payload));
        }
    }
}
