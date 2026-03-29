use axum::{extract::State, Json};

use super::AppState;
use crate::metrics::collect_gpu_metrics;
use crate::models::GpuMetrics;

pub async fn gpu_handler(
    State(_state): State<AppState>,
) -> Json<GpuMetrics> {
    Json(collect_gpu_metrics())
}
