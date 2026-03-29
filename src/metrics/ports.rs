use crate::models::PortInfo;
use std::process::Command;

#[allow(dead_code)]
fn parse_address_port(name: &str) -> Option<(String, u16)> {
    if name.starts_with('[') {
        let bracket_end = name.find(']')?;
        let addr = name[..=bracket_end].to_string();
        let port_str = &name[bracket_end + 2..];
        let port = port_str.parse::<u16>().ok()?;
        return Some((addr, port));
    }

    let colon_pos = name.rfind(':')?;
    let addr = name[..colon_pos].to_string();
    let port = name[colon_pos + 1..].parse::<u16>().ok()?;
    Some((addr, port))
}

fn well_known_service(port: u16) -> Option<String> {
    match port {
        22 => Some("SSH".into()),
        53 => Some("DNS".into()),
        80 => Some("HTTP".into()),
        443 => Some("HTTPS".into()),
        3000 => Some("Dev Server".into()),
        3306 => Some("MySQL".into()),
        5432 => Some("PostgreSQL".into()),
        5672 => Some("RabbitMQ".into()),
        6379 => Some("Redis".into()),
        8080 => Some("HTTP Alt".into()),
        8443 => Some("HTTPS Alt".into()),
        8989 => Some("sysmon".into()),
        9090 => Some("Prometheus".into()),
        27017 => Some("MongoDB".into()),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
pub fn collect_listening_ports() -> Vec<PortInfo> {
    let output = match Command::new("lsof")
        .args(["-iTCP", "-iUDP", "-sTCP:LISTEN", "-P", "-n"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ports: Vec<PortInfo> = stdout.lines().skip(1).filter_map(parse_lsof_line).collect();

    ports.sort_by_key(|p| p.port);
    ports.dedup_by(|a, b| a.port == b.port && a.protocol == b.protocol);
    ports
}

#[cfg(target_os = "macos")]
fn parse_lsof_line(line: &str) -> Option<PortInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
        return None;
    }

    let process_name = parts[0].to_string();
    let pid = parts[1].parse::<u32>().ok()?;
    let user = parts[2].to_string();
    let protocol = parts[7].to_lowercase();
    let name_part = parts[8];

    let (address, port) = parse_address_port(name_part)?;
    let is_external = matches!(address.as_str(), "*" | "0.0.0.0" | "::");
    let service = well_known_service(port);

    Some(PortInfo {
        port,
        protocol,
        address,
        pid,
        process_name,
        user,
        is_external,
        service,
    })
}

#[cfg(target_os = "linux")]
pub fn collect_listening_ports() -> Vec<PortInfo> {
    let mut all_ports = Vec::new();

    if let Some(tcp_ports) = run_ss(&["-tlnp"], "tcp") {
        all_ports.extend(tcp_ports);
    }

    if let Some(udp_ports) = run_ss(&["-ulnp"], "udp") {
        all_ports.extend(udp_ports);
    }

    all_ports.sort_by_key(|p| p.port);
    all_ports.dedup_by(|a, b| a.port == b.port && a.protocol == b.protocol);
    all_ports
}

#[cfg(target_os = "linux")]
fn run_ss(args: &[&str], protocol: &str) -> Option<Vec<PortInfo>> {
    let output = Command::new("ss").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(
        stdout
            .lines()
            .skip(1)
            .filter_map(|line| parse_ss_line(line, protocol))
            .collect(),
    )
}

#[cfg(target_os = "linux")]
fn resolve_uid_to_username(pid: u32) -> String {
    let status_path = format!("/proc/{}/status", pid);
    let content = match std::fs::read_to_string(&status_path) {
        Ok(c) => c,
        Err(_) => return "unknown".to_string(),
    };

    let uid_str = content
        .lines()
        .find(|l| l.starts_with("Uid:"))
        .and_then(|l| l.split_whitespace().nth(1));

    let uid = match uid_str {
        Some(s) => s,
        None => return "unknown".to_string(),
    };

    if let Ok(passwd) = std::fs::read_to_string("/etc/passwd") {
        for line in passwd.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() >= 3 && fields[2] == uid {
                return fields[0].to_string();
            }
        }
    }

    format!("uid:{}", uid)
}

/// Parse a line of `ss -tlnp` / `ss -ulnp` output.
/// Format: `State Recv-Q Send-Q Local_Address:Port Peer_Address:Port Process`
/// Process example: `users:(("sshd",pid=1234,fd=5))`
#[cfg(target_os = "linux")]
fn parse_ss_line(line: &str, protocol: &str) -> Option<PortInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }

    let local_addr_port = parts[3];

    let (address, port) = parse_ss_address_port(local_addr_port)?;

    let is_external = matches!(address.as_str(), "*" | "0.0.0.0" | "::" | "[::]");

    let rest = parts[5..].join(" ");
    let (process_name, pid) = parse_ss_process_field(&rest);

    let user = if pid > 0 {
        resolve_uid_to_username(pid)
    } else {
        "unknown".to_string()
    };

    let service = well_known_service(port);

    Some(PortInfo {
        port,
        protocol: protocol.to_string(),
        address,
        pid,
        process_name,
        user,
        is_external,
        service,
    })
}

/// Parse `addr:port` from ss local address field.
/// Handles: `0.0.0.0:80`, `*:80`, `[::]:22`, `[::1]:8080`
#[cfg(target_os = "linux")]
fn parse_ss_address_port(field: &str) -> Option<(String, u16)> {
    if field.starts_with('[') {
        let bracket_end = field.find(']')?;
        let addr = field[..=bracket_end].to_string();
        let port_str = &field[bracket_end + 2..];
        let port = port_str.parse::<u16>().ok()?;
        Some((addr, port))
    } else {
        let colon_pos = field.rfind(':')?;
        let addr = field[..colon_pos].to_string();
        let port = field[colon_pos + 1..].parse::<u16>().ok()?;
        Some((addr, port))
    }
}

/// Extract `(process_name, pid)` from ss process field like `users:(("sshd",pid=1234,fd=5))`.
#[cfg(target_os = "linux")]
fn parse_ss_process_field(field: &str) -> (String, u32) {
    if let Some(start) = field.find("((\"") {
        let after_quote = &field[start + 3..];
        let process_name = after_quote
            .find('"')
            .map(|end| after_quote[..end].to_string())
            .unwrap_or_default();

        let pid = after_quote
            .find("pid=")
            .and_then(|pos| {
                let after_pid = &after_quote[pos + 4..];
                let end = after_pid
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(after_pid.len());
                after_pid[..end].parse::<u32>().ok()
            })
            .unwrap_or(0);

        (process_name, pid)
    } else {
        ("unknown".to_string(), 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address_port_ipv4() {
        let (addr, port) = parse_address_port("127.0.0.1:8989").unwrap();
        assert_eq!(addr, "127.0.0.1");
        assert_eq!(port, 8989);
    }

    #[test]
    fn test_parse_address_port_wildcard() {
        let (addr, port) = parse_address_port("*:80").unwrap();
        assert_eq!(addr, "*");
        assert_eq!(port, 80);
    }

    #[test]
    fn test_parse_address_port_ipv6() {
        let (addr, port) = parse_address_port("[::1]:3000").unwrap();
        assert_eq!(addr, "[::1]");
        assert_eq!(port, 3000);
    }

    #[test]
    fn test_parse_address_port_invalid() {
        assert!(parse_address_port("noport").is_none());
        assert!(parse_address_port("addr:notanum").is_none());
    }

    #[test]
    fn test_well_known_service_ssh() {
        assert_eq!(well_known_service(22), Some("SSH".into()));
    }

    #[test]
    fn test_well_known_service_http() {
        assert_eq!(well_known_service(80), Some("HTTP".into()));
    }

    #[test]
    fn test_well_known_service_postgres() {
        assert_eq!(well_known_service(5432), Some("PostgreSQL".into()));
    }

    #[test]
    fn test_well_known_service_unknown() {
        assert_eq!(well_known_service(12345), None);
    }

    #[test]
    fn test_well_known_service_sysmon() {
        assert_eq!(well_known_service(8989), Some("sysmon".into()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_lsof_line_valid() {
        let line =
            "sysmon    12345 aalok    7u  IPv4 0x1234567890      0t0  TCP 127.0.0.1:8989 (LISTEN)";
        let info = parse_lsof_line(line).unwrap();
        assert_eq!(info.process_name, "sysmon");
        assert_eq!(info.pid, 12345);
        assert_eq!(info.user, "aalok");
        assert_eq!(info.protocol, "tcp");
        assert_eq!(info.address, "127.0.0.1");
        assert_eq!(info.port, 8989);
        assert!(!info.is_external);
        assert_eq!(info.service, Some("sysmon".into()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_lsof_line_external() {
        let line = "nginx     999   root    6u  IPv4 0xabc123          0t0  TCP *:80 (LISTEN)";
        let info = parse_lsof_line(line).unwrap();
        assert_eq!(info.process_name, "nginx");
        assert_eq!(info.port, 80);
        assert!(info.is_external);
        assert_eq!(info.service, Some("HTTP".into()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_lsof_line_too_short() {
        assert!(parse_lsof_line("too short").is_none());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_lsof_line_bad_pid() {
        let line = "cmd    notapid user    7u  IPv4 0x123      0t0  TCP 127.0.0.1:80 (LISTEN)";
        assert!(parse_lsof_line(line).is_none());
    }

    #[test]
    fn test_collect_listening_ports_returns_vec() {
        let ports = collect_listening_ports();
        assert!(ports.iter().all(|p| p.port > 0));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_ss_line_tcp_with_process() {
        let line = "LISTEN 0      128          0.0.0.0:22         0.0.0.0:*    users:((\"sshd\",pid=1234,fd=3))";
        let info = parse_ss_line(line, "tcp").unwrap();
        assert_eq!(info.port, 22);
        assert_eq!(info.protocol, "tcp");
        assert_eq!(info.address, "0.0.0.0");
        assert_eq!(info.process_name, "sshd");
        assert_eq!(info.pid, 1234);
        assert!(info.is_external);
        assert_eq!(info.service, Some("SSH".into()));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_ss_line_tcp_ipv6() {
        let line =
            "LISTEN 0      128             [::]:80            [::]:*    users:((\"nginx\",pid=999,fd=6))";
        let info = parse_ss_line(line, "tcp").unwrap();
        assert_eq!(info.port, 80);
        assert_eq!(info.address, "[::]");
        assert_eq!(info.process_name, "nginx");
        assert_eq!(info.pid, 999);
        assert!(info.is_external);
        assert_eq!(info.service, Some("HTTP".into()));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_ss_line_localhost() {
        let line = "LISTEN 0      511        127.0.0.1:3000       0.0.0.0:*    users:((\"node\",pid=5678,fd=21))";
        let info = parse_ss_line(line, "tcp").unwrap();
        assert_eq!(info.port, 3000);
        assert_eq!(info.address, "127.0.0.1");
        assert_eq!(info.process_name, "node");
        assert_eq!(info.pid, 5678);
        assert!(!info.is_external);
        assert_eq!(info.service, Some("Dev Server".into()));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_ss_line_no_process() {
        let line = "LISTEN 0      128          0.0.0.0:443        0.0.0.0:*";
        let info = parse_ss_line(line, "tcp").unwrap();
        assert_eq!(info.port, 443);
        assert_eq!(info.address, "0.0.0.0");
        assert_eq!(info.process_name, "unknown");
        assert_eq!(info.pid, 0);
        assert!(info.is_external);
        assert_eq!(info.service, Some("HTTPS".into()));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_ss_line_udp() {
        let line = "UNCONN 0      0            0.0.0.0:53         0.0.0.0:*    users:((\"dnsmasq\",pid=400,fd=4))";
        let info = parse_ss_line(line, "udp").unwrap();
        assert_eq!(info.port, 53);
        assert_eq!(info.protocol, "udp");
        assert_eq!(info.process_name, "dnsmasq");
        assert_eq!(info.pid, 400);
        assert!(info.is_external);
        assert_eq!(info.service, Some("DNS".into()));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_ss_line_too_short() {
        assert!(parse_ss_line("LISTEN 0", "tcp").is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_ss_address_port_ipv4() {
        let (addr, port) = parse_ss_address_port("0.0.0.0:80").unwrap();
        assert_eq!(addr, "0.0.0.0");
        assert_eq!(port, 80);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_ss_address_port_ipv6() {
        let (addr, port) = parse_ss_address_port("[::]:22").unwrap();
        assert_eq!(addr, "[::]");
        assert_eq!(port, 22);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_ss_address_port_wildcard() {
        let (addr, port) = parse_ss_address_port("*:53").unwrap();
        assert_eq!(addr, "*");
        assert_eq!(port, 53);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_ss_process_field_valid() {
        let (name, pid) = parse_ss_process_field("users:((\"sshd\",pid=1234,fd=5))");
        assert_eq!(name, "sshd");
        assert_eq!(pid, 1234);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_ss_process_field_empty() {
        let (name, pid) = parse_ss_process_field("");
        assert_eq!(name, "unknown");
        assert_eq!(pid, 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_ss_process_field_multiple_users() {
        let (name, pid) =
            parse_ss_process_field("users:((\"nginx\",pid=100,fd=6),(\"nginx\",pid=101,fd=6))");
        assert_eq!(name, "nginx");
        assert_eq!(pid, 100);
    }
}
