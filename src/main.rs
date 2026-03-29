use anyhow::Result;
use clap::{Parser, Subcommand};
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
mod service;
mod ws;

use config::Config;
use metrics::MetricsCollector;
use server::{create_server, start_server};
use ws::WebSocketState;

#[derive(Parser, Debug)]
#[command(name = "sysmon")]
#[command(about = "Lightweight system monitoring dashboard")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, env = "SYSMON_PORT")]
    port: Option<u16>,

    #[arg(long, env = "SYSMON_NO_BROWSER")]
    no_browser: bool,

    #[arg(long, env = "SYSMON_FORMAT")]
    json: bool,

    #[arg(short, long, env = "SYSMON_INTERVAL", value_name = "MS")]
    interval: Option<u64>,

    #[arg(short, long, env = "SYSMON_CONFIG")]
    config: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage sysmon as a background service (macOS launchd)
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
}

#[derive(Subcommand, Debug)]
enum ServiceAction {
    /// Install and start sysmon as a login service
    Install,
    /// Stop and remove the sysmon service
    Uninstall,
    /// Start the service
    Start,
    /// Stop the service
    Stop,
    /// Restart the service
    Restart,
    /// Show service status
    Status,
    /// Tail service logs
    Logs,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::Service { action }) = cli.command {
        return match action {
            ServiceAction::Install => service::install(),
            ServiceAction::Uninstall => service::uninstall(),
            ServiceAction::Start => service::start(),
            ServiceAction::Stop => service::stop(),
            ServiceAction::Restart => service::restart(),
            ServiceAction::Status => service::status(),
            ServiceAction::Logs => service::logs(),
        };
    }

    let log_level = if cli.json { Level::WARN } else { Level::INFO };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let mut config = Config::load(cli.config.as_deref())?;

    config.merge_cli_args(cli.port, cli.interval, cli.no_browser);
    if cli.json {
        config.format = "json".to_string();
    }

    let config = Arc::new(config);

    info!("Starting sysmon v{}", env!("CARGO_PKG_VERSION"));

    let ws_state = WebSocketState::new();

    let metrics_collector = Arc::new(MetricsCollector::new(config.clone()));

    let _collection_task = metrics_collector
        .clone()
        .start_collection_task(ws_state.clone());

    let (app, port) = create_server(config.clone(), ws_state.clone(), metrics_collector).await?;

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

    if !config.no_browser {
        info!("Opening browser to {}", url);
        if let Err(e) = webbrowser::open(&url) {
            warn!("Failed to open browser: {}", e);
        }
    }

    let server_task = tokio::spawn(async move {
        if let Err(e) = start_server(app, port).await {
            eprintln!("Server error: {}", e);
            std::process::exit(1);
        }
    });

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal, stopping server...");
        }
        _ = server_task => {
            info!("Server task completed");
        }
    }

    info!("Shutdown complete");

    Ok(())
}
