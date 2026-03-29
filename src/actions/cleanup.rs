use crate::config::Config;
use crate::models::{CleanupResponse, KilledProcess};
use std::collections::HashMap;
use std::process::Command;
use tracing::{info, warn};

const MAX_BATCH_KILL: u32 = 500;

/// Cleanup stale and zombie processes
pub async fn cleanup_processes(
    include_stale: bool,
    stale_max_age_hours: Option<u64>,
    dry_run: bool,
    config: &Config,
) -> anyhow::Result<CleanupResponse> {
    let mut response = CleanupResponse {
        killed_zombies: 0,
        killed_stale: 0,
        total_killed: 0,
        freed_bytes: 0,
        processes: Vec::new(),
    };

    // Get zombie processes
    let zombies = get_zombie_processes().await?;

    // Batch limit check
    let total_to_kill = zombies.len() as u32
        + (if include_stale {
            // Estimate stale processes count - actual count will be determined below
            0 // Will check later for stale processes
        } else {
            0
        });
    if total_to_kill > MAX_BATCH_KILL {
        return Err(anyhow::anyhow!(
            "Too many zombie processes to kill at once ({}). Use filters or manual selection.",
            zombies.len()
        ));
    }

    for pid in zombies {
        if dry_run {
            info!("[DRY RUN] Would kill zombie process {}", pid);
            response.killed_zombies += 1;
            response.total_killed += 1;
        } else {
            match kill_process_safe(pid, config).await {
                Ok(name) => {
                    response.killed_zombies += 1;
                    response.total_killed += 1;
                    response.processes.push(KilledProcess {
                        pid,
                        name,
                        process_type: "zombie".to_string(),
                    });
                }
                Err(e) => {
                    warn!("Failed to kill zombie process {}: {}", pid, e);
                }
            }
        }
    }

    // Get stale processes if requested
    if include_stale {
        let stale = get_stale_processes(stale_max_age_hours, config).await?;

        // Check combined batch limit
        let stale_count = stale.len() as u32;
        if response.total_killed + stale_count > MAX_BATCH_KILL {
            return Err(anyhow::anyhow!(
                "Too many processes to kill at once ({}, max: {}). Use filters to narrow down.",
                response.total_killed + stale_count,
                MAX_BATCH_KILL
            ));
        }

        for (pid, name, memory) in stale {
            if dry_run {
                info!("[DRY RUN] Would kill stale process {} ({})", pid, name);
                response.killed_stale += 1;
                response.total_killed += 1;
                response.freed_bytes += memory;
            } else {
                match kill_process_safe(pid, config).await {
                    Ok(killed_name) => {
                        response.killed_stale += 1;
                        response.total_killed += 1;
                        response.freed_bytes += memory;
                        response.processes.push(KilledProcess {
                            pid,
                            name: killed_name,
                            process_type: "stale".to_string(),
                        });
                    }
                    Err(e) => {
                        warn!("Failed to kill stale process {}: {}", pid, e);
                    }
                }
            }
        }
    }

    tracing::warn!(
        target: "sysmon::audit",
        action = "BATCH_CLEANUP",
        killed_zombies = response.killed_zombies,
        killed_stale = response.killed_stale,
        total_killed = response.total_killed,
        freed_bytes = response.freed_bytes,
        "Batch cleanup completed"
    );

    Ok(response)
}

/// Get zombie processes (processes in zombie state)
async fn get_zombie_processes() -> anyhow::Result<Vec<u32>> {
    let output = Command::new("ps")
        .args(["-ax", "-o", "pid,state,comm="])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "ps command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let mut zombies = Vec::new();
    let output_str = String::from_utf8_lossy(&output.stdout);

    for line in output_str.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(pid) = parts[0].parse::<u32>() {
                let state = parts[1];
                // Zombie state is 'Z' on Unix systems
                if state == "Z" || state.contains('Z') {
                    zombies.push(pid);
                }
            }
        }
    }

    Ok(zombies)
}

/// Get stale processes for cleanup
async fn get_stale_processes(
    max_age_hours: Option<u64>,
    _config: &Config,
) -> anyhow::Result<Vec<(u32, String, u64)>> {
    let max_age = max_age_hours.unwrap_or(24) * 3600; // Convert to seconds

    // Get process info with runtime
    let output = Command::new("ps")
        .args(["-ax", "-o", "pid,etime,comm,rss="])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "ps command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let mut process_groups: HashMap<String, Vec<(u32, u64, u64)>> = HashMap::new();
    let output_str = String::from_utf8_lossy(&output.stdout);

    for line in output_str.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            if let Ok(pid) = parts[0].parse::<u32>() {
                let etime = parts[1];
                let comm = parts[2];
                let rss = parts[3].parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes

                // Parse elapsed time (format varies: "1-02:34:56" for days, "02:34:56" for time)
                let runtime = parse_etime(etime);

                if runtime > max_age {
                    process_groups
                        .entry(comm.to_string())
                        .or_default()
                        .push((pid, runtime, rss));
                }
            }
        }
    }

    // Find groups with multiple instances
    let mut stale = Vec::new();
    for (name, processes) in process_groups {
        if processes.len() >= 3 {
            for (pid, _, rss) in processes {
                stale.push((pid, name.clone(), rss));
            }
        }
    }

    Ok(stale)
}

/// Parse elapsed time from ps output
fn parse_etime(etime: &str) -> u64 {
    // Formats: "1-02:34:56" (days-time), "02:34:56" (time), "34:56" (minutes-seconds)
    let mut total_seconds = 0u64;

    if let Some(dash_pos) = etime.find('-') {
        // Has days
        let days: u64 = etime[..dash_pos].parse().unwrap_or(0);
        total_seconds += days * 86400;
        let time_part = &etime[dash_pos + 1..];
        total_seconds += parse_time(time_part);
    } else {
        total_seconds += parse_time(etime);
    }

    total_seconds
}

/// Parse HH:MM:SS or MM:SS time format
fn parse_time(time: &str) -> u64 {
    let parts: Vec<&str> = time.split(':').collect();
    let mut seconds = 0u64;

    match parts.len() {
        3 => {
            // HH:MM:SS
            seconds += parts[0].parse::<u64>().unwrap_or(0) * 3600;
            seconds += parts[1].parse::<u64>().unwrap_or(0) * 60;
            seconds += parts[2].parse::<u64>().unwrap_or(0);
        }
        2 => {
            // MM:SS
            seconds += parts[0].parse::<u64>().unwrap_or(0) * 60;
            seconds += parts[1].parse::<u64>().unwrap_or(0);
        }
        _ => {}
    }

    seconds
}

/// Safely kill a process with validation
async fn kill_process_safe(pid: u32, config: &Config) -> anyhow::Result<String> {
    // Safety checks
    if pid == 1 {
        return Err(anyhow::anyhow!("Cannot kill system init process (PID 1)"));
    }

    let current_pid = std::process::id();
    if pid == current_pid {
        return Err(anyhow::anyhow!("Cannot kill sysmon itself"));
    }

    // Get process name
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Process {} not found", pid));
    }

    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Check protected processes
    let protected = config.protected_processes_set();
    if protected.contains(&name) {
        return Err(anyhow::anyhow!("{} is a protected process", name));
    }

    // Kill the process
    let output = Command::new("kill")
        .arg("-15") // SIGTERM
        .arg(pid.to_string())
        .output()?;

    if output.status.success() {
        info!("Killed process {} ({})", pid, name);
        Ok(name)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("Failed to kill process: {}", stderr))
    }
}
