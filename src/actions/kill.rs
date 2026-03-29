use crate::config::Config;
use std::process::Command;
use tracing::{error, info};

/// Kill a process by PID with optional signal
pub async fn kill_process(pid: u32, signal: Option<String>, config: &Config) -> anyhow::Result<String> {
    // Safety checks
    if pid == 1 {
        return Err(anyhow::anyhow!("Cannot kill system init process (PID 1)"));
    }

    let current_pid = std::process::id();
    if pid == current_pid {
        return Err(anyhow::anyhow!("Cannot kill sysmon itself"));
    }

    // Try to get process info for additional checks
    let process_name = get_process_name(pid).await.unwrap_or_default();
    
    if !process_name.is_empty() {
        let protected = config.protected_processes_set();
        if protected.contains(&process_name) {
            return Err(anyhow::anyhow!(
                "{} is a protected process and cannot be killed",
                process_name
            ));
        }
    }

    // Determine signal
    let sig = signal.unwrap_or_else(|| "TERM".to_string());
    let sig_num = match parse_signal(&sig) {
        Ok(num) => num,
        Err(e) => return Err(anyhow::anyhow!(e)),
    };

    info!(
        "Killing process {} ({}) with signal {}",
        pid, process_name, sig
    );

    // Try to kill using kill command
    let result = Command::new("kill")
        .arg(format!("-{}", sig_num))
        .arg(pid.to_string())
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                tracing::warn!(
                    target: "sysmon::audit",
                    action = "PROCESS_KILL",
                    pid = pid,
                    process_name = %process_name,
                    signal = sig_num,
                    "Process killed"
                );
                Ok(format!(
                    "Successfully sent signal {} to process {} ({})",
                    sig, pid, process_name
                ))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("Failed to kill process: {}", stderr))
            }
        }
        Err(e) => {
            error!("Failed to execute kill command: {}", e);
            Err(anyhow::anyhow!("Failed to execute kill command: {}", e))
        }
    }
}

/// Get process name from PID (best effort)
async fn get_process_name(pid: u32) -> anyhow::Result<String> {
    // On macOS, try ps command
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output()?;

    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout);
        Ok(name.trim().to_string())
    } else {
        Err(anyhow::anyhow!("Process not found"))
    }
}

/// Parse signal name to number
fn parse_signal(sig: &str) -> Result<i32, String> {
    match sig.to_uppercase().as_str() {
        "HUP" | "SIGHUP" | "1" => Ok(1),
        "INT" | "SIGINT" | "2" => Ok(2),
        "QUIT" | "SIGQUIT" | "3" => Ok(3),
        "TERM" | "SIGTERM" | "15" => Ok(15),
        "USR1" | "SIGUSR1" | "10" => Ok(10),
        "USR2" | "SIGUSR2" | "12" => Ok(12),
        "KILL" | "SIGKILL" | "9" => Err("SIGKILL (9) is not allowed for safety. Use SIGTERM (15) instead.".to_string()),
        _ => Err(format!("Unsupported signal: {}. Allowed: TERM, HUP, INT, QUIT, USR1, USR2", sig)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_signal_term() {
        assert_eq!(parse_signal("TERM").unwrap(), 15);
    }

    #[test]
    fn test_parse_signal_sigterm() {
        assert_eq!(parse_signal("SIGTERM").unwrap(), 15);
    }

    #[test]
    fn test_parse_signal_15() {
        assert_eq!(parse_signal("15").unwrap(), 15);
    }

    #[test]
    fn test_parse_signal_hup() {
        assert_eq!(parse_signal("HUP").unwrap(), 1);
    }

    #[test]
    fn test_parse_signal_sighup() {
        assert_eq!(parse_signal("SIGHUP").unwrap(), 1);
    }

    #[test]
    fn test_parse_signal_1() {
        assert_eq!(parse_signal("1").unwrap(), 1);
    }

    #[test]
    fn test_parse_signal_int() {
        assert_eq!(parse_signal("INT").unwrap(), 2);
    }

    #[test]
    fn test_parse_signal_sigint() {
        assert_eq!(parse_signal("SIGINT").unwrap(), 2);
    }

    #[test]
    fn test_parse_signal_2() {
        assert_eq!(parse_signal("2").unwrap(), 2);
    }

    #[test]
    fn test_parse_signal_quit() {
        assert_eq!(parse_signal("QUIT").unwrap(), 3);
    }

    #[test]
    fn test_parse_signal_sigquit() {
        assert_eq!(parse_signal("SIGQUIT").unwrap(), 3);
    }

    #[test]
    fn test_parse_signal_3() {
        assert_eq!(parse_signal("3").unwrap(), 3);
    }

    #[test]
    fn test_parse_signal_usr1() {
        assert_eq!(parse_signal("USR1").unwrap(), 10);
    }

    #[test]
    fn test_parse_signal_sigusr1() {
        assert_eq!(parse_signal("SIGUSR1").unwrap(), 10);
    }

    #[test]
    fn test_parse_signal_10() {
        assert_eq!(parse_signal("10").unwrap(), 10);
    }

    #[test]
    fn test_parse_signal_usr2() {
        assert_eq!(parse_signal("USR2").unwrap(), 12);
    }

    #[test]
    fn test_parse_signal_sigusr2() {
        assert_eq!(parse_signal("SIGUSR2").unwrap(), 12);
    }

    #[test]
    fn test_parse_signal_12() {
        assert_eq!(parse_signal("12").unwrap(), 12);
    }

    #[test]
    fn test_parse_signal_kill_blocked() {
        let result = parse_signal("KILL");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not allowed"));
    }

    #[test]
    fn test_parse_signal_sigkill_blocked() {
        let result = parse_signal("SIGKILL");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not allowed"));
    }

    #[test]
    fn test_parse_signal_9_blocked() {
        let result = parse_signal("9");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not allowed"));
    }

    #[test]
    fn test_parse_signal_unsupported() {
        let result = parse_signal("STOP");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported signal"));
    }

    #[test]
    fn test_parse_signal_garbage() {
        let result = parse_signal("foobar");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_signal_empty() {
        let result = parse_signal("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_signal_case_insensitive() {
        assert_eq!(parse_signal("term").unwrap(), 15);
        assert_eq!(parse_signal("Term").unwrap(), 15);
        assert_eq!(parse_signal("sigterm").unwrap(), 15);
        assert_eq!(parse_signal("SigTerm").unwrap(), 15);
    }

    #[test]
    fn test_parse_signal_kill_case_insensitive_blocked() {
        assert!(parse_signal("kill").is_err());
        assert!(parse_signal("Kill").is_err());
        assert!(parse_signal("sigkill").is_err());
    }
}
