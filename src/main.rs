mod device_health;
mod validation;

use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::error::Error;
use std::time::Duration;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use device_health::ServerHealth;
use validation::is_device_unhealthy;

static DATA_TOPIC: &str = "cloudmetric/simulator/health_data";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, _rx) = broadcast::channel::<ServerHealth>(16);

    let mut mqttoptions = MqttOptions::new("middle-mqtt", "broker.hivemq.com", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    let tx_mqtt = tx.clone();

    tokio::spawn(async move {
        let _ = client.subscribe(DATA_TOPIC, QoS::AtMostOnce).await;

        loop {
            match eventloop.poll().await {
                Ok(notification) => {
                    if let Event::Incoming(Packet::Publish(publish)) = notification {
                        match serde_json::from_slice::<ServerHealth>(&publish.payload) {
                            Ok(data) => {
                                if let Err(e) = tx_mqtt.send(data) {
                                    eprintln!("Error al enviar al canal interno: {}", e);
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "Error de parseo JSON: {}. Datos: {:?}",
                                    e, publish.payload
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error de conexión MQTT: {}. Reintentando...", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });

    let app = Router::new()
        .route("/ws", get(move |ws| ws_handler(ws, tx.clone())))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    println!("🚀 Servidor en puerto 8080. WebSocket en /ws");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, tx: broadcast::Sender<ServerHealth>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, tx))
}

async fn handle_socket(mut socket: WebSocket, tx: broadcast::Sender<ServerHealth>) {
    let mut rx = tx.subscribe();

    while let Ok(mut data) = rx.recv().await {
        if is_device_unhealthy(&data.health) {
            data.health.mark_unhealthy();
        } else {
            data.health.status = "healthy".to_string();
        }

        match serde_json::to_string(&data) {
            Ok(json) => {
                if socket.send(Message::Text(json)).await.is_err() {
                    break; // El cliente React se desconectó
                }
            }
            Err(e) => eprintln!("Error al serializar para WebSocket: {}", e),
        }
    }
}
