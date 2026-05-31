pub mod servers;

use crate::app_state::AppState;
use axum::Router;
use std::sync::Arc;

pub fn build_api_router(_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/servers", axum::routing::post(servers::create_server))
        .route(
            "/servers/{server_id}",
            axum::routing::delete(servers::delete_server),
        )
}
