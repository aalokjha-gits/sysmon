use crate::handlers::AppState;
use crate::metrics::collect_listening_ports;
use crate::models::PortInfo;
use axum::{extract::State, response::Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct PortsResponse {
    pub ports: Vec<PortInfo>,
    pub total: usize,
    pub external_count: usize,
}

pub async fn ports_handler(State(_state): State<AppState>) -> Json<PortsResponse> {
    let ports = collect_listening_ports();
    let total = ports.len();
    let external_count = ports.iter().filter(|p| p.is_external).count();

    Json(PortsResponse {
        ports,
        total,
        external_count,
    })
}
