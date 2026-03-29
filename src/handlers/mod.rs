use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::config::Config;
use crate::metrics::MetricsCollector;
use crate::ws::WebSocketState;

mod actions;
mod containers;
mod health;
mod metrics;
mod ports;
mod processes;

pub use actions::{batch_kill_handler, cleanup_handler, kill_handler, kill_stale_handler};
pub use containers::containers_handler;
pub use health::health_handler;
pub use metrics::{cpu_metrics_handler, memory_metrics_handler, system_metrics_handler};
pub use ports::ports_handler;
pub use processes::{processes_handler, stale_processes_handler};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub ws_state: WebSocketState,
    pub metrics_collector: Arc<MetricsCollector>,
}

/// Create the API router
pub fn create_api_router() -> Router<AppState> {
    Router::new()
        // Health
        .route("/health", get(health_handler))
        // Metrics
        .route("/metrics/system", get(system_metrics_handler))
        .route("/metrics/cpu", get(cpu_metrics_handler))
        .route("/metrics/memory", get(memory_metrics_handler))
        // Containers
        .route("/containers", get(containers_handler))
        .route("/ports", get(ports_handler))
        // Processes
        .route("/processes", get(processes_handler))
        .route("/processes/stale", get(stale_processes_handler))
        // Actions
        .route("/actions/kill", post(kill_handler))
        .route("/actions/kill-batch", post(batch_kill_handler))
        .route("/actions/kill-stale", post(kill_stale_handler))
        .route("/actions/cleanup", post(cleanup_handler))
}
