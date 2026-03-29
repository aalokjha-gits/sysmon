use crate::handlers::AppState;
use crate::models::{ProcessInfo, ProcessListParams, StaleResponse};
use crate::metrics::process::detect_stale_processes;
use axum::{
    extract::{Query, State},
    response::Json,
};
use axum::http::StatusCode;

/// Get process list with optional filtering and sorting
pub async fn processes_handler(
    State(state): State<AppState>,
    Query(params): Query<ProcessListParams>,
) -> Result<Json<Vec<ProcessInfo>>, StatusCode> {
    // Use targeted method to get only processes without full metrics collection
    let mut processes = state.metrics_collector.get_processes().await;

    // Apply filter if provided
    if let Some(filter) = params.filter {
        let filter_lower = filter.to_lowercase();
        processes.retain(|p| {
            p.name.to_lowercase().contains(&filter_lower)
                || p.command.to_lowercase().contains(&filter_lower)
                || p.pid.to_string() == filter
        });
    }

    // Apply sorting
    let sort_by = params.sort_by.unwrap_or_else(|| "cpu".to_string());
    match sort_by.as_str() {
        "memory" | "mem" => {
            processes.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
        }
        "name" => {
            processes.sort_by(|a, b| a.name.cmp(&b.name));
        }
        "pid" => {
            processes.sort_by(|a, b| a.pid.cmp(&b.pid));
        }
        "runtime" => {
            processes.sort_by(|a, b| b.runtime_seconds.cmp(&a.runtime_seconds));
        }
        _ => {
            // Default to CPU
            processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap());
        }
    }

    // Apply limit
    if let Some(limit) = params.limit {
        if limit > 0 && limit < processes.len() {
            processes.truncate(limit);
        }
    }

    Ok(Json(processes))
}

/// Get stale processes only
pub async fn stale_processes_handler(
    State(state): State<AppState>,
) -> Result<Json<StaleResponse>, StatusCode> {
    // Use targeted method to get only processes without full metrics collection
    let all_procs = state.metrics_collector.get_processes().await;
    let stale = detect_stale_processes(&all_procs, &state.config);

    let total_memory_waste: u64 = stale.iter().map(|p| p.memory_bytes).sum();
    let zombie_count = all_procs.iter().filter(|p| p.is_zombie).count();

    Ok(Json(StaleResponse {
        stale_count: stale.len(),
        zombie_count,
        total_memory_waste_bytes: total_memory_waste,
        processes: stale,
    }))
}
