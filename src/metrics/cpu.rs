use crate::models::{CoreMetrics, CpuMetrics};
use sysinfo::System;

/// Collect CPU metrics from the system
pub fn collect_cpu_metrics(system: &System) -> CpuMetrics {
    let cpus = system.cpus();

    // Collect per-core metrics
    let cores: Vec<CoreMetrics> = cpus
        .iter()
        .enumerate()
        .map(|(id, cpu)| CoreMetrics {
            id,
            usage_percent: cpu.cpu_usage(),
            frequency_mhz: cpu.frequency(),
        })
        .collect();

    // Calculate overall CPU usage as average of cores
    let overall_percent = if !cores.is_empty() {
        cores.iter().map(|c| c.usage_percent).sum::<f32>() / cores.len() as f32
    } else {
        0.0
    };

    CpuMetrics {
        overall_percent,
        cores,
    }
}
