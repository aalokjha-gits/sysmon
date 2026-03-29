# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-03-29

### Added

- Real-time system monitoring dashboard with embedded Svelte 5 web UI
- CPU monitoring (per-core usage and frequency)
- Memory monitoring (total, used, free, available, swap)
- Disk usage monitoring with per-disk breakdown
- Network interface monitoring (RX/TX bytes and packets)
- Process table with sorting, filtering, and search
- Process management: kill individual processes with safety guards
- Stale process detection (configurable age, CPU, and duplicate thresholds)
- Zombie process detection and batch cleanup
- Docker and Podman container monitoring
- Alert system with configurable CPU/memory thresholds and consecutive sample tracking
- WebSocket-based real-time data streaming
- TOML-based configuration with cascading file discovery
- CLI with environment variable support
- Automatic browser launch on startup
- JSON output mode for scripting
- macOS universal binary support (arm64 + x86_64)
- Homebrew formula and curl install script
- GitHub Actions CI/CD pipeline
- Responsive dark-theme dashboard (desktop, tablet, mobile)
- Protected process list (prevents killing system-critical processes)
- Command-line argument sanitization (redacts passwords, tokens, API keys)

[0.1.0]: https://github.com/aalokjha-gits/sysmon/releases/tag/v0.1.0
