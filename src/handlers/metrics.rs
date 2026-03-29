use crate::handlers::AppState;
use crate::models::{CpuMetrics, MemoryMetrics, SystemMetrics};
use axum::{
    extract::State,
    response::Json,
};
use axum::http::StatusCode;

/// Get full system metrics
pub async fn system_metrics_handler(
    State(state): State<AppState>,
) -> Result<Json<SystemMetrics>, StatusCode> {
    match state.metrics_collector.collect().await {
        Ok(metrics) => Ok(Json(metrics)),
        Err(e) => {
            tracing::error!("Failed to collect metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get CPU metrics only
pub async fn cpu_metrics_handler(
    State(state): State<AppState>,
) -> Result<Json<CpuMetrics>, StatusCode> {
    match state.metrics_collector.collect().await {
        Ok(metrics) => Ok(Json(metrics.cpu)),
        Err(e) => {
            tracing::error!("Failed to collect CPU metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get memory metrics only
pub async fn memory_metrics_handler(
    State(state): State<AppState>,
) -> Result<Json<MemoryMetrics>, StatusCode> {
    match state.metrics_collector.collect().await {
        Ok(metrics) => Ok(Json(metrics.memory)),
        Err(e) => {
            tracing::error!("Failed to collect memory metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
