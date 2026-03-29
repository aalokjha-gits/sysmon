use crate::models::PortInfo;
use std::process::Command;

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

    #[test]
    fn test_parse_lsof_line_external() {
        let line = "nginx     999   root    6u  IPv4 0xabc123          0t0  TCP *:80 (LISTEN)";
        let info = parse_lsof_line(line).unwrap();
        assert_eq!(info.process_name, "nginx");
        assert_eq!(info.port, 80);
        assert!(info.is_external);
        assert_eq!(info.service, Some("HTTP".into()));
    }

    #[test]
    fn test_parse_lsof_line_too_short() {
        assert!(parse_lsof_line("too short").is_none());
    }

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
}
