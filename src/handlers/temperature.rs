use axum::{extract::State, Json};

use super::AppState;
use crate::metrics::collect_temperature_metrics;
use crate::models::TemperatureMetrics;

pub async fn temperature_handler(
    State(_state): State<AppState>,
) -> Json<TemperatureMetrics> {
    Json(collect_temperature_metrics())
}
