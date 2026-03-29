use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

const SERVICE_NAME: &str = "sysmon";
const DEFAULT_PORT: u16 = 8989;

fn home_dir() -> Result<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .context("HOME environment variable not set")
}

fn unit_file_path() -> Result<PathBuf> {
    Ok(home_dir()?
        .join(".config/systemd/user")
        .join(format!("{}.service", SERVICE_NAME)))
}

fn binary_path() -> Result<String> {
    if let Ok(output) = Command::new("which").arg("sysmon").output() {
        if output.status.success() {
            if let Ok(path) = String::from_utf8(output.stdout) {
                let trimmed = path.trim();
                if !trimmed.is_empty() {
                    return Ok(trimmed.to_string());
                }
            }
        }
    }

    std::env::current_exe()
        .context("Cannot determine sysmon binary path")
        .map(|p| p.to_string_lossy().to_string())
}

fn generate_unit_file(bin_path: &str) -> String {
    format!(
        r#"[Unit]
Description=sysmon - lightweight system monitoring dashboard
After=network.target

[Service]
Type=simple
ExecStart={bin} --no-browser --port {port}
Restart=on-failure
RestartSec=5
Environment="RUST_LOG=info"

[Install]
WantedBy=default.target
"#,
        bin = bin_path,
        port = DEFAULT_PORT,
    )
}

fn systemctl(args: &[&str]) -> Result<std::process::Output> {
    let mut cmd_args = vec!["--user"];
    cmd_args.extend_from_slice(args);

    Command::new("systemctl")
        .args(&cmd_args)
        .output()
        .context("Failed to run systemctl")
}

fn systemctl_run(args: &[&str]) -> Result<()> {
    let output = systemctl(args)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "systemctl --user {} failed (exit code {:?}): {}",
            args.join(" "),
            output.status.code(),
            stderr.trim()
        );
    }
    Ok(())
}

fn daemon_reload() -> Result<()> {
    systemctl_run(&["daemon-reload"])
}

fn is_active() -> bool {
    systemctl(&["is-active", SERVICE_NAME])
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
        .unwrap_or(false)
}

pub fn install() -> Result<()> {
    let unit = unit_file_path()?;
    let bin = binary_path()?;

    if unit.exists() {
        bail!(
            "Service already installed at {}\nRun 'sysmon service uninstall' first, then reinstall.",
            unit.display()
        );
    }

    if let Some(parent) = unit.parent() {
        std::fs::create_dir_all(parent).context("Failed to create systemd user directory")?;
    }

    let content = generate_unit_file(&bin);
    std::fs::write(&unit, &content)
        .with_context(|| format!("Failed to write unit file to {}", unit.display()))?;

    daemon_reload()?;

    systemctl_run(&["enable", "--now", SERVICE_NAME])?;

    println!("✓ Service installed and started");
    println!();
    println!("  Unit:   {}", unit.display());
    println!("  Binary: {}", bin);
    println!("  Port:   {}", DEFAULT_PORT);
    println!("  URL:    http://127.0.0.1:{}", DEFAULT_PORT);
    println!();
    println!("  Starts automatically on login.");
    println!("  Auto-restarts on failure.");
    println!();
    println!("  sysmon service status    — check if running");
    println!("  sysmon service logs      — tail the journal");
    println!("  sysmon service stop      — stop the service");
    println!("  sysmon service uninstall — remove completely");

    Ok(())
}

pub fn uninstall() -> Result<()> {
    let unit = unit_file_path()?;

    if !unit.exists() {
        bail!("Service not installed (no unit file at {})", unit.display());
    }

    let _ = systemctl(&["disable", "--now", SERVICE_NAME]);

    std::fs::remove_file(&unit).with_context(|| format!("Failed to remove {}", unit.display()))?;

    daemon_reload()?;

    println!("✓ Service uninstalled");
    println!("  Removed: {}", unit.display());

    Ok(())
}

pub fn start() -> Result<()> {
    let unit = unit_file_path()?;

    if !unit.exists() {
        bail!("Service not installed. Run 'sysmon service install' first.");
    }

    if is_active() {
        println!("Service is already running.");
        println!("  URL: http://127.0.0.1:{}", DEFAULT_PORT);
        return Ok(());
    }

    systemctl_run(&["start", SERVICE_NAME])?;

    println!("✓ Service started");
    println!("  URL: http://127.0.0.1:{}", DEFAULT_PORT);

    Ok(())
}

pub fn stop() -> Result<()> {
    let unit = unit_file_path()?;

    if !unit.exists() {
        bail!("Service not installed. Nothing to stop.");
    }

    if !is_active() {
        println!("Service is not running.");
        return Ok(());
    }

    systemctl_run(&["stop", SERVICE_NAME])?;

    println!("✓ Service stopped");
    println!();
    println!("  Note: service will auto-start on next login.");
    println!("  Use 'sysmon service uninstall' to remove completely.");

    Ok(())
}

pub fn restart() -> Result<()> {
    let unit = unit_file_path()?;

    if !unit.exists() {
        bail!("Service not installed. Run 'sysmon service install' first.");
    }

    systemctl_run(&["restart", SERVICE_NAME])?;

    println!("✓ Service restarted");
    println!("  URL: http://127.0.0.1:{}", DEFAULT_PORT);

    Ok(())
}

pub fn status() -> Result<()> {
    let unit = unit_file_path()?;

    if !unit.exists() {
        println!("● sysmon — not installed");
        println!();
        println!("  Run 'sysmon service install' to set up the service.");
        return Ok(());
    }

    let output = systemctl(&["status", SERVICE_NAME])?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut active_state = "unknown";
    let mut pid: Option<&str> = None;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Active:") {
            if trimmed.contains("active (running)") {
                active_state = "running";
            } else if trimmed.contains("inactive") {
                active_state = "stopped";
            } else if trimmed.contains("failed") {
                active_state = "failed";
            } else if trimmed.contains("activating") {
                active_state = "starting";
            }
        }
        if trimmed.starts_with("Main PID:") {
            pid = trimmed
                .strip_prefix("Main PID:")
                .and_then(|s| s.trim().split_whitespace().next());
        }
    }

    match (active_state, pid) {
        ("running", Some(p)) => println!("● sysmon — running (PID {})", p),
        ("running", None) => println!("● sysmon — running"),
        ("starting", _) => println!("● sysmon — starting"),
        ("failed", _) => println!("● sysmon — failed"),
        ("stopped", _) => println!("● sysmon — stopped"),
        (state, _) => println!("● sysmon — {}", state),
    }

    println!();
    println!("  Port:   {}", DEFAULT_PORT);
    println!("  URL:    http://127.0.0.1:{}", DEFAULT_PORT);
    println!("  Unit:   {}", unit.display());

    Ok(())
}

pub fn logs() -> Result<()> {
    println!("Tailing sysmon journal (Ctrl+C to stop)\n");

    let status = Command::new("journalctl")
        .args(["--user-unit", SERVICE_NAME, "-f", "-n", "100"])
        .status()
        .context("Failed to run journalctl")?;

    if !status.success() {
        bail!("journalctl exited with code {:?}", status.code());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_file_path() {
        let path = unit_file_path().unwrap();
        assert!(path.ends_with(".config/systemd/user/sysmon.service"));
    }

    #[test]
    fn test_generate_unit_file_content() {
        let unit = generate_unit_file("/usr/local/bin/sysmon");
        assert!(unit.contains("[Unit]"));
        assert!(unit.contains("Description=sysmon"));
        assert!(unit.contains("After=network.target"));
        assert!(unit.contains("[Service]"));
        assert!(unit.contains("Type=simple"));
        assert!(unit.contains("/usr/local/bin/sysmon --no-browser --port 8989"));
        assert!(unit.contains("Restart=on-failure"));
        assert!(unit.contains("RestartSec=5"));
        assert!(unit.contains("RUST_LOG=info"));
        assert!(unit.contains("[Install]"));
        assert!(unit.contains("WantedBy=default.target"));
    }

    #[test]
    fn test_default_port() {
        assert_eq!(DEFAULT_PORT, 8989);
    }
}
