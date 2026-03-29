use crate::models::{DiskInfo, DiskMetrics};
use sysinfo::Disks;

pub fn collect_disk_metrics() -> DiskMetrics {
    let disks = Disks::new_with_refreshed_list();
    let mut disk_infos = Vec::new();

    for disk in disks.list() {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total.saturating_sub(available);
        let used_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        disk_infos.push(DiskInfo {
            name: disk.name().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            file_system: disk.file_system().to_string_lossy().to_string(),
            total_bytes: total,
            available_bytes: available,
            used_bytes: used,
            used_percent,
            is_removable: disk.is_removable(),
        });
    }

    DiskMetrics { disks: disk_infos }
}
