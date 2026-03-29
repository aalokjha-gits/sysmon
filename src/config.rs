use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Application configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// Server port (0 for ephemeral)
    #[serde(default = "default_port")]
    pub port: u16,

    /// Metrics collection interval in milliseconds
    #[serde(default = "default_interval")]
    pub interval_ms: u64,

    /// Output format: "json" or "text"
    #[serde(default = "default_format")]
    pub format: String,

    /// Disable auto-browser open
    #[serde(default)]
    pub no_browser: bool,

    /// Alert configuration
    #[serde(default)]
    pub alerts: AlertConfig,

    /// Stale process detection configuration
    #[serde(default)]
    pub stale_detection: StaleDetectionConfig,

    /// List of protected process names that cannot be killed
    #[serde(default = "default_protected_processes")]
    pub protected_processes: Vec<String>,

    /// Whether to allow killing root-owned processes
    #[serde(default)]
    pub allow_root_kill: bool,
}

/// Alert configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AlertConfig {
    /// CPU warning threshold (percent)
    #[serde(default = "default_cpu_warning")]
    pub cpu_warning: f64,

    /// CPU critical threshold (percent)
    #[serde(default = "default_cpu_critical")]
    pub cpu_critical: f64,

    /// Memory warning threshold (percent)
    #[serde(default = "default_memory_warning")]
    pub memory_warning: f64,

    /// Memory critical threshold (percent)
    #[serde(default = "default_memory_critical")]
    pub memory_critical: f64,

    /// Number of consecutive samples before alerting
    #[serde(default = "default_alert_samples")]
    pub consecutive_samples: usize,
}

/// Stale process detection configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaleDetectionConfig {
    /// Maximum age in hours to be considered stale
    #[serde(default = "default_stale_age_hours")]
    pub max_age_hours: u64,

    /// Maximum CPU usage percent to be considered stale
    #[serde(default = "default_stale_cpu_percent")]
    pub max_cpu_percent: f64,

    /// Minimum number of duplicate processes to flag
    #[serde(default = "default_stale_duplicate_count")]
    pub min_duplicate_count: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: default_port(),
            interval_ms: default_interval(),
            format: default_format(),
            no_browser: false,
            alerts: AlertConfig::default(),
            stale_detection: StaleDetectionConfig::default(),
            protected_processes: default_protected_processes(),
            allow_root_kill: false,
        }
    }
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            cpu_warning: default_cpu_warning(),
            cpu_critical: default_cpu_critical(),
            memory_warning: default_memory_warning(),
            memory_critical: default_memory_critical(),
            consecutive_samples: default_alert_samples(),
        }
    }
}

impl Default for StaleDetectionConfig {
    fn default() -> Self {
        Self {
            max_age_hours: default_stale_age_hours(),
            max_cpu_percent: default_stale_cpu_percent(),
            min_duplicate_count: default_stale_duplicate_count(),
        }
    }
}

fn default_port() -> u16 {
    0
}

fn default_interval() -> u64 {
    2000
}

fn default_format() -> String {
    "text".to_string()
}

fn default_cpu_warning() -> f64 {
    80.0
}

fn default_cpu_critical() -> f64 {
    95.0
}

fn default_memory_warning() -> f64 {
    90.0
}

fn default_memory_critical() -> f64 {
    95.0
}

fn default_alert_samples() -> usize {
    3
}

fn default_stale_age_hours() -> u64 {
    24
}

fn default_stale_cpu_percent() -> f64 {
    1.0
}

fn default_stale_duplicate_count() -> usize {
    3
}

fn default_protected_processes() -> Vec<String> {
    vec![
        // macOS system critical
        "kernel_task".to_string(),
        "launchd".to_string(),
        "WindowServer".to_string(),
        "loginwindow".to_string(),
        "Dock".to_string(),
        "Finder".to_string(),
        "SystemUIServer".to_string(),
        "cfprefsd".to_string(),
        "distnoted".to_string(),
        "logd".to_string(),
        "notifyd".to_string(),
        "configd".to_string(),
        "securityd".to_string(),
        "trustd".to_string(),
        "kernelmanagerd".to_string(),
        "syspolicyd".to_string(),
        "mds".to_string(),
        "mds_stores".to_string(),
        "opendirectoryd".to_string(),
        "coreservicesd".to_string(),
        // Authentication/Security
        "sshd".to_string(),
        "sudo".to_string(),
        // Linux system critical
        "init".to_string(),
        "systemd".to_string(),
        "journald".to_string(),
        "udevd".to_string(),
        // This process
        "sysmon".to_string(),
    ]
}

impl Config {
    /// Load configuration from file or use defaults
    pub fn load(path: Option<&str>) -> Result<Self> {
        if let Some(path) = path {
            if Path::new(path).exists() {
                let content = fs::read_to_string(path)?;
                let config: Config = toml::from_str(&content)?;
                return Ok(config);
            }
        }

        // Try default config locations
        let default_paths = [
            "sysmon.toml",
            "/etc/sysmon/config.toml",
            &format!(
                "{}/.config/sysmon/config.toml",
                std::env::var("HOME").unwrap_or_default()
            ),
        ];

        for path in &default_paths {
            if Path::new(path).exists() {
                let content = fs::read_to_string(path)?;
                let config: Config = toml::from_str(&content)?;
                return Ok(config);
            }
        }

        // Return default config
        Ok(Config::default())
    }

    #[allow(dead_code)]
    pub fn save(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Get protected processes as a HashSet for efficient lookup
    pub fn protected_processes_set(&self) -> HashSet<String> {
        self.protected_processes.iter().cloned().collect()
    }

    /// Merge CLI arguments into config
    pub fn merge_cli_args(&mut self, port: Option<u16>, interval: Option<u64>, no_browser: bool) {
        if let Some(port) = port {
            self.port = port;
        }
        if let Some(interval) = interval {
            self.interval_ms = interval;
        }
        if no_browser {
            self.no_browser = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default_port() {
        let config = Config::default();
        assert_eq!(config.port, 0);
    }

    #[test]
    fn test_config_default_interval_ms() {
        let config = Config::default();
        assert_eq!(config.interval_ms, 2000);
    }

    #[test]
    fn test_config_default_format() {
        let config = Config::default();
        assert_eq!(config.format, "text");
    }

    #[test]
    fn test_config_default_no_browser() {
        let config = Config::default();
        assert!(!config.no_browser);
    }

    #[test]
    fn test_config_default_allow_root_kill() {
        let config = Config::default();
        assert!(!config.allow_root_kill);
    }

    #[test]
    fn test_alert_config_default_cpu_warning() {
        let alert = AlertConfig::default();
        assert!((alert.cpu_warning - 80.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_alert_config_default_cpu_critical() {
        let alert = AlertConfig::default();
        assert!((alert.cpu_critical - 95.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_alert_config_default_memory_warning() {
        let alert = AlertConfig::default();
        assert!((alert.memory_warning - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_alert_config_default_memory_critical() {
        let alert = AlertConfig::default();
        assert!((alert.memory_critical - 95.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_alert_config_default_consecutive_samples() {
        let alert = AlertConfig::default();
        assert_eq!(alert.consecutive_samples, 3);
    }

    #[test]
    fn test_stale_detection_default_max_age_hours() {
        let stale = StaleDetectionConfig::default();
        assert_eq!(stale.max_age_hours, 24);
    }

    #[test]
    fn test_stale_detection_default_max_cpu_percent() {
        let stale = StaleDetectionConfig::default();
        assert!((stale.max_cpu_percent - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_stale_detection_default_min_duplicate_count() {
        let stale = StaleDetectionConfig::default();
        assert_eq!(stale.min_duplicate_count, 3);
    }

    #[test]
    fn test_config_load_none_returns_defaults() {
        let config = Config::load(None).expect("load(None) should succeed");
        assert_eq!(config.port, 0);
        assert_eq!(config.interval_ms, 2000);
        assert_eq!(config.format, "text");
        assert!(!config.no_browser);
    }

    #[test]
    fn test_config_load_nonexistent_path_returns_defaults() {
        let config =
            Config::load(Some("/tmp/nonexistent_sysmon_test_config.toml")).expect("should succeed");
        assert_eq!(config.port, 0);
        assert_eq!(config.interval_ms, 2000);
    }

    #[test]
    fn test_merge_cli_args_overrides_port() {
        let mut config = Config::default();
        config.merge_cli_args(Some(8080), None, false);
        assert_eq!(config.port, 8080);
        assert_eq!(config.interval_ms, 2000);
        assert!(!config.no_browser);
    }

    #[test]
    fn test_merge_cli_args_overrides_interval() {
        let mut config = Config::default();
        config.merge_cli_args(None, Some(5000), false);
        assert_eq!(config.interval_ms, 5000);
        assert_eq!(config.port, 0);
    }

    #[test]
    fn test_merge_cli_args_overrides_no_browser() {
        let mut config = Config::default();
        config.merge_cli_args(None, None, true);
        assert!(config.no_browser);
    }

    #[test]
    fn test_merge_cli_args_overrides_all() {
        let mut config = Config::default();
        config.merge_cli_args(Some(9090), Some(1000), true);
        assert_eq!(config.port, 9090);
        assert_eq!(config.interval_ms, 1000);
        assert!(config.no_browser);
    }

    #[test]
    fn test_merge_cli_args_no_browser_false_does_not_override() {
        let mut config = Config::default();
        config.no_browser = true;
        config.merge_cli_args(None, None, false);
        assert!(config.no_browser);
    }

    #[test]
    fn test_protected_processes_set_contains_kernel_task() {
        let config = Config::default();
        let set = config.protected_processes_set();
        assert!(set.contains("kernel_task"));
    }

    #[test]
    fn test_protected_processes_set_contains_launchd() {
        let config = Config::default();
        let set = config.protected_processes_set();
        assert!(set.contains("launchd"));
    }

    #[test]
    fn test_protected_processes_set_contains_sysmon() {
        let config = Config::default();
        let set = config.protected_processes_set();
        assert!(set.contains("sysmon"));
    }

    #[test]
    fn test_protected_processes_set_contains_systemd() {
        let config = Config::default();
        let set = config.protected_processes_set();
        assert!(set.contains("systemd"));
    }

    #[test]
    fn test_protected_processes_set_contains_init() {
        let config = Config::default();
        let set = config.protected_processes_set();
        assert!(set.contains("init"));
    }

    #[test]
    fn test_protected_processes_set_does_not_contain_random() {
        let config = Config::default();
        let set = config.protected_processes_set();
        assert!(!set.contains("my_random_process"));
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let config = Config::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: Config = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.port, config.port);
        assert_eq!(deserialized.interval_ms, config.interval_ms);
        assert_eq!(deserialized.format, config.format);
        assert_eq!(deserialized.no_browser, config.no_browser);
        assert_eq!(deserialized.allow_root_kill, config.allow_root_kill);
    }

    #[test]
    fn test_config_toml_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).expect("toml serialize");
        let deserialized: Config = toml::from_str(&toml_str).expect("toml deserialize");
        assert_eq!(deserialized.port, config.port);
        assert_eq!(deserialized.interval_ms, config.interval_ms);
        assert_eq!(deserialized.format, config.format);
    }

    #[test]
    fn test_config_save_and_load() {
        let config = Config::default();
        let path = "/tmp/sysmon_test_config.toml";
        config.save(path).expect("save should succeed");
        let loaded = Config::load(Some(path)).expect("load should succeed");
        assert_eq!(loaded.port, config.port);
        assert_eq!(loaded.interval_ms, config.interval_ms);
        assert_eq!(loaded.format, config.format);
        let _ = std::fs::remove_file(path);
    }
}
