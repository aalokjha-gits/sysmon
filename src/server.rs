use axum::http::{HeaderValue, Method};
use axum::{
    body::Body,
    http::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, Span};

use crate::config::Config;
use crate::embedded::serve_embedded_file;
use crate::handlers::{create_api_router, AppState};
use crate::metrics::MetricsCollector;
use crate::ws::{ws_handler, WebSocketState};

/// Create and configure the Axum application
pub async fn create_server(
    config: Arc<Config>,
    ws_state: WebSocketState,
    metrics_collector: Arc<MetricsCollector>,
) -> anyhow::Result<(Router, u16)> {
    // Create app state
    let app_state = AppState {
        config: config.clone(),
        ws_state: ws_state.clone(),
        metrics_collector,
    };

    // Configure CORS - restrictive, localhost only
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin: &HeaderValue, _| {
            if let Ok(origin_str) = origin.to_str() {
                origin_str.starts_with("http://127.0.0.1")
                    || origin_str.starts_with("http://localhost")
                    || origin_str.starts_with("http://[::1]")
            } else {
                false
            }
        }))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    // Create API router
    let api_router = create_api_router();

    // Create router
    let app = Router::new()
        // WebSocket endpoint
        .route("/api/v1/ws", get(ws_handler))
        // API routes
        .nest("/api/v1", api_router)
        // Static files from embedded assets
        .route("/{*path}", get(serve_embedded_file))
        .route("/", get(serve_embedded_file))
        // Layers
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_response(
                    |_response: &Response, _latency: std::time::Duration, _span: &Span| {
                        // Log response details if needed
                    },
                ),
        )
        .layer(middleware::from_fn(logging_middleware))
        .with_state(app_state);

    // Bind to port
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = TcpListener::bind(&addr).await?;
    let actual_port = listener.local_addr()?.port();

    info!("Server configured to listen on 127.0.0.1:{}", actual_port);

    Ok((app, actual_port))
}

/// Logging middleware for all requests
async fn logging_middleware(request: Request<Body>, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    info!(
        "{} {} - {} in {:?}",
        method,
        uri,
        response.status(),
        duration
    );

    response
}

/// Start the server and return the port it's listening on
pub async fn start_server(app: Router, port: u16) -> anyhow::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(&addr).await?;

    info!("Server listening on http://127.0.0.1:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}
