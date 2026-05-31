use axum::{Router, routing::get};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::app_state::AppState;
use crate::api::build_api_router;
use crate::websocket::handler::ws_handler;

pub async fn start(app_state: Arc<AppState>) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .merge(build_api_router(app_state.clone()))
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;

    println!("🚀 WebSocket server on :8080/ws");

    axum::serve(listener, app).await?;

    Ok(())
}
