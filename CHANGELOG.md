# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-03-29

### Added

- **Sidebar workspace layout** — 5 dedicated views (Overview, Processes, Network, Containers, Alerts) with keyboard shortcuts 1-5, always-expanded sidebar, mobile bottom nav
- **Port monitoring** — detect all listening TCP/UDP ports via `lsof`, well-known service identification, external exposure warnings, `GET /api/v1/ports` endpoint, real-time port data in WebSocket metrics broadcast
- **CPU & Memory timeseries charts** — rolling 5-minute SVG line charts with gradient fill, gridlines, hover crosshair and tooltip
- **CPU cores multi-line chart** — per-core timeseries with 12-color palette, right-side legend showing "Core N — X%", sort toggle (by core # or usage)
- **Metrics history stores** — accumulate CPU, memory, and per-core data over time from WebSocket
- **Overview dashboard** — system info strip, CPU/Memory timeseries, CPU cores chart, Load + Disk panels, navigation chips to Network and Containers views
- **Dedicated views** — full-width process tree, side-by-side network interfaces + ports tables, container management, alert center with severity sorting

### Changed

- Dashboard layout replaced from fixed two-column split to sidebar + workspace architecture
- Each monitoring domain gets its own full-width view instead of cramped panels
- Load Average panel redesigned to single compact line
- Disk panel redesigned with full mount paths visible and per-disk progress bars
- Overview bottom row uses 2:3 grid ratio (Load smaller, Disk larger)
- Centered metric pills in top bar

## [0.3.0] - 2026-03-29

### Added

- **Daemon mode** — run sysmon as a background service via macOS launchd
- **`sysmon service` CLI** — install, uninstall, start, stop, restart, status, logs subcommands
- **Homebrew services** — `brew services start sysmon` support via formula service block
- **Auto-start on login** — launchd plist with RunAtLoad and KeepAlive (auto-restart on crash)
- **Log file** — daemon output written to `~/Library/Logs/sysmon.log`
- **Fixed daemon port** — port 8989 for consistent access at `http://127.0.0.1:8989`

## [0.2.0] - 2026-03-29

### Added

- **Pause/Freeze toggle** — freeze the process table to inspect, sort, and interact without data refreshing
- **Stable sort** — PID tiebreaker eliminates row jitter between equal-value processes on refresh
- **Multi-select & batch actions** — checkbox column with select-all, floating action bar for batch kill/signal
- **Signal picker** — send SIGTERM, SIGHUP, SIGINT, SIGQUIT, SIGSTOP, SIGCONT, SIGUSR1/2 to processes
- **Batch kill endpoint** — `POST /api/v1/actions/kill-batch` for killing multiple PIDs in one request
- **Pin processes** — pin specific processes to the top of the table for monitoring
- **Per-process CPU sparklines** — inline SVG charts showing CPU usage history (last 30 samples)
- **Process detail drawer** — double-click any process row for full details (command, user, CPU/memory bars, sparkline)
- **Column picker** — show/hide table columns (PID, Name, CPU, Memory, Status, Age, User, Command)
- **Threshold highlighting** — rows glow red/purple when CPU/memory exceed configurable thresholds
- **Hierarchical process tree** — collapsible parent-child tree view with expand/collapse all
- **Process filter buttons** — filter by All, Active, Idle, Stale, Zombie
- **Status propagation** — parent processes show "running" when any descendant is running (Activity Monitor style)

### Fixed

- Process status now propagates from children to parents in tree view
- Resolved "unknown" user for root-owned processes on macOS via ps supplement
- Fixed circular import between metrics and processState stores causing runtime crash
- SIGSTOP and SIGCONT signals now supported in kill operations
- Process data supplemented with macOS `ps` for accurate CPU/memory/status/user across all processes

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

[0.4.0]: https://github.com/aalokjha-gits/sysmon/releases/tag/v0.4.0
[0.3.0]: https://github.com/aalokjha-gits/sysmon/releases/tag/v0.3.0
[0.2.0]: https://github.com/aalokjha-gits/sysmon/releases/tag/v0.2.0
[0.1.0]: https://github.com/aalokjha-gits/sysmon/releases/tag/v0.1.0
