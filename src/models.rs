use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// System-wide metrics snapshot
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub load_avg: [f64; 3],
    pub top_processes: Vec<ProcessInfo>,
    pub alerts: Vec<Alert>,
    pub system: SystemInfo,
    pub network: NetworkMetrics,
    pub disk: DiskMetrics,
    pub ports: Vec<PortInfo>,
    pub temperature: TemperatureMetrics,
    pub gpu: GpuMetrics,
    pub timestamp: DateTime<Utc>,
}

/// Network metrics
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkMetrics {
    pub interfaces: Vec<NetworkInterface>,
    pub total_received_bytes: u64,
    pub total_transmitted_bytes: u64,
}

/// Per-interface network metrics
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkInterface {
    pub name: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
    pub received_packets: u64,
    pub transmitted_packets: u64,
}

/// Disk metrics
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DiskMetrics {
    pub disks: Vec<DiskInfo>,
}

/// Per-disk information
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub used_percent: f64,
    pub is_removable: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortInfo {
    pub port: u16,
    pub protocol: String,
    pub address: String,
    pub pid: u32,
    pub process_name: String,
    pub user: String,
    pub is_external: bool,
    pub service: Option<String>,
}

/// Temperature metrics from system sensors
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TemperatureMetrics {
    pub sensors: Vec<TemperatureSensor>,
}

/// Individual temperature sensor reading
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TemperatureSensor {
    pub label: String,
    pub temperature_celsius: Option<f32>,
    pub max_celsius: Option<f32>,
    pub critical_celsius: Option<f32>,
}

/// GPU metrics
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GpuMetrics {
    pub gpus: Vec<GpuInfo>,
}

/// Individual GPU information
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub vram_total_bytes: Option<u64>,
    pub vram_used_bytes: Option<u64>,
    pub utilization_percent: Option<f32>,
    pub temperature_celsius: Option<f32>,
    pub power_watts: Option<f32>,
}

/// Historical metrics query parameters
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct HistoryQuery {
    pub range: Option<String>,
    pub metric: Option<String>,
}

/// Historical metrics response
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryResponse {
    pub points: Vec<HistoryPoint>,
    pub range_seconds: u64,
    pub metric: String,
    pub point_count: usize,
}

/// Single historical data point
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryPoint {
    pub timestamp: i64,
    pub value: f64,
}

/// CPU metrics
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CpuMetrics {
    pub overall_percent: f32,
    pub cores: Vec<CoreMetrics>,
}

/// Per-core CPU metrics
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CoreMetrics {
    pub id: usize,
    pub usage_percent: f32,
    pub frequency_mhz: u64,
}

/// Memory metrics
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub used_percent: f64,
}

/// Process information
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub memory_percent: f64,
    pub status: String,
    pub started_at: u64,
    pub runtime_seconds: u64,
    pub user: String,
    pub is_stale: bool,
    pub is_zombie: bool,
    pub command: String,
}

/// Stale process detection result
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaleProcess {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub runtime_hours: f64,
    pub duplicate_count: usize,
    pub stale_reason: String,
}

/// Alert definition
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Alert {
    pub id: String,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
    pub since: DateTime<Utc>,
}

/// System information
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub uptime_seconds: u64,
    pub cpu_count: usize,
    pub total_memory_bytes: u64,
}

/// WebSocket message wrapper
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebSocketMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub timestamp: DateTime<Utc>,
    pub sequence: u64,
    pub data: Option<SystemMetrics>,
}

impl WebSocketMessage {
    pub fn metrics_update(sequence: u64, data: SystemMetrics) -> Self {
        Self {
            msg_type: "metrics_update".to_string(),
            timestamp: Utc::now(),
            sequence,
            data: Some(data),
        }
    }

    pub fn ping(sequence: u64) -> Self {
        Self {
            msg_type: "ping".to_string(),
            timestamp: Utc::now(),
            sequence,
            data: None,
        }
    }
}

// Request/Response types for actions

/// Kill process request
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KillRequest {
    pub pid: u32,
    pub signal: Option<String>,
}

/// Kill stale processes request
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KillStaleRequest {
    pub max_age_hours: Option<u64>,
    pub dry_run: Option<bool>,
}

/// Cleanup request
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CleanupRequest {
    pub include_stale: Option<bool>,
    pub stale_max_age_hours: Option<u64>,
    pub dry_run: Option<bool>,
}

/// Kill response
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KillResponse {
    pub success: bool,
    pub message: String,
}

/// Killed process info
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KilledProcess {
    pub pid: u32,
    pub name: String,
    pub process_type: String,
}

/// Cleanup response
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CleanupResponse {
    pub killed_zombies: u32,
    pub killed_stale: u32,
    pub total_killed: u32,
    pub freed_bytes: u64,
    pub processes: Vec<KilledProcess>,
}

/// Health response
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: DateTime<Utc>,
}

/// Container information (optional)
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ContainerInfo {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    pub status: String,
    pub state: String,
    pub ports: Vec<ContainerPort>,
    pub cpu_percent: f64,
    pub memory_bytes: u64,
    pub memory_percent: f64,
}

/// Container port mapping
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ContainerPort {
    pub private_port: u16,
    pub public_port: Option<u16>,
    pub port_type: String,
}

/// Process list query parameters
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ProcessListParams {
    pub sort_by: Option<String>,
    pub limit: Option<usize>,
    pub filter: Option<String>,
}

/// Stale processes response
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaleResponse {
    pub processes: Vec<StaleProcess>,
    pub total_memory_waste_bytes: u64,
    pub stale_count: usize,
    pub zombie_count: usize,
}

/// Batch kill request — send a signal to multiple PIDs at once
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchKillRequest {
    pub pids: Vec<u32>,
    pub signal: Option<String>,
}

/// Result of killing a single process within a batch operation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchKillResult {
    pub pid: u32,
    pub success: bool,
    pub message: String,
}

/// Batch kill response with per-PID results and summary counts
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchKillResponse {
    pub results: Vec<BatchKillResult>,
    pub total_attempted: u32,
    pub total_succeeded: u32,
    pub total_failed: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_core_metrics() -> CoreMetrics {
        CoreMetrics {
            id: 0,
            usage_percent: 45.5,
            frequency_mhz: 3200,
        }
    }

    fn sample_cpu_metrics() -> CpuMetrics {
        CpuMetrics {
            overall_percent: 55.0,
            cores: vec![sample_core_metrics()],
        }
    }

    fn sample_memory_metrics() -> MemoryMetrics {
        MemoryMetrics {
            total_bytes: 16_000_000_000,
            used_bytes: 8_000_000_000,
            free_bytes: 4_000_000_000,
            available_bytes: 8_000_000_000,
            swap_total_bytes: 2_000_000_000,
            swap_used_bytes: 500_000_000,
            used_percent: 50.0,
        }
    }

    fn sample_process_info() -> ProcessInfo {
        ProcessInfo {
            pid: 1234,
            ppid: Some(1),
            name: "test_proc".to_string(),
            cpu_percent: 12.5,
            memory_bytes: 100_000_000,
            memory_percent: 0.625,
            status: "running".to_string(),
            started_at: 1700000000,
            runtime_seconds: 3600,
            user: "testuser".to_string(),
            is_stale: false,
            is_zombie: false,
            command: "/usr/bin/test_proc --flag".to_string(),
        }
    }

    fn sample_alert() -> Alert {
        Alert {
            id: "alert-1".to_string(),
            alert_type: "cpu".to_string(),
            severity: "warning".to_string(),
            message: "CPU usage high".to_string(),
            value: 85.0,
            threshold: 80.0,
            since: Utc::now(),
        }
    }

    fn sample_system_info() -> SystemInfo {
        SystemInfo {
            hostname: "testhost".to_string(),
            os_name: "macOS".to_string(),
            os_version: "14.0".to_string(),
            kernel_version: "23.0.0".to_string(),
            uptime_seconds: 86400,
            cpu_count: 8,
            total_memory_bytes: 16_000_000_000,
        }
    }

    fn sample_network_metrics() -> NetworkMetrics {
        NetworkMetrics {
            interfaces: vec![NetworkInterface {
                name: "en0".to_string(),
                received_bytes: 1_000_000,
                transmitted_bytes: 500_000,
                received_packets: 10000,
                transmitted_packets: 5000,
            }],
            total_received_bytes: 1_000_000,
            total_transmitted_bytes: 500_000,
        }
    }

    fn sample_disk_metrics() -> DiskMetrics {
        DiskMetrics {
            disks: vec![DiskInfo {
                name: "disk0".to_string(),
                mount_point: "/".to_string(),
                file_system: "apfs".to_string(),
                total_bytes: 500_000_000_000,
                available_bytes: 200_000_000_000,
                used_bytes: 300_000_000_000,
                used_percent: 60.0,
                is_removable: false,
            }],
        }
    }

    fn sample_port_info() -> PortInfo {
        PortInfo {
            port: 8989,
            protocol: "tcp".to_string(),
            address: "127.0.0.1".to_string(),
            pid: 1234,
            process_name: "sysmon".to_string(),
            user: "testuser".to_string(),
            is_external: false,
            service: Some("sysmon".to_string()),
        }
    }

    fn sample_temperature_metrics() -> TemperatureMetrics {
        TemperatureMetrics {
            sensors: vec![TemperatureSensor {
                label: "CPU".to_string(),
                temperature_celsius: Some(55.0),
                max_celsius: Some(100.0),
                critical_celsius: Some(105.0),
            }],
        }
    }

    fn sample_gpu_metrics() -> GpuMetrics {
        GpuMetrics {
            gpus: vec![GpuInfo {
                name: "Test GPU".to_string(),
                vendor: "TestVendor".to_string(),
                vram_total_bytes: Some(8_000_000_000),
                vram_used_bytes: Some(2_000_000_000),
                utilization_percent: Some(45.0),
                temperature_celsius: Some(65.0),
                power_watts: Some(150.0),
            }],
        }
    }

    fn sample_system_metrics() -> SystemMetrics {
        SystemMetrics {
            cpu: sample_cpu_metrics(),
            memory: sample_memory_metrics(),
            load_avg: [1.5, 2.0, 1.8],
            top_processes: vec![sample_process_info()],
            alerts: vec![sample_alert()],
            system: sample_system_info(),
            network: sample_network_metrics(),
            disk: sample_disk_metrics(),
            ports: vec![sample_port_info()],
            temperature: sample_temperature_metrics(),
            gpu: sample_gpu_metrics(),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_websocket_message_metrics_update_type() {
        let data = sample_system_metrics();
        let msg = WebSocketMessage::metrics_update(1, data);
        assert_eq!(msg.msg_type, "metrics_update");
    }

    #[test]
    fn test_websocket_message_metrics_update_has_data() {
        let data = sample_system_metrics();
        let msg = WebSocketMessage::metrics_update(42, data);
        assert!(msg.data.is_some());
        assert_eq!(msg.sequence, 42);
    }

    #[test]
    fn test_websocket_message_ping_type() {
        let msg = WebSocketMessage::ping(7);
        assert_eq!(msg.msg_type, "ping");
    }

    #[test]
    fn test_websocket_message_ping_has_no_data() {
        let msg = WebSocketMessage::ping(7);
        assert!(msg.data.is_none());
        assert_eq!(msg.sequence, 7);
    }

    #[test]
    fn test_process_info_serde_roundtrip() {
        let proc = sample_process_info();
        let json = serde_json::to_string(&proc).expect("serialize");
        let deser: ProcessInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.pid, 1234);
        assert_eq!(deser.name, "test_proc");
        assert_eq!(deser.ppid, Some(1));
        assert!(!deser.is_stale);
        assert!(!deser.is_zombie);
    }

    #[test]
    fn test_cpu_metrics_serde_roundtrip() {
        let cpu = sample_cpu_metrics();
        let json = serde_json::to_string(&cpu).expect("serialize");
        let deser: CpuMetrics = serde_json::from_str(&json).expect("deserialize");
        assert!((deser.overall_percent - 55.0).abs() < f32::EPSILON);
        assert_eq!(deser.cores.len(), 1);
        assert_eq!(deser.cores[0].id, 0);
    }

    #[test]
    fn test_memory_metrics_serde_roundtrip() {
        let mem = sample_memory_metrics();
        let json = serde_json::to_string(&mem).expect("serialize");
        let deser: MemoryMetrics = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.total_bytes, 16_000_000_000);
        assert_eq!(deser.used_bytes, 8_000_000_000);
        assert!((deser.used_percent - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_network_interface_serde_roundtrip() {
        let iface = NetworkInterface {
            name: "lo0".to_string(),
            received_bytes: 999,
            transmitted_bytes: 888,
            received_packets: 100,
            transmitted_packets: 50,
        };
        let json = serde_json::to_string(&iface).expect("serialize");
        let deser: NetworkInterface = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.name, "lo0");
        assert_eq!(deser.received_bytes, 999);
    }

    #[test]
    fn test_disk_info_serde_roundtrip() {
        let disk = DiskInfo {
            name: "sda1".to_string(),
            mount_point: "/home".to_string(),
            file_system: "ext4".to_string(),
            total_bytes: 1_000_000_000_000,
            available_bytes: 400_000_000_000,
            used_bytes: 600_000_000_000,
            used_percent: 60.0,
            is_removable: true,
        };
        let json = serde_json::to_string(&disk).expect("serialize");
        let deser: DiskInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.name, "sda1");
        assert!(deser.is_removable);
        assert!((deser.used_percent - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_alert_serde_roundtrip() {
        let alert = sample_alert();
        let json = serde_json::to_string(&alert).expect("serialize");
        let deser: Alert = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.id, "alert-1");
        assert_eq!(deser.alert_type, "cpu");
        assert_eq!(deser.severity, "warning");
        assert!((deser.value - 85.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_system_info_serde_roundtrip() {
        let sys = sample_system_info();
        let json = serde_json::to_string(&sys).expect("serialize");
        let deser: SystemInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.hostname, "testhost");
        assert_eq!(deser.cpu_count, 8);
        assert_eq!(deser.uptime_seconds, 86400);
    }

    #[test]
    fn test_system_metrics_serde_roundtrip() {
        let metrics = sample_system_metrics();
        let json = serde_json::to_string(&metrics).expect("serialize");
        let deser: SystemMetrics = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.top_processes.len(), 1);
        assert_eq!(deser.alerts.len(), 1);
        assert_eq!(deser.load_avg.len(), 3);
    }

    #[test]
    fn test_websocket_message_serde_roundtrip() {
        let msg = WebSocketMessage::ping(5);
        let json = serde_json::to_string(&msg).expect("serialize");
        let deser: WebSocketMessage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.msg_type, "ping");
        assert_eq!(deser.sequence, 5);
        assert!(deser.data.is_none());
    }

    #[test]
    fn test_websocket_message_with_data_serde_roundtrip() {
        let msg = WebSocketMessage::metrics_update(10, sample_system_metrics());
        let json = serde_json::to_string(&msg).expect("serialize");
        let deser: WebSocketMessage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.msg_type, "metrics_update");
        assert!(deser.data.is_some());
    }

    #[test]
    fn test_kill_request_deserialize() {
        let json = r#"{"pid": 42, "signal": "TERM"}"#;
        let req: KillRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.pid, 42);
        assert_eq!(req.signal, Some("TERM".to_string()));
    }

    #[test]
    fn test_kill_request_deserialize_no_signal() {
        let json = r#"{"pid": 100}"#;
        let req: KillRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.pid, 100);
        assert!(req.signal.is_none());
    }

    #[test]
    fn test_cleanup_request_deserialize() {
        let json = r#"{"include_stale": true, "stale_max_age_hours": 48, "dry_run": true}"#;
        let req: CleanupRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.include_stale, Some(true));
        assert_eq!(req.stale_max_age_hours, Some(48));
        assert_eq!(req.dry_run, Some(true));
    }

    #[test]
    fn test_cleanup_request_deserialize_empty() {
        let json = r#"{}"#;
        let req: CleanupRequest = serde_json::from_str(json).expect("deserialize");
        assert!(req.include_stale.is_none());
        assert!(req.stale_max_age_hours.is_none());
        assert!(req.dry_run.is_none());
    }

    #[test]
    fn test_kill_stale_request_serde_roundtrip() {
        let req = KillStaleRequest {
            max_age_hours: Some(12),
            dry_run: Some(false),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let deser: KillStaleRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.max_age_hours, Some(12));
        assert_eq!(deser.dry_run, Some(false));
    }

    #[test]
    fn test_kill_response_serde_roundtrip() {
        let resp = KillResponse {
            success: true,
            message: "done".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let deser: KillResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(deser.success);
        assert_eq!(deser.message, "done");
    }

    #[test]
    fn test_cleanup_response_serde_roundtrip() {
        let resp = CleanupResponse {
            killed_zombies: 2,
            killed_stale: 3,
            total_killed: 5,
            freed_bytes: 1024,
            processes: vec![KilledProcess {
                pid: 99,
                name: "zombie_proc".to_string(),
                process_type: "zombie".to_string(),
            }],
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let deser: CleanupResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.total_killed, 5);
        assert_eq!(deser.processes.len(), 1);
        assert_eq!(deser.processes[0].pid, 99);
    }

    #[test]
    fn test_stale_process_serde_roundtrip() {
        let sp = StaleProcess {
            pid: 555,
            name: "old_worker".to_string(),
            cpu_percent: 0.1,
            memory_bytes: 50_000,
            runtime_hours: 48.5,
            duplicate_count: 4,
            stale_reason: "too old".to_string(),
        };
        let json = serde_json::to_string(&sp).expect("serialize");
        let deser: StaleProcess = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.pid, 555);
        assert_eq!(deser.duplicate_count, 4);
    }

    #[test]
    fn test_container_info_default() {
        let ci = ContainerInfo::default();
        assert!(ci.id.is_empty());
        assert!(ci.names.is_empty());
        assert!(ci.ports.is_empty());
    }

    #[test]
    fn test_container_info_serde_roundtrip() {
        let ci = ContainerInfo {
            id: "abc123".to_string(),
            names: vec!["web".to_string()],
            image: "nginx:latest".to_string(),
            status: "Up 5 hours".to_string(),
            state: "running".to_string(),
            ports: vec![ContainerPort {
                private_port: 80,
                public_port: Some(8080),
                port_type: "tcp".to_string(),
            }],
            cpu_percent: 2.5,
            memory_bytes: 50_000_000,
            memory_percent: 1.2,
        };
        let json = serde_json::to_string(&ci).expect("serialize");
        let deser: ContainerInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.id, "abc123");
        assert_eq!(deser.ports[0].public_port, Some(8080));
    }

    #[test]
    fn test_health_response_serde_roundtrip() {
        let hr = HealthResponse {
            status: "ok".to_string(),
            version: "0.1.0".to_string(),
            uptime_seconds: 1000,
            timestamp: Utc::now(),
        };
        let json = serde_json::to_string(&hr).expect("serialize");
        let deser: HealthResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.status, "ok");
        assert_eq!(deser.version, "0.1.0");
    }

    #[test]
    fn test_process_list_params_default() {
        let params = ProcessListParams::default();
        assert!(params.sort_by.is_none());
        assert!(params.limit.is_none());
        assert!(params.filter.is_none());
    }

    #[test]
    fn test_stale_response_serde_roundtrip() {
        let resp = StaleResponse {
            processes: vec![],
            total_memory_waste_bytes: 0,
            stale_count: 0,
            zombie_count: 0,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let deser: StaleResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.stale_count, 0);
        assert!(deser.processes.is_empty());
    }

    #[test]
    fn test_batch_kill_request_deserialize() {
        let json = r#"{"pids": [1, 2, 3], "signal": "TERM"}"#;
        let req: BatchKillRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.pids, vec![1, 2, 3]);
        assert_eq!(req.signal, Some("TERM".to_string()));
    }

    #[test]
    fn test_batch_kill_request_deserialize_no_signal() {
        let json = r#"{"pids": [42]}"#;
        let req: BatchKillRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.pids, vec![42]);
        assert!(req.signal.is_none());
    }

    #[test]
    fn test_batch_kill_result_serde_roundtrip() {
        let result = BatchKillResult {
            pid: 123,
            success: true,
            message: "ok".to_string(),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deser: BatchKillResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.pid, 123);
        assert!(deser.success);
    }

    #[test]
    fn test_batch_kill_response_serde_roundtrip() {
        let resp = BatchKillResponse {
            results: vec![
                BatchKillResult {
                    pid: 10,
                    success: true,
                    message: "sent".to_string(),
                },
                BatchKillResult {
                    pid: 20,
                    success: false,
                    message: "denied".to_string(),
                },
            ],
            total_attempted: 2,
            total_succeeded: 1,
            total_failed: 1,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let deser: BatchKillResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.total_attempted, 2);
        assert_eq!(deser.total_succeeded, 1);
        assert_eq!(deser.total_failed, 1);
        assert_eq!(deser.results.len(), 2);
    }

    #[test]
    fn test_port_info_serde_roundtrip() {
        let port = sample_port_info();
        let json = serde_json::to_string(&port).expect("serialize");
        let deser: PortInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.port, 8989);
        assert_eq!(deser.protocol, "tcp");
        assert_eq!(deser.process_name, "sysmon");
        assert!(!deser.is_external);
        assert_eq!(deser.service, Some("sysmon".to_string()));
    }

    #[test]
    fn test_port_info_external() {
        let port = PortInfo {
            port: 80,
            protocol: "tcp".to_string(),
            address: "0.0.0.0".to_string(),
            pid: 100,
            process_name: "nginx".to_string(),
            user: "root".to_string(),
            is_external: true,
            service: Some("HTTP".to_string()),
        };
        let json = serde_json::to_string(&port).expect("serialize");
        let deser: PortInfo = serde_json::from_str(&json).expect("deserialize");
        assert!(deser.is_external);
        assert_eq!(deser.service, Some("HTTP".to_string()));
    }

    #[test]
    fn test_batch_kill_response_empty() {
        let resp = BatchKillResponse {
            results: vec![],
            total_attempted: 0,
            total_succeeded: 0,
            total_failed: 0,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let deser: BatchKillResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(deser.results.is_empty());
        assert_eq!(deser.total_attempted, 0);
    }

    #[test]
    fn test_temperature_metrics_serde_roundtrip() {
        let temp = sample_temperature_metrics();
        let json = serde_json::to_string(&temp).expect("serialize");
        let deser: TemperatureMetrics = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.sensors.len(), 1);
        assert_eq!(deser.sensors[0].label, "CPU");
        assert_eq!(deser.sensors[0].temperature_celsius, Some(55.0));
        assert_eq!(deser.sensors[0].critical_celsius, Some(105.0));
    }

    #[test]
    fn test_temperature_sensor_nullable_fields() {
        let sensor = TemperatureSensor {
            label: "Unknown".to_string(),
            temperature_celsius: None,
            max_celsius: None,
            critical_celsius: None,
        };
        let json = serde_json::to_string(&sensor).expect("serialize");
        let deser: TemperatureSensor = serde_json::from_str(&json).expect("deserialize");
        assert!(deser.temperature_celsius.is_none());
        assert!(deser.max_celsius.is_none());
        assert!(deser.critical_celsius.is_none());
    }

    #[test]
    fn test_gpu_metrics_serde_roundtrip() {
        let gpu = sample_gpu_metrics();
        let json = serde_json::to_string(&gpu).expect("serialize");
        let deser: GpuMetrics = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.gpus.len(), 1);
        assert_eq!(deser.gpus[0].name, "Test GPU");
        assert_eq!(deser.gpus[0].vram_total_bytes, Some(8_000_000_000));
        assert_eq!(deser.gpus[0].utilization_percent, Some(45.0));
    }

    #[test]
    fn test_gpu_info_no_metrics() {
        let gpu = GpuInfo {
            name: "Basic GPU".to_string(),
            vendor: "Unknown".to_string(),
            vram_total_bytes: None,
            vram_used_bytes: None,
            utilization_percent: None,
            temperature_celsius: None,
            power_watts: None,
        };
        let json = serde_json::to_string(&gpu).expect("serialize");
        let deser: GpuInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.name, "Basic GPU");
        assert!(deser.vram_total_bytes.is_none());
    }

    #[test]
    fn test_gpu_metrics_empty() {
        let gpu = GpuMetrics { gpus: vec![] };
        let json = serde_json::to_string(&gpu).expect("serialize");
        let deser: GpuMetrics = serde_json::from_str(&json).expect("deserialize");
        assert!(deser.gpus.is_empty());
    }

    #[test]
    fn test_history_query_default() {
        let q = HistoryQuery::default();
        assert!(q.range.is_none());
        assert!(q.metric.is_none());
    }

    #[test]
    fn test_history_response_serde_roundtrip() {
        let resp = HistoryResponse {
            points: vec![
                HistoryPoint {
                    timestamp: 1000,
                    value: 45.5,
                },
                HistoryPoint {
                    timestamp: 1002,
                    value: 46.0,
                },
            ],
            range_seconds: 3600,
            metric: "cpu".to_string(),
            point_count: 2,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let deser: HistoryResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.points.len(), 2);
        assert_eq!(deser.range_seconds, 3600);
        assert_eq!(deser.metric, "cpu");
        assert_eq!(deser.point_count, 2);
    }
}
