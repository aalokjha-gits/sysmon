# sysmon

<p align="center">
  <img src="assets/logo.svg" alt="sysmon logo" width="120">
</p>

<p align="center">
  <b>Lightweight system monitoring dashboard with a beautiful web UI</b>
</p>

<p align="center">
  <a href="https://github.com/aalokjha-gits/sysmon/releases/latest">
    <img src="https://img.shields.io/github/v/release/aalokjha-gits/sysmon?include_prereleases&sort=semver&style=flat-square" alt="Release">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License">
  </a>
  <a href="https://www.rust-lang.org">
    <img src="https://img.shields.io/badge/Rust-1.75+-orange?style=flat-square&logo=rust" alt="Rust">
  </a>
  <a href="https://svelte.dev">
    <img src="https://img.shields.io/badge/Svelte-5-ff3e00?style=flat-square&logo=svelte" alt="Svelte">
  </a>
</p>

---

## Features

- **Single Binary** — Everything embedded, no external dependencies
- **Sidebar + Workspace UI** — Navigate between Overview, Processes, Network, Containers, and Alerts views with keyboard shortcuts (1-5)
- **Lightweight** — Minimal resource footprint, written in Rust
- **Real-time Metrics** — CPU, memory, disk, network, port, temperature, GPU, and process monitoring via WebSocket with rolling timeseries charts
- **Temperature Monitoring** — Real-time thermal sensor data with color-coded bars and threshold alerts
- **GPU Monitoring** — GPU name, VRAM usage, utilization, temperature, and power draw (NVIDIA, AMD, Apple Silicon)
- **Historical Data** — SQLite-backed metrics persistence with 7-day retention and configurable history API
- **Process Tree** — Hierarchical parent-child process view with expand/collapse, search, filters (Active/Idle/Stale/Zombie), multi-select batch kill, signal picker, per-process CPU sparklines, and detail drawer
- **Port Monitoring** — Detect all listening TCP/UDP ports with process ownership, service identification, and external exposure warnings
- **Container Monitoring** — Docker and Podman container stats
- **Alert System** — Configurable CPU/memory thresholds with consecutive sample tracking
- **Daemon Mode** — Run as a background service via launchd (macOS) or systemd (Linux) with `sysmon service` commands
- **CPU Cores Chart** — Multi-line per-core timeseries with color-coded legend and sort by core # or usage
- **Responsive Design** — Sidebar collapses to bottom nav on mobile
- **Configurable** — TOML-based configuration with sensible defaults

## Quick Install

### Option 1: One-liner (macOS / Linux)

```bash
curl -sSL https://raw.githubusercontent.com/aalokjha-gits/sysmon/main/install.sh | sh
```

### Option 2: Homebrew (macOS)

```bash
brew tap aalokjha-gits/sysmon
brew install sysmon
```

### Option 3: Build from source

```bash
git clone https://github.com/aalokjha-gits/sysmon.git
cd sysmon
make install
```

## Usage

### Start with defaults

```bash
sysmon
```

Picks an ephemeral port and opens your browser automatically.

### Custom port

```bash
sysmon --port 8080
```

### Without browser

```bash
sysmon --no-browser
```

### JSON output mode

```bash
sysmon --json
```

### Daemon mode

```bash
sysmon service install    # Install service (launchd on macOS, systemd on Linux)
sysmon service start      # Start the daemon
sysmon service stop       # Stop the daemon
sysmon service status     # Check daemon status
sysmon service logs       # View daemon logs
sysmon service uninstall  # Remove service
```

The daemon runs on port 8989 with `--no-browser`. Access at `http://127.0.0.1:8989`.

On macOS, uses launchd (`~/Library/LaunchAgents/`). On Linux, uses systemd user units (`~/.config/systemd/user/`).

### Homebrew services

```bash
brew services start sysmon   # Start as background service
brew services stop sysmon    # Stop service
```

### All options

```
USAGE:
    sysmon [OPTIONS] [COMMAND]

OPTIONS:
    -p, --port <PORT>        Server port (0 for ephemeral) [env: SYSMON_PORT]
    -i, --interval <MS>      Metrics collection interval in ms [env: SYSMON_INTERVAL]
    -c, --config <PATH>      Configuration file path [env: SYSMON_CONFIG]
        --no-browser         Disable auto-browser open [env: SYSMON_NO_BROWSER]
        --json               Output format as JSON
    -h, --help               Print help information
    -V, --version            Print version information

COMMANDS:
    service                  Manage sysmon as a system service (install, start, stop, etc.)
```

## Architecture

```
┌─────────────────────────────────────────┐
│           Single Binary                 │
│  ┌──────────────┐  ┌────────────────┐   │
│  │   Rust       │  │  Svelte UI     │   │
│  │   Backend    │  │  (embedded)    │   │
│  │              │  │                │   │
│  │ • Axum       │  │ • Real-time    │   │
│  │ • WebSocket  │  │   dashboard    │   │
│  │ • sysinfo    │  │ • Process mgmt │   │
│  └──────────────┘  └────────────────┘   │
│           ↕ WebSocket                   │
└─────────────────────────────────────────┘
                   ↕
         ┌─────────────────┐
         │  Web Browser    │
         └─────────────────┘
```

### Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust + Axum + Tokio |
| Frontend | Svelte 5 + TypeScript |
| Styling | Custom CSS variables (dark theme) |
| Visualizations | Custom CSS bars + SVG gauges |
| Icons | Inline SVG |
| Embedding | rust-embed (compressed) |

## Configuration

sysmon looks for configuration in this order:

1. Path passed via `--config`
2. `sysmon.toml` in the current directory
3. `/etc/sysmon/config.toml`
4. `~/.config/sysmon/config.toml`
5. Built-in defaults

Create a config file:

```toml
# Server port (0 = ephemeral)
port = 0

# Metrics collection interval in milliseconds
interval_ms = 2000

# Output format: "json" or "text"
format = "text"

# Disable auto-browser open
no_browser = false

# Whether to allow killing root-owned processes
allow_root_kill = false

# Alert thresholds
[alerts]
cpu_warning = 80.0
cpu_critical = 95.0
memory_warning = 90.0
memory_critical = 95.0
consecutive_samples = 3

# Stale process detection
[stale_detection]
max_age_hours = 24
max_cpu_percent = 1.0
min_duplicate_count = 3

# Processes that can never be killed
# Default includes kernel_task, launchd, WindowServer, etc.
# protected_processes = ["kernel_task", "launchd", "sysmon"]
```

## API

sysmon exposes a REST + WebSocket API:

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/health` | Health check |
| `GET` | `/api/v1/metrics/system` | Full system metrics |
| `GET` | `/api/v1/metrics/cpu` | CPU metrics only |
| `GET` | `/api/v1/metrics/memory` | Memory metrics only |
| `GET` | `/api/v1/containers` | Docker/Podman containers |
| `GET` | `/api/v1/processes` | Process list (supports `sort_by`, `limit`, `filter`) |
| `GET` | `/api/v1/processes/stale` | Stale processes |
| `GET` | `/api/v1/ports` | Listening TCP/UDP ports |
| `GET` | `/api/v1/temperature` | Temperature sensor data |
| `GET` | `/api/v1/gpu` | GPU metrics |
| `GET` | `/api/v1/history` | Historical metrics (query: `range`, `metric`) |
| `POST` | `/api/v1/actions/kill` | Kill a process |
| `POST` | `/api/v1/actions/kill-batch` | Batch kill multiple processes |
| `POST` | `/api/v1/actions/kill-stale` | Kill stale processes |
| `POST` | `/api/v1/actions/cleanup` | Batch cleanup (zombies + stale) |
| `WS` | `/api/v1/ws` | Real-time metrics stream |

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for full development setup.

### Quick start

```bash
# Install dependencies
cd ui && npm install && cd ..

# Terminal 1: UI dev server
cd ui && npm run dev

# Terminal 2: Rust backend
cargo run
```

### Build

```bash
make build          # Development build
make release        # Optimized release build
make universal      # Universal macOS binary (arm64 + x86_64)
make linux          # Linux native build (run on Linux)
make linux-x86_64   # Cross-compile for Linux x86_64
make linux-aarch64  # Cross-compile for Linux aarch64
```

### Test & Lint

```bash
make test       # Run all tests
make lint       # Run clippy + svelte-check
```

## Makefile Targets

| Target | Description |
|--------|-------------|
| `make build` | Build UI + Rust binary |
| `make dev` | Show development instructions |
| `make ui` | Build Svelte UI only |
| `make release` | Optimized release binary |
| `make universal` | Universal macOS binary (arm64 + x86_64) |
| `make linux` | Linux native build (run on Linux host) |
| `make linux-x86_64` | Cross-compile for Linux x86_64 |
| `make linux-aarch64` | Cross-compile for Linux aarch64 |
| `make install` | Install to `/usr/local/bin` |
| `make uninstall` | Remove from `/usr/local/bin` |
| `make lint` | Run clippy + svelte-check |
| `make test` | Run tests |
| `make clean` | Remove build artifacts |

## Roadmap

- [x] Linux support
- [x] GPU monitoring
- [x] Temperature monitoring
- [x] Historical data persistence
- [ ] Alerts and notifications (desktop/email)
- [ ] Plugin system

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) — System information gathering
- [Axum](https://github.com/tokio-rs/axum) — Web framework
- [Svelte](https://svelte.dev) — Frontend framework
- [rust-embed](https://github.com/pyrossh/rust-embed) — Static file embedding
- [rusqlite](https://github.com/rusqlite/rusqlite) — SQLite bindings for Rust
