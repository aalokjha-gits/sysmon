use crate::handlers::AppState;
use crate::metrics::container::collect_container_info;
use axum::{
    extract::State,
    response::Json,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct ContainersResponse {
    pub containers: Vec<ContainerView>,
    pub runtime: Option<String>,
}

#[derive(Serialize)]
pub struct ContainerView {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub cpu_percent: f64,
    pub memory_bytes: u64,
    pub ports: Vec<String>,
}

pub async fn containers_handler(
    State(_state): State<AppState>,
) -> Json<ContainersResponse> {
    let containers = collect_container_info().await;

    if containers.is_empty() {
        // Check if runtime is available by trying to run podman/docker
        let runtime = detect_runtime();
        return Json(ContainersResponse {
            containers: vec![],
            runtime,
        });
    }

    let views: Vec<ContainerView> = containers.into_iter().map(|c| {
        let name = c.names.first().cloned().unwrap_or_else(|| c.id.clone());
        let ports: Vec<String> = c.ports.iter().map(|p| {
            if let Some(pub_port) = p.public_port {
                format!("{}:{}/{}", pub_port, p.private_port, p.port_type)
            } else {
                format!("{}/{}", p.private_port, p.port_type)
            }
        }).collect();

        ContainerView {
            id: c.id,
            name,
            image: c.image,
            status: c.state,
            cpu_percent: c.cpu_percent,
            memory_bytes: c.memory_bytes,
            ports,
        }
    }).collect();

    Json(ContainersResponse {
        containers: views,
        runtime: Some(detect_runtime().unwrap_or_else(|| "unknown".to_string())),
    })
}

fn detect_runtime() -> Option<String> {
    if std::process::Command::new("podman").arg("--version").output().is_ok() {
        Some("podman".to_string())
    } else if std::process::Command::new("docker").arg("--version").output().is_ok() {
        Some("docker".to_string())
    } else {
        None
    }
}
