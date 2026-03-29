use crate::models::{HistoryPoint, HistoryResponse, SystemMetrics};
use directories::ProjectDirs;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{debug, error, info, warn};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> Result<Self, rusqlite::Error> {
        let db_path = Self::get_db_path();
        info!("Opening metrics database at {}", db_path.display());

        let conn = Connection::open(&db_path)?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA cache_size = 10000;
             PRAGMA temp_store = MEMORY;",
        )?;

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.create_tables()?;
        Ok(db)
    }

    fn get_db_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "sysmon", "sysmon") {
            let data_dir = proj_dirs.data_local_dir();
            if std::fs::create_dir_all(data_dir).is_ok() {
                return data_dir.join("metrics.db");
            }
        }
        // Fallback to current directory
        warn!("Could not determine data directory, using current directory for metrics.db");
        PathBuf::from("metrics.db")
    }

    fn create_tables(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                cpu_percent REAL NOT NULL,
                memory_percent REAL NOT NULL,
                load_1m REAL NOT NULL,
                load_5m REAL NOT NULL,
                load_15m REAL NOT NULL,
                temperature_avg REAL,
                gpu_utilization REAL,
                gpu_memory_percent REAL,
                gpu_temperature REAL
            );
            CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON metrics(timestamp);",
        )?;
        Ok(())
    }

    pub fn insert_metrics(&self, metrics: &SystemMetrics) {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to lock database: {}", e);
                return;
            }
        };

        let temp_avg: Option<f64> = {
            let temps: Vec<f32> = metrics
                .temperature
                .sensors
                .iter()
                .filter_map(|s| s.temperature_celsius)
                .collect();
            if temps.is_empty() {
                None
            } else {
                Some(temps.iter().sum::<f32>() as f64 / temps.len() as f64)
            }
        };

        let (gpu_util, gpu_mem_pct, gpu_temp) = metrics
            .gpu
            .gpus
            .first()
            .map(|g| {
                let mem_pct = match (g.vram_used_bytes, g.vram_total_bytes) {
                    (Some(used), Some(total)) if total > 0 => {
                        Some(used as f64 / total as f64 * 100.0)
                    }
                    _ => None,
                };
                (
                    g.utilization_percent.map(|v| v as f64),
                    mem_pct,
                    g.temperature_celsius.map(|v| v as f64),
                )
            })
            .unwrap_or((None, None, None));

        let timestamp = chrono::Utc::now().timestamp();

        if let Err(e) = conn.execute(
            "INSERT INTO metrics (timestamp, cpu_percent, memory_percent, load_1m, load_5m, load_15m, temperature_avg, gpu_utilization, gpu_memory_percent, gpu_temperature)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                timestamp,
                metrics.cpu.overall_percent as f64,
                metrics.memory.used_percent,
                metrics.load_avg[0],
                metrics.load_avg[1],
                metrics.load_avg[2],
                temp_avg,
                gpu_util,
                gpu_mem_pct,
                gpu_temp,
            ],
        ) {
            error!("Failed to insert metrics: {}", e);
        }
    }

    pub fn query_history(&self, metric: &str, range_seconds: u64) -> HistoryResponse {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => {
                return HistoryResponse {
                    points: vec![],
                    range_seconds,
                    metric: metric.to_string(),
                    point_count: 0,
                };
            }
        };

        let now = chrono::Utc::now().timestamp();
        let start = now - range_seconds as i64;

        let column = match metric {
            "cpu" => "cpu_percent",
            "memory" => "memory_percent",
            "temperature" => "temperature_avg",
            "gpu" => "gpu_utilization",
            "gpu_memory" => "gpu_memory_percent",
            "gpu_temperature" => "gpu_temperature",
            "load_1m" => "load_1m",
            "load_5m" => "load_5m",
            "load_15m" => "load_15m",
            _ => "cpu_percent",
        };

        // Downsample for larger ranges to keep response size manageable
        // Target ~500 points max
        let total_points = range_seconds / 2; // 2s intervals
        let nth = if total_points > 500 {
            (total_points / 500).max(1)
        } else {
            1
        };

        let query = format!(
            "SELECT timestamp, {} FROM metrics WHERE timestamp >= ?1 AND {} IS NOT NULL ORDER BY timestamp ASC",
            column, column
        );

        let mut stmt = match conn.prepare(&query) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to prepare history query: {}", e);
                return HistoryResponse {
                    points: vec![],
                    range_seconds,
                    metric: metric.to_string(),
                    point_count: 0,
                };
            }
        };

        let rows = match stmt.query_map(params![start], |row| {
            Ok(HistoryPoint {
                timestamp: row.get(0)?,
                value: row.get(1)?,
            })
        }) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to query history: {}", e);
                return HistoryResponse {
                    points: vec![],
                    range_seconds,
                    metric: metric.to_string(),
                    point_count: 0,
                };
            }
        };

        let all_points: Vec<HistoryPoint> = rows.filter_map(|r| r.ok()).collect();

        // Downsample
        let points: Vec<HistoryPoint> = if nth > 1 {
            all_points.iter().step_by(nth as usize).cloned().collect()
        } else {
            all_points
        };

        let point_count = points.len();
        HistoryResponse {
            points,
            range_seconds,
            metric: metric.to_string(),
            point_count,
        }
    }

    pub fn prune_old_data(&self, max_age_seconds: i64) {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return,
        };

        let cutoff = chrono::Utc::now().timestamp() - max_age_seconds;
        match conn.execute("DELETE FROM metrics WHERE timestamp < ?1", params![cutoff]) {
            Ok(deleted) => {
                if deleted > 0 {
                    debug!("Pruned {} old metrics rows", deleted);
                }
            }
            Err(e) => error!("Failed to prune old metrics: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use chrono::Utc;

    fn test_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )
        .unwrap();
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.create_tables().unwrap();
        db
    }

    fn sample_metrics() -> SystemMetrics {
        SystemMetrics {
            cpu: CpuMetrics {
                overall_percent: 55.0,
                cores: vec![],
            },
            memory: MemoryMetrics {
                total_bytes: 16_000_000_000,
                used_bytes: 8_000_000_000,
                free_bytes: 4_000_000_000,
                available_bytes: 8_000_000_000,
                swap_total_bytes: 0,
                swap_used_bytes: 0,
                used_percent: 50.0,
            },
            load_avg: [1.5, 2.0, 1.8],
            top_processes: vec![],
            alerts: vec![],
            system: SystemInfo {
                hostname: "test".to_string(),
                os_name: "test".to_string(),
                os_version: "1.0".to_string(),
                kernel_version: "1.0".to_string(),
                uptime_seconds: 1000,
                cpu_count: 4,
                total_memory_bytes: 16_000_000_000,
            },
            network: NetworkMetrics {
                interfaces: vec![],
                total_received_bytes: 0,
                total_transmitted_bytes: 0,
            },
            disk: DiskMetrics { disks: vec![] },
            ports: vec![],
            temperature: TemperatureMetrics {
                sensors: vec![TemperatureSensor {
                    label: "CPU".to_string(),
                    temperature_celsius: Some(55.0),
                    max_celsius: Some(100.0),
                    critical_celsius: None,
                }],
            },
            gpu: GpuMetrics {
                gpus: vec![GpuInfo {
                    name: "Test GPU".to_string(),
                    vendor: "Test".to_string(),
                    vram_total_bytes: Some(8_000_000_000),
                    vram_used_bytes: Some(2_000_000_000),
                    utilization_percent: Some(30.0),
                    temperature_celsius: Some(60.0),
                    power_watts: None,
                }],
            },
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_insert_and_query() {
        let db = test_db();
        let metrics = sample_metrics();

        db.insert_metrics(&metrics);

        let result = db.query_history("cpu", 3600);
        assert_eq!(result.metric, "cpu");
        assert_eq!(result.point_count, 1);
        assert!((result.points[0].value - 55.0).abs() < 0.01);
    }

    #[test]
    fn test_query_empty_db() {
        let db = test_db();
        let result = db.query_history("cpu", 3600);
        assert_eq!(result.point_count, 0);
        assert!(result.points.is_empty());
    }

    #[test]
    fn test_query_memory() {
        let db = test_db();
        db.insert_metrics(&sample_metrics());
        let result = db.query_history("memory", 3600);
        assert_eq!(result.point_count, 1);
        assert!((result.points[0].value - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_query_temperature() {
        let db = test_db();
        db.insert_metrics(&sample_metrics());
        let result = db.query_history("temperature", 3600);
        assert_eq!(result.point_count, 1);
        assert!((result.points[0].value - 55.0).abs() < 0.01);
    }

    #[test]
    fn test_query_gpu() {
        let db = test_db();
        db.insert_metrics(&sample_metrics());
        let result = db.query_history("gpu", 3600);
        assert_eq!(result.point_count, 1);
        assert!((result.points[0].value - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_prune_removes_old_data() {
        let db = test_db();

        // Insert a row with a timestamp in the past
        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO metrics (timestamp, cpu_percent, memory_percent, load_1m, load_5m, load_15m) VALUES (?1, 50.0, 50.0, 1.0, 1.0, 1.0)",
                params![chrono::Utc::now().timestamp() - 7200], // 2 hours ago
            ).unwrap();
        }

        // Prune data older than 1 hour
        db.prune_old_data(3600);

        let result = db.query_history("cpu", 86400);
        assert_eq!(result.point_count, 0);
    }

    #[test]
    fn test_prune_keeps_recent_data() {
        let db = test_db();
        db.insert_metrics(&sample_metrics());

        // Prune only data older than 1 hour
        db.prune_old_data(3600);

        let result = db.query_history("cpu", 3600);
        assert_eq!(result.point_count, 1);
    }

    #[test]
    fn test_insert_with_no_temperature() {
        let db = test_db();
        let mut metrics = sample_metrics();
        metrics.temperature.sensors.clear();
        db.insert_metrics(&metrics);

        let result = db.query_history("temperature", 3600);
        assert_eq!(result.point_count, 0); // NULL values filtered out
    }

    #[test]
    fn test_insert_with_no_gpu() {
        let db = test_db();
        let mut metrics = sample_metrics();
        metrics.gpu.gpus.clear();
        db.insert_metrics(&metrics);

        let result = db.query_history("gpu", 3600);
        assert_eq!(result.point_count, 0); // NULL values filtered out
    }

    #[test]
    fn test_unknown_metric_defaults_to_cpu() {
        let db = test_db();
        db.insert_metrics(&sample_metrics());
        let result = db.query_history("unknown_metric", 3600);
        assert_eq!(result.point_count, 1); // Falls back to cpu_percent
    }
}
