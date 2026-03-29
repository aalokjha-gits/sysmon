use crate::config::Config;
use crate::models::{ProcessInfo, StaleProcess};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{ProcessStatus, System};

/// Sanitize command-line to remove potential secrets
fn sanitize_command(cmd: &str) -> String {
    // Patterns that might contain secrets
    let sensitive_patterns = [
        "--password",
        "--passwd",
        "--secret",
        "--token",
        "--api-key",
        "--apikey",
        "--auth",
        "--credential",
        "-p ", // common short flag for password
        "DATABASE_URL=",
        "API_KEY=",
        "SECRET_KEY=",
        "AWS_SECRET",
        "PRIVATE_KEY",
    ];

    let mut result = cmd.to_string();
    for pattern in &sensitive_patterns {
        if let Some(pos) = result.to_lowercase().find(&pattern.to_lowercase()) {
            // Find the end of the value (next space or end of string)
            let value_start = pos + pattern.len();
            // Skip any = or space
            let value_start = result[value_start..]
                .find(|c: char| !c.is_whitespace() && c != '=')
                .map(|p| value_start + p)
                .unwrap_or(value_start);
            let value_end = result[value_start..]
                .find(char::is_whitespace)
                .map(|p| value_start + p)
                .unwrap_or(result.len());

            if value_start < result.len() {
                result.replace_range(value_start..value_end, "[REDACTED]");
            }
        }
    }
    result
}

/// Collect process information from the system
pub fn collect_processes(system: &System, config: &Config) -> Vec<ProcessInfo> {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let total_memory = system.total_memory();

    // First pass: collect all processes
    let processes: Vec<ProcessInfo> = system
        .processes()
        .iter()
        .map(|(pid, process)| {
            let pid = pid.as_u32();
            let name = process.name().to_string_lossy().to_string();
            let cpu_percent = process.cpu_usage();
            let memory_bytes = process.memory();
            let memory_percent = if total_memory > 0 {
                (memory_bytes as f64 / total_memory as f64) * 100.0
            } else {
                0.0
            };

            let status = match process.status() {
                ProcessStatus::Run => "running".to_string(),
                ProcessStatus::Sleep => "sleeping".to_string(),
                ProcessStatus::Idle => "idle".to_string(),
                ProcessStatus::Zombie => "zombie".to_string(),
                ProcessStatus::Stop => "stopped".to_string(),
                ProcessStatus::Dead => "dead".to_string(),
                _ => format!("{:?}", process.status()).to_lowercase(),
            };
            let is_zombie = process.status() == ProcessStatus::Zombie;
            let started_at = process.start_time();
            let runtime_seconds = current_time.saturating_sub(started_at);

            let ppid = process.parent().map(|p| p.as_u32());
            let user = process
                .effective_user_id()
                .map(|u| u.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let command = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");
            let command = sanitize_command(&command);
            let command = if command.len() > 200 {
                command[..200].to_string()
            } else {
                command
            };

            ProcessInfo {
                pid,
                ppid,
                name,
                cpu_percent,
                memory_bytes,
                memory_percent,
                status,
                started_at,
                runtime_seconds,
                user,
                is_stale: false, // Will be set in second pass
                is_zombie,
                command,
            }
        })
        .collect();

    // Second pass: detect stale processes
    let stale_pids = detect_stale_pids(&processes, config);

    // Update stale flag
    processes
        .into_iter()
        .map(|mut p| {
            if stale_pids.contains(&p.pid) {
                p.is_stale = true;
            }
            p
        })
        .collect()
}

/// Internal stale detection - returns references to stale processes with their group info
fn find_stale_candidates<'a>(
    processes: &'a [ProcessInfo],
    config: &Config,
) -> Vec<(&'a ProcessInfo, usize)> {
    // (process, group_size)
    let protected_set = config.protected_processes_set();
    let stale_config = &config.stale_detection;
    let mut stale_candidates = Vec::new();

    // Group processes by name
    let mut process_groups: HashMap<String, Vec<&'a ProcessInfo>> = HashMap::new();
    for process in processes {
        if protected_set.contains(&process.name) {
            continue;
        }
        process_groups
            .entry(process.name.clone())
            .or_default()
            .push(process);
    }

    let max_age_seconds = stale_config.max_age_hours * 3600;
    let max_cpu = stale_config.max_cpu_percent;

    for group in process_groups.values() {
        if group.len() < stale_config.min_duplicate_count {
            continue;
        }

        let candidates: Vec<&&ProcessInfo> = group
            .iter()
            .filter(|p| p.runtime_seconds > max_age_seconds && (p.cpu_percent as f64) < max_cpu)
            .collect();

        if candidates.len() >= stale_config.min_duplicate_count {
            for process in candidates {
                stale_candidates.push((*process, group.len()));
            }
        }
    }

    stale_candidates
}

/// Detect stale processes and return their PIDs (used internally for marking)
fn detect_stale_pids(processes: &[ProcessInfo], config: &Config) -> Vec<u32> {
    find_stale_candidates(processes, config)
        .into_iter()
        .map(|(p, _)| p.pid)
        .collect()
}

#[allow(dead_code)]
pub fn detect_stale_processes(processes: &[ProcessInfo], config: &Config) -> Vec<StaleProcess> {
    let mut stale_processes: Vec<StaleProcess> = find_stale_candidates(processes, config)
        .into_iter()
        .map(|(process, group_size)| {
            let runtime_hours = process.runtime_seconds as f64 / 3600.0;
            StaleProcess {
                pid: process.pid,
                name: process.name.clone(),
                cpu_percent: process.cpu_percent,
                memory_bytes: process.memory_bytes,
                runtime_hours,
                duplicate_count: group_size,
                stale_reason: format!(
                    "Runtime: {:.1}h, CPU: {:.1}%, Duplicates: {}",
                    runtime_hours, process.cpu_percent, group_size
                ),
            }
        })
        .collect();

    stale_processes.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
    stale_processes
}

#[allow(dead_code)]
pub fn find_process_by_pid(processes: &[ProcessInfo], pid: u32) -> Option<&ProcessInfo> {
    processes.iter().find(|p| p.pid == pid)
}

#[allow(dead_code)]
pub fn can_kill_process(process: &ProcessInfo, config: &Config) -> Result<(), String> {
    let protected_set = config.protected_processes_set();

    // Cannot kill PID 1
    if process.pid == 1 {
        return Err("Cannot kill system init process (PID 1)".to_string());
    }

    // Cannot kill sysmon itself
    if process.name == "sysmon" {
        return Err("Cannot kill sysmon itself".to_string());
    }

    // Cannot kill protected processes
    if protected_set.contains(&process.name) {
        return Err(format!("{} is a protected process", process.name));
    }

    if (process.user == "0" || process.user == "root") && !config.allow_root_kill {
        return Err(format!(
            "Cannot kill root-owned process {} (set allow_root_kill = true in sysmon.toml config to override)",
            process.pid
        ));
    }

    Ok(())
}

#[allow(dead_code)]
pub fn get_zombie_processes(processes: &[ProcessInfo]) -> Vec<&ProcessInfo> {
    processes.iter().filter(|p| p.is_zombie).collect()
}

#[allow(dead_code)]
pub fn get_stale_processes(processes: &[ProcessInfo]) -> Vec<&ProcessInfo> {
    processes.iter().filter(|p| p.is_stale).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn make_process(pid: u32, name: &str, cpu: f32, runtime_secs: u64) -> ProcessInfo {
        ProcessInfo {
            pid,
            ppid: Some(1),
            name: name.to_string(),
            cpu_percent: cpu,
            memory_bytes: 100_000,
            memory_percent: 0.5,
            status: "running".to_string(),
            started_at: 1700000000,
            runtime_seconds: runtime_secs,
            user: "testuser".to_string(),
            is_stale: false,
            is_zombie: false,
            command: format!("/usr/bin/{}", name),
        }
    }

    fn make_zombie(pid: u32, name: &str) -> ProcessInfo {
        ProcessInfo {
            pid,
            ppid: Some(1),
            name: name.to_string(),
            cpu_percent: 0.0,
            memory_bytes: 0,
            memory_percent: 0.0,
            status: "zombie".to_string(),
            started_at: 1700000000,
            runtime_seconds: 86400,
            user: "testuser".to_string(),
            is_stale: false,
            is_zombie: true,
            command: String::new(),
        }
    }

    #[test]
    fn test_sanitize_command_redacts_password() {
        let cmd = "myapp --password mysecret123 --verbose";
        let result = sanitize_command(cmd);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("mysecret123"));
    }

    #[test]
    fn test_sanitize_command_redacts_token() {
        let cmd = "curl --token abc123def456 https://api.example.com";
        let result = sanitize_command(cmd);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("abc123def456"));
    }

    #[test]
    fn test_sanitize_command_redacts_api_key() {
        let cmd = "app --api-key sk-12345 --output json";
        let result = sanitize_command(cmd);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("sk-12345"));
    }

    #[test]
    fn test_sanitize_command_redacts_apikey_no_dash() {
        let cmd = "app --apikey mykey123 --verbose";
        let result = sanitize_command(cmd);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("mykey123"));
    }

    #[test]
    fn test_sanitize_command_redacts_env_api_key() {
        let cmd = "API_KEY=secret_value myapp start";
        let result = sanitize_command(cmd);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("secret_value"));
    }

    #[test]
    fn test_sanitize_command_redacts_database_url() {
        let cmd = "DATABASE_URL=postgres://user:pass@host/db myapp";
        let result = sanitize_command(cmd);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("postgres://user:pass@host/db"));
    }

    #[test]
    fn test_sanitize_command_redacts_aws_secret() {
        let cmd = "AWS_SECRET_ACCESS_KEY=AKIAIOSFODNN7EXAMPLE myapp";
        let result = sanitize_command(cmd);
        assert!(result.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_command_leaves_safe_commands_alone() {
        let cmd = "/usr/bin/ls -la /home";
        let result = sanitize_command(cmd);
        assert_eq!(result, cmd);
    }

    #[test]
    fn test_sanitize_command_redacts_secret_flag() {
        let cmd = "app --secret supersecretvalue";
        let result = sanitize_command(cmd);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("supersecretvalue"));
    }

    #[test]
    fn test_can_kill_process_blocks_pid_1() {
        let config = Config::default();
        let proc = make_process(1, "init", 0.0, 100);
        let result = can_kill_process(&proc, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("PID 1"));
    }

    #[test]
    fn test_can_kill_process_blocks_sysmon() {
        let config = Config::default();
        let proc = make_process(999, "sysmon", 1.0, 100);
        let result = can_kill_process(&proc, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("sysmon"));
    }

    #[test]
    fn test_can_kill_process_blocks_protected_process() {
        let config = Config::default();
        let proc = make_process(100, "kernel_task", 5.0, 1000);
        let result = can_kill_process(&proc, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("protected"));
    }

    #[test]
    fn test_can_kill_process_blocks_launchd() {
        let config = Config::default();
        let proc = make_process(50, "launchd", 0.5, 5000);
        let result = can_kill_process(&proc, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_can_kill_process_blocks_root_owned_by_default() {
        let config = Config::default();
        let mut proc = make_process(500, "some_daemon", 1.0, 100);
        proc.user = "0".to_string();
        let result = can_kill_process(&proc, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("root-owned"));
    }

    #[test]
    fn test_can_kill_process_blocks_root_user_string() {
        let config = Config::default();
        let mut proc = make_process(500, "some_daemon", 1.0, 100);
        proc.user = "root".to_string();
        let result = can_kill_process(&proc, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_can_kill_process_allows_root_when_configured() {
        let mut config = Config::default();
        config.allow_root_kill = true;
        let mut proc = make_process(500, "some_daemon", 1.0, 100);
        proc.user = "0".to_string();
        let result = can_kill_process(&proc, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_can_kill_process_allows_root_user_string_when_configured() {
        let mut config = Config::default();
        config.allow_root_kill = true;
        let mut proc = make_process(500, "some_daemon", 1.0, 100);
        proc.user = "root".to_string();
        let result = can_kill_process(&proc, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_can_kill_process_allows_normal_process() {
        let config = Config::default();
        let proc = make_process(12345, "my_app", 50.0, 3600);
        let result = can_kill_process(&proc, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_process_by_pid_found() {
        let procs = vec![
            make_process(100, "a", 1.0, 10),
            make_process(200, "b", 2.0, 20),
            make_process(300, "c", 3.0, 30),
        ];
        let found = find_process_by_pid(&procs, 200);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "b");
    }

    #[test]
    fn test_find_process_by_pid_not_found() {
        let procs = vec![
            make_process(100, "a", 1.0, 10),
            make_process(200, "b", 2.0, 20),
        ];
        let found = find_process_by_pid(&procs, 999);
        assert!(found.is_none());
    }

    #[test]
    fn test_find_process_by_pid_empty_list() {
        let procs: Vec<ProcessInfo> = vec![];
        let found = find_process_by_pid(&procs, 1);
        assert!(found.is_none());
    }

    #[test]
    fn test_get_zombie_processes_finds_zombies() {
        let procs = vec![
            make_process(100, "alive", 1.0, 10),
            make_zombie(200, "dead1"),
            make_process(300, "alive2", 2.0, 20),
            make_zombie(400, "dead2"),
        ];
        let zombies = get_zombie_processes(&procs);
        assert_eq!(zombies.len(), 2);
        assert_eq!(zombies[0].pid, 200);
        assert_eq!(zombies[1].pid, 400);
    }

    #[test]
    fn test_get_zombie_processes_none_found() {
        let procs = vec![
            make_process(100, "alive", 1.0, 10),
            make_process(200, "alive2", 2.0, 20),
        ];
        let zombies = get_zombie_processes(&procs);
        assert!(zombies.is_empty());
    }

    #[test]
    fn test_get_zombie_processes_empty_list() {
        let procs: Vec<ProcessInfo> = vec![];
        let zombies = get_zombie_processes(&procs);
        assert!(zombies.is_empty());
    }

    #[test]
    fn test_get_stale_processes_finds_stale() {
        let mut procs = vec![
            make_process(100, "fresh", 1.0, 10),
            make_process(200, "old", 0.1, 100000),
        ];
        procs[1].is_stale = true;
        let stale = get_stale_processes(&procs);
        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0].pid, 200);
    }

    #[test]
    fn test_get_stale_processes_none_stale() {
        let procs = vec![
            make_process(100, "a", 1.0, 10),
            make_process(200, "b", 2.0, 20),
        ];
        let stale = get_stale_processes(&procs);
        assert!(stale.is_empty());
    }

    #[test]
    fn test_detect_stale_processes_detects_old_low_cpu_duplicates() {
        let config = Config::default();
        let age = config.stale_detection.max_age_hours * 3600 + 1;

        let procs = vec![
            make_process(100, "worker", 0.1, age),
            make_process(101, "worker", 0.2, age),
            make_process(102, "worker", 0.3, age),
        ];

        let stale = detect_stale_processes(&procs, &config);
        assert_eq!(stale.len(), 3);
        for sp in &stale {
            assert_eq!(sp.name, "worker");
            assert_eq!(sp.duplicate_count, 3);
        }
    }

    #[test]
    fn test_detect_stale_processes_ignores_short_running() {
        let config = Config::default();

        let procs = vec![
            make_process(100, "worker", 0.1, 60),
            make_process(101, "worker", 0.2, 60),
            make_process(102, "worker", 0.3, 60),
        ];

        let stale = detect_stale_processes(&procs, &config);
        assert!(stale.is_empty());
    }

    #[test]
    fn test_detect_stale_processes_ignores_high_cpu() {
        let config = Config::default();
        let age = config.stale_detection.max_age_hours * 3600 + 1;

        let procs = vec![
            make_process(100, "worker", 50.0, age),
            make_process(101, "worker", 60.0, age),
            make_process(102, "worker", 70.0, age),
        ];

        let stale = detect_stale_processes(&procs, &config);
        assert!(stale.is_empty());
    }

    #[test]
    fn test_detect_stale_processes_ignores_too_few_duplicates() {
        let config = Config::default();
        let age = config.stale_detection.max_age_hours * 3600 + 1;

        let procs = vec![
            make_process(100, "worker", 0.1, age),
            make_process(101, "worker", 0.2, age),
        ];

        let stale = detect_stale_processes(&procs, &config);
        assert!(stale.is_empty());
    }

    #[test]
    fn test_detect_stale_processes_ignores_protected() {
        let config = Config::default();
        let age = config.stale_detection.max_age_hours * 3600 + 1;

        let procs = vec![
            make_process(100, "kernel_task", 0.1, age),
            make_process(101, "kernel_task", 0.2, age),
            make_process(102, "kernel_task", 0.3, age),
        ];

        let stale = detect_stale_processes(&procs, &config);
        assert!(stale.is_empty());
    }

    #[test]
    fn test_detect_stale_processes_mixed_qualifying_and_not() {
        let config = Config::default();
        let age = config.stale_detection.max_age_hours * 3600 + 1;

        let procs = vec![
            make_process(100, "worker", 0.1, age),
            make_process(101, "worker", 0.2, age),
            make_process(102, "worker", 0.3, age),
            make_process(200, "fresh_app", 50.0, 10),
            make_process(201, "fresh_app", 45.0, 20),
            make_process(202, "fresh_app", 40.0, 30),
        ];

        let stale = detect_stale_processes(&procs, &config);
        assert_eq!(stale.len(), 3);
        for sp in &stale {
            assert_eq!(sp.name, "worker");
        }
    }

    #[test]
    fn test_detect_stale_processes_returns_sorted_by_memory() {
        let config = Config::default();
        let age = config.stale_detection.max_age_hours * 3600 + 1;

        let mut p1 = make_process(100, "worker", 0.1, age);
        p1.memory_bytes = 1000;
        let mut p2 = make_process(101, "worker", 0.2, age);
        p2.memory_bytes = 5000;
        let mut p3 = make_process(102, "worker", 0.3, age);
        p3.memory_bytes = 3000;

        let procs = vec![p1, p2, p3];
        let stale = detect_stale_processes(&procs, &config);
        assert_eq!(stale.len(), 3);
        assert!(stale[0].memory_bytes >= stale[1].memory_bytes);
        assert!(stale[1].memory_bytes >= stale[2].memory_bytes);
    }

    #[test]
    fn test_detect_stale_processes_partial_old_below_threshold() {
        let config = Config::default();
        let age = config.stale_detection.max_age_hours * 3600 + 1;

        let procs = vec![
            make_process(100, "worker", 0.1, age),
            make_process(101, "worker", 0.2, age),
            make_process(102, "worker", 50.0, age),
        ];

        let stale = detect_stale_processes(&procs, &config);
        assert!(stale.is_empty());
    }
}
