use crate::actions::{cleanup_processes, kill_process};
use crate::handlers::AppState;
use crate::models::{
    BatchKillRequest, BatchKillResponse, BatchKillResult, CleanupRequest, CleanupResponse,
    KillRequest, KillResponse, KillStaleRequest,
};
use axum::http::StatusCode;
use axum::{extract::State, Json};

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
    )
    .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Failed to kill stale processes: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn batch_kill_handler(
    State(state): State<AppState>,
    Json(request): Json<BatchKillRequest>,
) -> Result<Json<BatchKillResponse>, StatusCode> {
    let mut results = Vec::with_capacity(request.pids.len());
    let mut succeeded: u32 = 0;
    let mut failed: u32 = 0;

    for &pid in &request.pids {
        match kill_process(pid, request.signal.clone(), &state.config).await {
            Ok(message) => {
                succeeded += 1;
                results.push(BatchKillResult {
                    pid,
                    success: true,
                    message,
                });
            }
            Err(e) => {
                failed += 1;
                results.push(BatchKillResult {
                    pid,
                    success: false,
                    message: e.to_string(),
                });
            }
        }
    }

    Ok(Json(BatchKillResponse {
        total_attempted: results.len() as u32,
        total_succeeded: succeeded,
        total_failed: failed,
        results,
    }))
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
    )
    .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Failed to cleanup processes: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
