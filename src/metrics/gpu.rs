use crate::models::{GpuInfo, GpuMetrics};
use std::process::Command;

pub fn collect_gpu_metrics() -> GpuMetrics {
    #[cfg(target_os = "macos")]
    let gpus = collect_macos_gpus();

    #[cfg(target_os = "linux")]
    let gpus = collect_linux_gpus();

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let gpus = Vec::new();

    GpuMetrics { gpus }
}

#[cfg(target_os = "macos")]
fn collect_macos_gpus() -> Vec<GpuInfo> {
    let output = match Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    let json_str = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    parse_macos_gpu_json(&json_str)
}

#[cfg(target_os = "macos")]
fn parse_macos_gpu_json(json_str: &str) -> Vec<GpuInfo> {
    let parsed: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut gpus = Vec::new();

    let displays = match parsed.get("SPDisplaysDataType").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    for display in displays {
        let name = display
            .get("sppci_model")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown GPU")
            .to_string();

        let vendor = display
            .get("sppci_vendor")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                display
                    .get("spdisplays_vendor")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
            })
            .to_string();

        let vram_total_bytes = display
            .get("sppci_vram")
            .and_then(|v| v.as_str())
            .and_then(parse_vram_string);

        gpus.push(GpuInfo {
            name,
            vendor,
            vram_total_bytes,
            vram_used_bytes: None,
            utilization_percent: None,
            temperature_celsius: None,
            power_watts: None,
        });
    }

    gpus
}

#[cfg(target_os = "macos")]
fn parse_vram_string(s: &str) -> Option<u64> {
    let s = s.trim().to_lowercase();
    if let Some(num_str) = s.strip_suffix("gb") {
        num_str
            .trim()
            .parse::<u64>()
            .ok()
            .map(|n| n * 1024 * 1024 * 1024)
    } else if let Some(num_str) = s.strip_suffix("mb") {
        num_str.trim().parse::<u64>().ok().map(|n| n * 1024 * 1024)
    } else {
        s.split_whitespace()
            .next()
            .and_then(|n| n.parse::<u64>().ok())
            .map(|n| n * 1024 * 1024) // assume MB
    }
}

#[cfg(target_os = "linux")]
fn collect_linux_gpus() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    if let Some(nvidia_gpus) = try_nvidia_smi() {
        gpus.extend(nvidia_gpus);
    }

    if gpus.is_empty() {
        if let Some(drm_gpus) = try_drm_sysfs() {
            gpus.extend(drm_gpus);
        }
    }

    gpus
}

#[cfg(target_os = "linux")]
fn try_nvidia_smi() -> Option<Vec<GpuInfo>> {
    // query: name, temp, util, mem_used, mem_total, power
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,temperature.gpu,utilization.gpu,memory.used,memory.total,power.draw",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let gpus: Vec<GpuInfo> = stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() < 6 {
                return None;
            }

            Some(GpuInfo {
                name: parts[0].to_string(),
                vendor: "NVIDIA".to_string(),
                vram_total_bytes: parts[4].parse::<u64>().ok().map(|mb| mb * 1024 * 1024),
                vram_used_bytes: parts[3].parse::<u64>().ok().map(|mb| mb * 1024 * 1024),
                utilization_percent: parts[2].parse::<f32>().ok(),
                temperature_celsius: parts[1].parse::<f32>().ok(),
                power_watts: parts[5].parse::<f32>().ok(),
            })
        })
        .collect();

    if gpus.is_empty() {
        None
    } else {
        Some(gpus)
    }
}

#[cfg(target_os = "linux")]
fn try_drm_sysfs() -> Option<Vec<GpuInfo>> {
    use std::fs;
    use std::path::Path;

    let drm_dir = Path::new("/sys/class/drm");
    if !drm_dir.exists() {
        return None;
    }

    let mut gpus = Vec::new();

    for entry in fs::read_dir(drm_dir).ok()? {
        let entry = entry.ok()?;
        let name = entry.file_name().to_string_lossy().to_string();

        // Only look at card0, card1, etc. (not card0-HDMI-A-1 etc.)
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }

        let device_dir = entry.path().join("device");
        if !device_dir.exists() {
            continue;
        }

        let vendor = read_sysfs_file(&device_dir.join("vendor"))
            .map(|v| match v.trim() {
                "0x1002" => "AMD".to_string(),
                "0x8086" => "Intel".to_string(),
                "0x10de" => "NVIDIA".to_string(),
                other => other.to_string(),
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let gpu_name = read_sysfs_file(&device_dir.join("label"))
            .or_else(|| read_sysfs_file(&device_dir.join("product_name")))
            .unwrap_or_else(|| format!("{} GPU ({})", vendor, name));

        let utilization = read_sysfs_file(&device_dir.join("gpu_busy_percent"))
            .and_then(|s| s.trim().parse::<f32>().ok());

        let vram_total = read_sysfs_file(&device_dir.join("mem_info_vram_total"))
            .and_then(|s| s.trim().parse::<u64>().ok());

        let vram_used = read_sysfs_file(&device_dir.join("mem_info_vram_used"))
            .and_then(|s| s.trim().parse::<u64>().ok());

        let temperature = read_hwmon_temp(&device_dir);

        gpus.push(GpuInfo {
            name: gpu_name,
            vendor,
            vram_total_bytes: vram_total,
            vram_used_bytes: vram_used,
            utilization_percent: utilization,
            temperature_celsius: temperature,
            power_watts: None,
        });
    }

    if gpus.is_empty() {
        None
    } else {
        Some(gpus)
    }
}

#[cfg(target_os = "linux")]
fn read_sysfs_file(path: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

#[cfg(target_os = "linux")]
fn read_hwmon_temp(device_dir: &std::path::Path) -> Option<f32> {
    let hwmon_dir = device_dir.join("hwmon");
    if !hwmon_dir.exists() {
        return None;
    }

    for entry in std::fs::read_dir(&hwmon_dir).ok()? {
        let entry = entry.ok()?;
        let temp_path = entry.path().join("temp1_input");
        if let Some(val) = read_sysfs_file(&temp_path) {
            // hwmon reports millidegrees
            if let Ok(millidegrees) = val.trim().parse::<f32>() {
                return Some(millidegrees / 1000.0);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_gpu_returns_valid_struct() {
        let metrics = collect_gpu_metrics();
        for gpu in &metrics.gpus {
            assert!(!gpu.name.is_empty());
            assert!(!gpu.vendor.is_empty());
        }
    }

    #[test]
    fn test_gpu_metrics_serializes() {
        let metrics = collect_gpu_metrics();
        let json = serde_json::to_string(&metrics).expect("serialize");
        assert!(json.contains("gpus"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_vram_string_gb() {
        assert_eq!(parse_vram_string("8 GB"), Some(8 * 1024 * 1024 * 1024));
        assert_eq!(parse_vram_string("16GB"), Some(16 * 1024 * 1024 * 1024));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_vram_string_mb() {
        assert_eq!(parse_vram_string("512 MB"), Some(512 * 1024 * 1024));
        assert_eq!(parse_vram_string("1024MB"), Some(1024 * 1024 * 1024));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_macos_gpu_json_valid() {
        let json = r#"{
            "SPDisplaysDataType": [{
                "sppci_model": "Apple M1 Pro",
                "sppci_vendor": "Apple",
                "sppci_vram": "16 GB"
            }]
        }"#;
        let gpus = parse_macos_gpu_json(json);
        assert_eq!(gpus.len(), 1);
        assert_eq!(gpus[0].name, "Apple M1 Pro");
        assert_eq!(gpus[0].vendor, "Apple");
        assert_eq!(gpus[0].vram_total_bytes, Some(16 * 1024 * 1024 * 1024));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_macos_gpu_json_empty() {
        let gpus = parse_macos_gpu_json("{}");
        assert!(gpus.is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_macos_gpu_json_invalid() {
        let gpus = parse_macos_gpu_json("not json");
        assert!(gpus.is_empty());
    }
}
