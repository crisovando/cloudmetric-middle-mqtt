use axum::{Router, routing::get};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::api::build_api_router;
use crate::app_state::AppState;
use crate::websocket::handler::ws_handler;

async fn root_handler() -> &'static str {
    println!("Root HTTP request received");
    "ok"
}

pub async fn start(app_state: Arc<AppState>) -> anyhow::Result<()> {
    println!("Building WebSocket router...");

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/ws", get(ws_handler))
        .merge(build_api_router(app_state.clone()))
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    println!("Binding to 0.0.0.0:8080...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    let addr = listener.local_addr()?;
    println!("WebSocket server ready on ws://{}{}", addr, "/ws");
    println!("Waiting for connections...");

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", e);
        return Err(e.into());
    }

    Ok(())
}
