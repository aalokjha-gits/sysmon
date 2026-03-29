# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0] - 2026-03-30

### Added

- **Time range selector on timeseries charts** — Switch between Live (5m), 1h, 6h, 24h, 7d, and 30d ranges on CPU, Memory, Temperature, and GPU charts, fetching historical data from the SQLite-backed history API
- **Grafana-style core isolation on CPU Cores chart** — Click any core in the legend to isolate it; unselected cores fade to 8% opacity with a "clear" button to reset; multi-select supported
- **Human-friendly temperature panel** — Sensors grouped by type (CPU, GPU, Storage, Memory, Power, Battery, etc.) with status badges (Cool/Normal/Warm/Hot/Critical), overall status in header, grid card layout

### Changed

- **DB retention extended to 30 days** — Historical metrics now kept for 30 days (previously 7 days), auto-pruned
- **Temperature data filtering** — Backend now filters out sensors with impossible readings (≤0°C or ≥150°C) to prevent display artifacts
- **Overview layout improved** — CPU Cores chart moved directly below CPU/Memory row for prominence; Temperature and GPU compact panels use equal-width grid
- **Metric pill formatting** — TEMP pill now shows rounded value (e.g., "33.0°C") instead of raw float

### Fixed

- **Temperature timeseries showing -417°C** — Caused by macOS SMC sensors returning negative values; now filtered at backend and frontend levels
- **Temperature sensor names truncated** — Grid cell minimum width increased from 100px to 130px
- **CPU Cores chart hidden by overflow** — Added flex-shrink: 0 to chart cards so they don't collapse in the scrollable overview

## [0.6.0] - 2026-03-29

### Added

- **Temperature monitoring** — Real-time thermal sensor data via sysinfo Components API (macOS SMC/IOKit, Linux hwmon/thermal_zone)
- **GPU monitoring** — GPU name, vendor, VRAM usage, utilization, temperature, and power draw (macOS via `system_profiler`, Linux via `nvidia-smi` and sysfs DRM)
- **Historical data persistence** — SQLite-backed metrics storage with WAL mode, 7-day retention, and automatic pruning
- **History API** — `GET /api/v1/history?range=1h|6h|24h|7d|30d&metric=cpu|memory|temperature|gpu|load_1m|load_5m|load_15m` for querying historical data with automatic downsampling
- **Temperature timeseries chart** — Amber-colored rolling line chart on Overview dashboard
- **GPU timeseries chart** — Violet-colored rolling line chart on Overview dashboard
- **Temperature compact panel** — Sensor list with color-coded bars (green < 60°C, yellow 60-80°C, red > 80°C) showing max/critical thresholds
- **GPU compact panel** — GPU info display with VRAM usage bar, utilization percentage, temperature, and power metrics
- **TEMP and GPU metric pills** — Average temperature and GPU utilization displayed in the top metric bar
- **New API endpoints** — `GET /api/v1/temperature`, `GET /api/v1/gpu`, `GET /api/v1/history`
- 177 Rust tests (up from 151), 14 Svelte tests

### Dependencies

- Added `rusqlite` 0.32 (with bundled SQLite) for historical data storage
- Added `directories` 6 for platform-appropriate data directory paths

## [0.5.0] - 2026-03-29

### Added

- **Linux support** — full cross-platform support for Linux (x86_64 and aarch64)
- **Port monitoring on Linux** — uses `ss -tlnp` (macOS continues using `lsof`)
- **systemd service management** — `sysmon service install/start/stop/restart/status/logs` using systemd user units on Linux
- **Linux CI** — Rust checks now run on both macOS and Ubuntu
- **Linux release binaries** — x86_64 and aarch64 Linux binaries published in GitHub releases
- **Linux install script** — `install.sh` auto-detects Linux and downloads the correct binary
- **Linux protected processes** — kthreadd, dbus-daemon, NetworkManager, polkitd, systemd-* services added to default protected list
- **Makefile Linux targets** — `make linux`, `make linux-x86_64`, `make linux-aarch64`

### Changed

- Service module refactored from single file to platform module (`service/macos.rs`, `service/linux.rs`)
- Service help text now shows platform-aware description (launchd on macOS, systemd on Linux)

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

[0.5.0]: https://github.com/aalokjha-gits/sysmon/releases/tag/v0.5.0
[0.4.0]: https://github.com/aalokjha-gits/sysmon/releases/tag/v0.4.0
[0.3.0]: https://github.com/aalokjha-gits/sysmon/releases/tag/v0.3.0
[0.2.0]: https://github.com/aalokjha-gits/sysmon/releases/tag/v0.2.0
[0.1.0]: https://github.com/aalokjha-gits/sysmon/releases/tag/v0.1.0
