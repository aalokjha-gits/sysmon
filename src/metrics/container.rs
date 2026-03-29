use crate::models::{ContainerInfo, ContainerPort};
use std::process::Command;
use tracing::{debug, error, warn};

/// Collect container information from Podman or Docker
/// This is optional and will gracefully return empty list if containers are not available
pub async fn collect_container_info() -> Vec<ContainerInfo> {
    match tokio::task::spawn_blocking(|| {
        // Try Podman first
        match try_podman() {
            Ok(containers) => {
                debug!(
                    "Successfully collected {} containers from Podman",
                    containers.len()
                );
                return containers;
            }
            Err(e) => {
                debug!("Podman not available: {}", e);
            }
        }

        // Fall back to Docker
        match try_docker() {
            Ok(containers) => {
                debug!(
                    "Successfully collected {} containers from Docker",
                    containers.len()
                );
                containers
            }
            Err(e) => {
                debug!("Docker not available: {}", e);
                Vec::new()
            }
        }
    })
    .await
    {
        Ok(containers) => containers,
        Err(e) => {
            error!("Container collection task panicked: {}", e);
            Vec::new()
        }
    }
}

/// Try to get container info from Podman
fn try_podman() -> anyhow::Result<Vec<ContainerInfo>> {
    let output = Command::new("podman")
        .args(["ps", "-a", "--format", "json"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Podman command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    parse_container_output(&output.stdout)
}

/// Try to get container info from Docker
fn try_docker() -> anyhow::Result<Vec<ContainerInfo>> {
    let output = Command::new("docker")
        .args(["ps", "-a", "--format", "json"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Docker command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    parse_container_output(&output.stdout)
}

/// Parse container output from JSON
fn parse_container_output(data: &[u8]) -> anyhow::Result<Vec<ContainerInfo>> {
    let json_str = String::from_utf8_lossy(data);

    // Try to parse as an array first
    if let Ok(containers) = serde_json::from_str::<Vec<PodmanContainer>>(&json_str) {
        return Ok(containers.into_iter().map(|c| c.into()).collect());
    }

    // Try to parse as a single object (newline-delimited JSON)
    let mut containers = Vec::new();
    for line in json_str.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<PodmanContainer>(line) {
            Ok(container) => containers.push(container.into()),
            Err(e) => {
                warn!("Failed to parse container line: {}", e);
            }
        }
    }

    Ok(containers)
}

/// Podman/Docker container JSON structure
#[derive(Debug, serde::Deserialize)]
struct PodmanContainer {
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "Names")]
    names: Vec<String>,
    #[serde(rename = "Image")]
    image: String,
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "State")]
    state: String,
    #[serde(rename = "Ports")]
    #[serde(default)]
    ports: Option<Vec<PodmanPort>>,
}

/// Podman/Docker port mapping
#[derive(Debug, serde::Deserialize)]
struct PodmanPort {
    #[serde(rename = "PrivatePort")]
    private_port: u16,
    #[serde(rename = "PublicPort")]
    #[serde(default)]
    public_port: Option<u16>,
    #[serde(rename = "Type")]
    port_type: String,
}

impl From<PodmanContainer> for ContainerInfo {
    fn from(c: PodmanContainer) -> Self {
        let ports = c
            .ports
            .unwrap_or_default()
            .into_iter()
            .map(|p| ContainerPort {
                private_port: p.private_port,
                public_port: p.public_port,
                port_type: p.port_type,
            })
            .collect();

        ContainerInfo {
            id: c.id[..12].to_string(),
            names: c.names,
            image: c.image,
            status: c.status,
            state: c.state,
            ports,
            cpu_percent: 0.0,
            memory_bytes: 0,
            memory_percent: 0.0,
        }
    }
}

#[allow(dead_code)]
pub async fn get_container_stats() -> Vec<ContainerInfo> {
    match tokio::task::spawn_blocking(|| {
        // Try Podman stats first
        if let Ok(stats) = try_podman_stats() {
            return stats;
        }

        // Fall back to Docker stats
        if let Ok(stats) = try_docker_stats() {
            return stats;
        }

        Vec::new()
    })
    .await
    {
        Ok(stats) => stats,
        Err(e) => {
            error!("Container stats task panicked: {}", e);
            Vec::new()
        }
    }
}

#[allow(dead_code)]
fn try_podman_stats() -> anyhow::Result<Vec<ContainerInfo>> {
    let output = Command::new("podman")
        .args(["stats", "--no-stream", "--format", "json"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Podman stats failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    parse_stats_output(&output.stdout)
}

#[allow(dead_code)]
fn try_docker_stats() -> anyhow::Result<Vec<ContainerInfo>> {
    let output = Command::new("docker")
        .args(["stats", "--no-stream", "--format", "json"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Docker stats failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    parse_stats_output(&output.stdout)
}

#[allow(dead_code)]
fn parse_stats_output(data: &[u8]) -> anyhow::Result<Vec<ContainerInfo>> {
    let json_str = String::from_utf8_lossy(data);
    let mut containers = Vec::new();

    for line in json_str.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<ContainerStats>(line) {
            Ok(stats) => containers.push(stats.into()),
            Err(e) => {
                warn!("Failed to parse stats line: {}", e);
            }
        }
    }

    Ok(containers)
}

/// Container stats structure
#[derive(Debug, serde::Deserialize)]
struct ContainerStats {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "CPUPerc")]
    cpu_perc: String,
    #[serde(rename = "MemUsage")]
    mem_usage: String,
    #[serde(rename = "MemPerc")]
    mem_perc: String,
}

impl From<ContainerStats> for ContainerInfo {
    fn from(s: ContainerStats) -> Self {
        // Parse CPU percentage
        let cpu_percent = s
            .cpu_perc
            .trim_end_matches('%')
            .parse::<f64>()
            .unwrap_or(0.0);

        // Parse memory percentage
        let memory_percent = s
            .mem_perc
            .trim_end_matches('%')
            .parse::<f64>()
            .unwrap_or(0.0);

        // Parse memory usage (extract bytes from format like "10.5MB / 100MB")
        let memory_bytes = parse_memory_usage(&s.mem_usage);

        ContainerInfo {
            id: s.id[..12].to_string(),
            names: vec![s.name],
            image: String::new(),
            status: String::new(),
            state: String::new(),
            ports: Vec::new(),
            cpu_percent,
            memory_bytes,
            memory_percent,
        }
    }
}

/// Parse memory usage string (e.g., "10.5MB / 100MB")
fn parse_memory_usage(usage: &str) -> u64 {
    let parts: Vec<&str> = usage.split('/').collect();
    if parts.is_empty() {
        return 0;
    }

    let used = parts[0].trim();
    parse_byte_size(used)
}

/// Parse byte size string (e.g., "10.5MB", "1.2GiB")
fn parse_byte_size(size: &str) -> u64 {
    let size = size.trim();

    // Extract number and unit
    let (num_str, unit) =
        size.chars()
            .fold((String::new(), String::new()), |(mut num, mut unit), c| {
                if c.is_ascii_digit() || c == '.' {
                    if unit.is_empty() {
                        num.push(c);
                    }
                } else {
                    unit.push(c);
                }
                (num, unit)
            });

    let num: f64 = num_str.parse().unwrap_or(0.0);
    let multiplier = match unit.trim().to_lowercase().as_str() {
        "b" => 1.0,
        "kb" | "kib" => 1024.0,
        "mb" | "mib" => 1024.0 * 1024.0,
        "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        "tb" | "tib" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };

    (num * multiplier) as u64
}
