use crate::handlers::AppState;
use crate::models::{CleanupRequest, KillRequest, KillResponse, CleanupResponse, KillStaleRequest};
use crate::actions::{cleanup_processes, kill_process};
use axum::{
    extract::State,
    Json,
};
use axum::http::StatusCode;

/// Kill a single process
pub async fn kill_handler(
    State(state): State<AppState>,
    Json(request): Json<KillRequest>,
) -> Result<Json<KillResponse>, StatusCode> {
    match kill_process(request.pid, request.signal, &state.config).await {
        Ok(message) => Ok(Json(KillResponse {
            success: true,
            message,
        })),
        Err(e) => Ok(Json(KillResponse {
            success: false,
            message: e.to_string(),
        })),
    }
}

/// Kill stale processes
pub async fn kill_stale_handler(
    State(state): State<AppState>,
    Json(request): Json<KillStaleRequest>,
) -> Result<Json<CleanupResponse>, StatusCode> {
    let dry_run = request.dry_run.unwrap_or(false);
    
    match cleanup_processes(
        true, // include_stale
        request.max_age_hours,
        dry_run,
        &state.config,
    ).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Failed to kill stale processes: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Full cleanup (zombies + optional stale)
pub async fn cleanup_handler(
    State(state): State<AppState>,
    Json(request): Json<CleanupRequest>,
) -> Result<Json<CleanupResponse>, StatusCode> {
    let include_stale = request.include_stale.unwrap_or(true);
    let dry_run = request.dry_run.unwrap_or(false);
    
    match cleanup_processes(
        include_stale,
        request.stale_max_age_hours,
        dry_run,
        &state.config,
    ).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Failed to cleanup processes: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
