use crate::app_state::AppState;
use crate::websocket::dto::WsAlert;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct AlertsQuery {
    pub server_id: Option<String>,
}

pub async fn get_alerts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AlertsQuery>,
) -> Json<Vec<WsAlert>> {
    let alerts = state.alerts.read().await;

    let filtered: Vec<WsAlert> = alerts
        .iter()
        .filter(|a| {
            query
                .server_id
                .as_ref()
                .is_none_or(|id| &a.server_id == id)
        })
        .map(WsAlert::from)
        .collect();

    Json(filtered)
}
