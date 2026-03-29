use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

mod actions;
mod config;
mod embedded;
mod handlers;
mod metrics;
mod models;
mod server;
mod ws;

use config::Config;
use metrics::MetricsCollector;
use server::{create_server, start_server};
use ws::WebSocketState;

/// System monitoring dashboard for macOS
#[derive(Parser, Debug)]
#[command(name = "sysmon")]
#[command(about = "Lightweight system monitoring dashboard")]
#[command(version)]
struct Cli {
    /// Server port (0 for ephemeral port)
    #[arg(short, long, env = "SYSMON_PORT")]
    port: Option<u16>,

    /// Disable auto-browser open
    #[arg(long, env = "SYSMON_NO_BROWSER")]
    no_browser: bool,

    /// Output format: json or text
    #[arg(long, env = "SYSMON_FORMAT")]
    json: bool,

    /// Metrics collection interval in milliseconds
    #[arg(short, long, env = "SYSMON_INTERVAL", value_name = "MS")]
    interval: Option<u64>,

    /// Configuration file path
    #[arg(short, long, env = "SYSMON_CONFIG")]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.json { Level::WARN } else { Level::INFO };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    // Load configuration
    let mut config = Config::load(cli.config.as_deref())?;

    // Merge CLI arguments into config
    config.merge_cli_args(cli.port, cli.interval, cli.no_browser);
    if cli.json {
        config.format = "json".to_string();
    }

    let config = Arc::new(config);

    info!("Starting sysmon v{}", env!("CARGO_PKG_VERSION"));

    // Create WebSocket state
    let ws_state = WebSocketState::new();

    // Create metrics collector
    let metrics_collector = Arc::new(MetricsCollector::new(config.clone()));

    // Start metrics collection in background
    let _collection_task = metrics_collector
        .clone()
        .start_collection_task(ws_state.clone());

    // Create and start server
    let (app, port) = create_server(config.clone(), ws_state.clone(), metrics_collector).await?;

    // Print startup info
    let url = format!("http://127.0.0.1:{}", port);

    if config.format == "json" {
        println!(
            "{}",
            serde_json::json!({
                "status": "started",
                "port": port,
                "url": url,
            })
        );
    } else {
        println!("\n╔══════════════════════════════════════════╗");
        println!("║           Sysmon Dashboard               ║");
        println!("╚══════════════════════════════════════════╝");
        println!("\n  Server running at: {}", url);
        println!("  Metrics interval:  {}ms", config.interval_ms);
        println!("  Press Ctrl+C to stop\n");
    }

    // Open browser if enabled
    if !config.no_browser {
        info!("Opening browser to {}", url);
        if let Err(e) = webbrowser::open(&url) {
            warn!("Failed to open browser: {}", e);
        }
    }

    // Start server
    let server_task = tokio::spawn(async move {
        if let Err(e) = start_server(app, port).await {
            eprintln!("Server error: {}", e);
            std::process::exit(1);
        }
    });

    // Wait for shutdown signal
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal, stopping server...");
        }
        _ = server_task => {
            info!("Server task completed");
        }
    }

    // Graceful shutdown
    info!("Shutdown complete");

    Ok(())
}
