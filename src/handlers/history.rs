use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};

use super::AppState;
use crate::models::{HistoryQuery, HistoryResponse};

pub async fn history_handler(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>, StatusCode> {
    let db = state
        .database
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let range_seconds: u64 = match query.range.as_deref() {
        Some("1h") => 3600,
        Some("6h") => 21600,
        Some("24h") | Some("1d") => 86400,
        Some("7d") => 604800,
        Some("30d") => 2592000,
        _ => 3600,
    };

    let metric = query.metric.as_deref().unwrap_or("cpu");

    Ok(Json(db.query_history(metric, range_seconds)))
}
