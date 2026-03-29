use crate::handlers::AppState;
use crate::models::HealthResponse;
use axum::{extract::State, response::Json};
use chrono::Utc;

/// Health check handler
pub async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    let uptime = state.ws_state.uptime_seconds();

    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        timestamp: Utc::now(),
    })
}
