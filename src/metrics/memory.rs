use crate::models::MemoryMetrics;
use sysinfo::System;

/// Collect memory metrics from the system
pub fn collect_memory_metrics(system: &System) -> MemoryMetrics {
    let total = system.total_memory();
    let used = system.used_memory();
    let available = system.available_memory();
    let free = system.free_memory();

    let swap_total = system.total_swap();
    let swap_used = system.used_swap();

    // Calculate used percentage
    let used_percent = if total > 0 {
        (used as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    MemoryMetrics {
        total_bytes: total,
        used_bytes: used,
        free_bytes: free,
        available_bytes: available,
        swap_total_bytes: swap_total,
        swap_used_bytes: swap_used,
        used_percent,
    }
}
