use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

const LABEL: &str = "com.sysmon.agent";
const DEFAULT_PORT: u16 = 8989;

fn home_dir() -> Result<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .context("HOME environment variable not set")
}

fn plist_path() -> Result<PathBuf> {
    Ok(home_dir()?
        .join("Library/LaunchAgents")
        .join(format!("{}.plist", LABEL)))
}

fn log_path() -> Result<PathBuf> {
    Ok(home_dir()?.join("Library/Logs/sysmon.log"))
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

fn generate_plist(bin_path: &str, log_file: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{bin}</string>
        <string>--no-browser</string>
        <string>--port</string>
        <string>{port}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{log}</string>
    <key>StandardErrorPath</key>
    <string>{log}</string>
    <key>ProcessType</key>
    <string>Background</string>
</dict>
</plist>"#,
        label = LABEL,
        bin = bin_path,
        port = DEFAULT_PORT,
        log = log_file,
    )
}

fn get_service_pid() -> Option<u32> {
    let output = Command::new("launchctl").arg("list").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.contains(LABEL) {
            let pid_str = line.split_whitespace().next()?;
            if pid_str != "-" {
                return pid_str.parse::<u32>().ok();
            }
            return None;
        }
    }
    None
}

fn is_loaded() -> bool {
    Command::new("launchctl")
        .arg("list")
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .any(|l| l.contains(LABEL))
        })
        .unwrap_or(false)
}

pub fn install() -> Result<()> {
    let plist = plist_path()?;
    let log = log_path()?;
    let bin = binary_path()?;

    if plist.exists() {
        bail!(
            "Service already installed at {}\nRun 'sysmon service uninstall' first, then reinstall.",
            plist.display()
        );
    }

    if let Some(parent) = plist.parent() {
        std::fs::create_dir_all(parent).context("Failed to create LaunchAgents directory")?;
    }

    let content = generate_plist(&bin, &log.to_string_lossy());
    std::fs::write(&plist, &content)
        .with_context(|| format!("Failed to write plist to {}", plist.display()))?;

    let status = Command::new("launchctl")
        .args(["load", "-w"])
        .arg(&plist)
        .status()
        .context("Failed to run launchctl")?;

    if !status.success() {
        let _ = std::fs::remove_file(&plist);
        bail!("launchctl load failed (exit code {:?})", status.code());
    }

    println!("✓ Service installed and started");
    println!();
    println!("  Plist:  {}", plist.display());
    println!("  Binary: {}", bin);
    println!("  Log:    {}", log.display());
    println!("  Port:   {}", DEFAULT_PORT);
    println!("  URL:    http://127.0.0.1:{}", DEFAULT_PORT);
    println!();
    println!("  Starts automatically on login.");
    println!("  Auto-restarts on crash.");
    println!();
    println!("  sysmon service status    — check if running");
    println!("  sysmon service logs      — tail the log file");
    println!("  sysmon service stop      — stop the service");
    println!("  sysmon service uninstall — remove completely");

    Ok(())
}

pub fn uninstall() -> Result<()> {
    let plist = plist_path()?;

    if !plist.exists() {
        bail!("Service not installed (no plist at {})", plist.display());
    }

    let _ = Command::new("launchctl")
        .args(["unload", "-w"])
        .arg(&plist)
        .status();

    std::fs::remove_file(&plist)
        .with_context(|| format!("Failed to remove {}", plist.display()))?;

    println!("✓ Service uninstalled");
    println!("  Removed: {}", plist.display());

    let log = log_path()?;
    if log.exists() {
        println!("  Log file kept at: {}", log.display());
        println!("  Delete manually if no longer needed.");
    }

    Ok(())
}

pub fn start() -> Result<()> {
    let plist = plist_path()?;

    if !plist.exists() {
        bail!("Service not installed. Run 'sysmon service install' first.");
    }

    if is_loaded() && get_service_pid().is_some() {
        println!("Service is already running.");
        println!("  URL: http://127.0.0.1:{}", DEFAULT_PORT);
        return Ok(());
    }

    let status = Command::new("launchctl")
        .args(["load", "-w"])
        .arg(&plist)
        .status()
        .context("Failed to run launchctl")?;

    if !status.success() {
        bail!(
            "Failed to start service (launchctl exit code {:?})",
            status.code()
        );
    }

    println!("✓ Service started");
    println!("  URL: http://127.0.0.1:{}", DEFAULT_PORT);

    Ok(())
}

pub fn stop() -> Result<()> {
    let plist = plist_path()?;

    if !plist.exists() {
        bail!("Service not installed. Nothing to stop.");
    }

    if !is_loaded() {
        println!("Service is not running.");
        return Ok(());
    }

    let status = Command::new("launchctl")
        .args(["unload"])
        .arg(&plist)
        .status()
        .context("Failed to run launchctl")?;

    if !status.success() {
        bail!(
            "Failed to stop service (launchctl exit code {:?})",
            status.code()
        );
    }

    println!("✓ Service stopped");
    println!();
    println!("  Note: service will auto-start on next login.");
    println!("  Use 'sysmon service uninstall' to remove completely.");

    Ok(())
}

pub fn restart() -> Result<()> {
    let plist = plist_path()?;

    if !plist.exists() {
        bail!("Service not installed. Run 'sysmon service install' first.");
    }

    let _ = Command::new("launchctl")
        .args(["unload"])
        .arg(&plist)
        .status();

    std::thread::sleep(std::time::Duration::from_millis(500));

    let status = Command::new("launchctl")
        .args(["load", "-w"])
        .arg(&plist)
        .status()
        .context("Failed to run launchctl")?;

    if !status.success() {
        bail!("Failed to restart service");
    }

    println!("✓ Service restarted");
    println!("  URL: http://127.0.0.1:{}", DEFAULT_PORT);

    Ok(())
}

pub fn status() -> Result<()> {
    let plist = plist_path()?;

    if !plist.exists() {
        println!("● sysmon — not installed");
        println!();
        println!("  Run 'sysmon service install' to set up the daemon.");
        return Ok(());
    }

    let loaded = is_loaded();
    let pid = get_service_pid();

    match (loaded, pid) {
        (true, Some(p)) => {
            println!("● sysmon — running (PID {})", p);
        }
        (true, None) => {
            println!("● sysmon — loaded (waiting to start)");
        }
        (false, _) => {
            println!("● sysmon — stopped");
        }
    }

    println!();
    println!("  Port:   {}", DEFAULT_PORT);
    println!("  URL:    http://127.0.0.1:{}", DEFAULT_PORT);
    println!("  Plist:  {}", plist.display());
    println!("  Log:    {}", log_path()?.display());

    let log = log_path()?;
    if log.exists() {
        if let Ok(metadata) = std::fs::metadata(&log) {
            let size = metadata.len();
            if size > 0 {
                println!("  Log sz: {} KB", size / 1024);
            }
        }
    }

    Ok(())
}

pub fn logs() -> Result<()> {
    let log = log_path()?;

    if !log.exists() {
        bail!(
            "No log file at {}\nThe service may not have started yet.",
            log.display()
        );
    }

    println!("Tailing {} (Ctrl+C to stop)\n", log.display());

    let status = Command::new("tail")
        .args(["-f", "-n", "100"])
        .arg(&log)
        .status()
        .context("Failed to tail log file")?;

    if !status.success() {
        bail!("tail exited with code {:?}", status.code());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plist_path() {
        let path = plist_path().unwrap();
        assert!(path.ends_with("Library/LaunchAgents/com.sysmon.agent.plist"));
    }

    #[test]
    fn test_log_path() {
        let path = log_path().unwrap();
        assert!(path.ends_with("Library/Logs/sysmon.log"));
    }

    #[test]
    fn test_generate_plist_content() {
        let plist = generate_plist(
            "/usr/local/bin/sysmon",
            "/Users/test/Library/Logs/sysmon.log",
        );
        assert!(plist.contains("com.sysmon.agent"));
        assert!(plist.contains("/usr/local/bin/sysmon"));
        assert!(plist.contains("--no-browser"));
        assert!(plist.contains("--port"));
        assert!(plist.contains("8989"));
        assert!(plist.contains("RunAtLoad"));
        assert!(plist.contains("KeepAlive"));
        assert!(plist.contains("/Users/test/Library/Logs/sysmon.log"));
    }

    #[test]
    fn test_generate_plist_valid_xml() {
        let plist = generate_plist("/bin/sysmon", "/tmp/sysmon.log");
        assert!(plist.starts_with("<?xml version="));
        assert!(plist.contains("<!DOCTYPE plist"));
        assert!(plist.contains("<plist version=\"1.0\">"));
        assert!(plist.contains("</plist>"));
    }

    #[test]
    fn test_default_port() {
        assert_eq!(DEFAULT_PORT, 8989);
    }

    #[test]
    fn test_label() {
        assert_eq!(LABEL, "com.sysmon.agent");
    }
}
