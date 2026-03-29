# Contributing to sysmon

Thanks for your interest in contributing! This guide will help you get set up.

## Prerequisites

- **Rust** 1.75+ ([rustup.rs](https://rustup.rs))
- **Node.js** 20+ ([nodejs.org](https://nodejs.org))
- **npm** (comes with Node.js)
- **macOS** (primary development platform)

## Getting Started

```bash
# Clone the repo
git clone https://github.com/aalokjha-gits/sysmon.git
cd sysmon

# Install UI dependencies
cd ui && npm install && cd ..

# Build everything
make build

# Run
./target/release/sysmon
```

## Development Workflow

### Running in dev mode

You need two terminals:

```bash
# Terminal 1: Svelte dev server (with hot reload)
cd ui && npm run dev

# Terminal 2: Rust backend
cargo run
```

The Svelte dev server proxies API requests to the Rust backend (configured in `ui/vite.config.ts`).

### Building

```bash
make build      # Build UI then Rust binary
make release    # Optimized + stripped binary
make universal  # macOS universal binary (arm64 + x86_64)
```

### Testing

```bash
make test       # Run Rust tests
make lint       # Run clippy + svelte-check
```

Please ensure `make lint` and `make test` pass before submitting a PR.

## Project Structure

```
sysmon/
├── src/                    # Rust backend
│   ├── main.rs             # Entry point, CLI, startup
│   ├── server.rs           # Axum server setup
│   ├── config.rs           # TOML configuration
│   ├── models.rs           # Shared data types
│   ├── ws.rs               # WebSocket handling
│   ├── embedded.rs         # Static file serving (rust-embed)
│   ├── handlers/           # API route handlers
│   │   └── mod.rs
│   ├── metrics/            # System metrics collection
│   │   ├── mod.rs          # MetricsCollector + alerts
│   │   ├── cpu.rs
│   │   ├── memory.rs
│   │   ├── disk.rs
│   │   ├── network.rs
│   │   ├── process.rs
│   │   └── container.rs
│   └── actions/            # Process management actions
│       ├── kill.rs
│       └── cleanup.rs
├── ui/                     # Svelte 5 frontend
│   ├── src/
│   │   ├── routes/         # SvelteKit pages
│   │   ├── lib/
│   │   │   ├── components/ # UI components
│   │   │   ├── stores/     # Svelte stores (state + WebSocket)
│   │   │   └── types/      # TypeScript type definitions
│   │   └── app.css         # Global styles
│   └── vite.config.ts
├── .github/                # CI/CD workflows
├── homebrew/               # Homebrew formula
├── Cargo.toml
├── Makefile
└── install.sh              # Curl install script
```

## Code Style

### Rust

- Follow standard Rust conventions (`rustfmt`, `clippy`)
- All warnings treated as errors in CI (`clippy -- -D warnings`)
- Handle errors explicitly — no `unwrap()` in production paths
- Use `tracing` for logging, not `println!`
- Keep `unsafe` to an absolute minimum (currently none)

### TypeScript / Svelte

- Strict TypeScript (`strict: true` in tsconfig)
- Svelte 5 runes (`$state`, `$derived`, `$effect`) — not legacy stores in components
- Shared state uses Svelte writable/derived stores in `stores/`
- Component-scoped `<style>` blocks, global styles in `app.css`
- CSS custom properties for theming — no CSS framework

### General

- Keep PRs focused — one feature or fix per PR
- Write descriptive commit messages
- Add types for any new data structures (both Rust `models.rs` and TS `types/index.ts`)

## Making Changes

### Adding a new metric

1. Create a collector in `src/metrics/` (e.g., `gpu.rs`)
2. Add the data type to `src/models.rs`
3. Wire it into `MetricsCollector::collect()` in `src/metrics/mod.rs`
4. Add a corresponding TypeScript type in `ui/src/lib/types/index.ts`
5. Create UI components in `ui/src/lib/components/`
6. Add to the dashboard layout in `ui/src/routes/+page.svelte`

### Adding an API endpoint

1. Add the handler function in `src/handlers/mod.rs`
2. Register the route in `create_api_router()`
3. Add types to `src/models.rs` if needed
4. Add the client function in `ui/src/lib/stores/metrics.ts`

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Run `make lint && make test`
5. Commit with a clear message
6. Push and open a PR against `main`

### PR checklist

- [ ] `make lint` passes
- [ ] `make test` passes
- [ ] New features have corresponding types in both Rust and TypeScript
- [ ] Responsive layout tested (if touching UI)
- [ ] No `as any`, `@ts-ignore`, or `unwrap()` in new code

## Reporting Issues

When filing a bug report, please include:

- macOS version
- sysmon version (`sysmon --version`)
- Steps to reproduce
- Expected vs actual behavior
- Console output if relevant

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
