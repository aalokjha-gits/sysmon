use crate::config::Config;
use crate::models::{Alert, SystemInfo, SystemMetrics};
use crate::ws::WebSocketState;
use chrono::Utc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{System, MINIMUM_CPU_UPDATE_INTERVAL};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info};

pub mod container;
mod cpu;
pub mod disk;
mod memory;
pub mod network;
pub mod process;

pub use cpu::collect_cpu_metrics;
pub use disk::collect_disk_metrics;
pub use memory::collect_memory_metrics;
pub use network::collect_network_metrics;
pub use process::collect_processes;

/// Metrics collector that owns the sysinfo System instance
pub struct MetricsCollector {
    system: Arc<RwLock<System>>,
    config: Arc<Config>,
    last_collect_time: Arc<RwLock<Instant>>,
    alert_history: Arc<RwLock<Vec<Alert>>>,
    consecutive_high_cpu: Arc<RwLock<usize>>,
    consecutive_high_memory: Arc<RwLock<usize>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: Arc<Config>) -> Self {
        let mut system = System::new_all();

        // Initial refresh
        system.refresh_all();

        Self {
            system: Arc::new(RwLock::new(system)),
            config,
            last_collect_time: Arc::new(RwLock::new(Instant::now())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            consecutive_high_cpu: Arc::new(RwLock::new(0)),
            consecutive_high_memory: Arc::new(RwLock::new(0)),
        }
    }

    /// Collect all metrics
    pub async fn collect(&self) -> anyhow::Result<SystemMetrics> {
        // Check timing BEFORE acquiring the write lock
        {
            let last_collect = *self.last_collect_time.read().await;
            let elapsed = Instant::now().duration_since(last_collect);
            if elapsed < MINIMUM_CPU_UPDATE_INTERVAL {
                tokio::time::sleep(MINIMUM_CPU_UPDATE_INTERVAL - elapsed).await;
            }
        }

        // NOW acquire the write lock
        let mut system = self.system.write().await;

        // Refresh system info
        system.refresh_all();
        *self.last_collect_time.write().await = Instant::now();

        // Collect individual metrics
        let cpu = collect_cpu_metrics(&system);
        let memory = collect_memory_metrics(&system);
        let load_avg = collect_load_average(&system);
        let processes = collect_processes(&system, &self.config);
        let system_info = collect_system_info(&system);

        // Drop the system lock before doing network/disk (which don't need it)
        drop(system);

        // These don't need the system lock - they create their own instances
        let network = collect_network_metrics();
        let disk = collect_disk_metrics();

        // Get top processes (top 20 by CPU) - use partial sort for efficiency
        let mut top_processes = processes.clone();
        if top_processes.len() > 20 {
            top_processes.select_nth_unstable_by(19, |a, b| {
                b.cpu_percent
                    .partial_cmp(&a.cpu_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            top_processes.truncate(20);
            top_processes.sort_by(|a, b| {
                b.cpu_percent
                    .partial_cmp(&a.cpu_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            top_processes.sort_by(|a, b| {
                b.cpu_percent
                    .partial_cmp(&a.cpu_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        // Evaluate alerts
        let alerts = self.evaluate_alerts(&cpu, &memory, &processes).await;

        Ok(SystemMetrics {
            cpu,
            memory,
            load_avg,
            top_processes,
            alerts,
            system: system_info,
            network,
            disk,
            timestamp: Utc::now(),
        })
    }

    /// Start the background collection task
    pub fn start_collection_task(
        self: Arc<Self>,
        ws_state: WebSocketState,
    ) -> tokio::task::JoinHandle<()> {
        let interval_ms = self.config.interval_ms;

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(interval_ms));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            info!(
                "Starting metrics collection task with {}ms interval",
                interval_ms
            );

            loop {
                ticker.tick().await;

                match self.collect().await {
                    Ok(metrics) => {
                        debug!(
                            "Collected metrics: CPU {:.1}%, Memory {:.1}%",
                            metrics.cpu.overall_percent, metrics.memory.used_percent
                        );
                        ws_state.broadcast_metrics(metrics);
                    }
                    Err(e) => {
                        error!("Failed to collect metrics: {}", e);
                    }
                }
            }
        })
    }

    /// Evaluate alert conditions
    async fn evaluate_alerts(
        &self,
        cpu: &crate::models::CpuMetrics,
        memory: &crate::models::MemoryMetrics,
        processes: &[crate::models::ProcessInfo],
    ) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let config = &self.config.alerts;

        // Check CPU usage
        let cpu_percent = cpu.overall_percent as f64;
        if cpu_percent >= config.cpu_critical {
            let mut consecutive = self.consecutive_high_cpu.write().await;
            *consecutive += 1;
            if *consecutive >= config.consecutive_samples {
                alerts.push(Alert {
                    id: uuid::Uuid::new_v4().to_string(),
                    alert_type: "high_cpu".to_string(),
                    severity: "critical".to_string(),
                    message: format!("CPU usage is at {:.1}%", cpu_percent),
                    value: cpu_percent,
                    threshold: config.cpu_critical,
                    since: Utc::now(),
                });
            }
        } else if cpu_percent >= config.cpu_warning {
            let mut consecutive = self.consecutive_high_cpu.write().await;
            *consecutive += 1;
            if *consecutive >= config.consecutive_samples {
                alerts.push(Alert {
                    id: uuid::Uuid::new_v4().to_string(),
                    alert_type: "high_cpu".to_string(),
                    severity: "warning".to_string(),
                    message: format!("CPU usage is at {:.1}%", cpu_percent),
                    value: cpu_percent,
                    threshold: config.cpu_warning,
                    since: Utc::now(),
                });
            }
        } else {
            *self.consecutive_high_cpu.write().await = 0;
        }

        // Check memory usage
        let mem_percent = memory.used_percent;
        if mem_percent >= config.memory_critical {
            let mut consecutive = self.consecutive_high_memory.write().await;
            *consecutive += 1;
            if *consecutive >= config.consecutive_samples {
                alerts.push(Alert {
                    id: uuid::Uuid::new_v4().to_string(),
                    alert_type: "high_memory".to_string(),
                    severity: "critical".to_string(),
                    message: format!("Memory usage is at {:.1}%", mem_percent),
                    value: mem_percent,
                    threshold: config.memory_critical,
                    since: Utc::now(),
                });
            }
        } else if mem_percent >= config.memory_warning {
            let mut consecutive = self.consecutive_high_memory.write().await;
            *consecutive += 1;
            if *consecutive >= config.consecutive_samples {
                alerts.push(Alert {
                    id: uuid::Uuid::new_v4().to_string(),
                    alert_type: "high_memory".to_string(),
                    severity: "warning".to_string(),
                    message: format!("Memory usage is at {:.1}%", mem_percent),
                    value: mem_percent,
                    threshold: config.memory_warning,
                    since: Utc::now(),
                });
            }
        } else {
            *self.consecutive_high_memory.write().await = 0;
        }

        // Check for stale processes
        let stale_count = processes.iter().filter(|p| p.is_stale).count();
        if stale_count > 0 {
            alerts.push(Alert {
                id: uuid::Uuid::new_v4().to_string(),
                alert_type: "stale_processes".to_string(),
                severity: "info".to_string(),
                message: format!("Found {} stale processes", stale_count),
                value: stale_count as f64,
                threshold: 0.0,
                since: Utc::now(),
            });
        }

        // Check for zombie processes
        let zombie_count = processes.iter().filter(|p| p.is_zombie).count();
        if zombie_count > 5 {
            alerts.push(Alert {
                id: uuid::Uuid::new_v4().to_string(),
                alert_type: "zombie_processes".to_string(),
                severity: if zombie_count > 20 {
                    "critical"
                } else {
                    "warning"
                }
                .to_string(),
                message: format!("Found {} zombie processes", zombie_count),
                value: zombie_count as f64,
                threshold: 5.0,
                since: Utc::now(),
            });
        }

        // Update alert history
        {
            let mut history = self.alert_history.write().await;
            history.extend(alerts.clone());
            // Keep only last 100 alerts
            if history.len() > 100 {
                let split_at = history.len() - 100;
                *history = history.split_off(split_at);
            }
        }

        alerts
    }

    #[allow(dead_code)]
    pub async fn get_system_info(&self) -> SystemInfo {
        let system = self.system.read().await;
        collect_system_info(&system)
    }

    #[allow(dead_code)]
    pub async fn get_system(&self) -> tokio::sync::RwLockReadGuard<'_, System> {
        self.system.read().await
    }

    /// Get processes without collecting all metrics
    pub async fn get_processes(&self) -> Vec<crate::models::ProcessInfo> {
        let system = self.system.read().await;
        process::collect_processes(&system, &self.config)
    }
}

/// Collect system information
fn collect_system_info(system: &System) -> SystemInfo {
    SystemInfo {
        hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
        os_name: System::name().unwrap_or_else(|| "unknown".to_string()),
        os_version: System::os_version().unwrap_or_else(|| "unknown".to_string()),
        kernel_version: System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
        uptime_seconds: System::uptime(),
        cpu_count: system.cpus().len(),
        total_memory_bytes: system.total_memory(),
    }
}

/// Collect load average
fn collect_load_average(_system: &System) -> [f64; 3] {
    let load_avg = System::load_average();
    [load_avg.one, load_avg.five, load_avg.fifteen]
}
