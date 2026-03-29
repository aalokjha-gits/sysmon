use crate::models::{SystemMetrics, WebSocketMessage};
use axum::{
    extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// WebSocket state shared across handlers
#[derive(Clone)]
pub struct WebSocketState {
    /// Broadcast channel for metrics updates
    pub tx: broadcast::Sender<WebSocketMessage>,
    /// Server start time for uptime calculation
    pub start_time: std::time::Instant,
    /// Per-instance sequence counter for WebSocket messages
    sequence: Arc<AtomicU64>,
}

impl WebSocketState {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self {
            tx,
            start_time: std::time::Instant::now(),
            sequence: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get next sequence number
    pub fn next_sequence(&self) -> u64 {
        self.sequence.fetch_add(1, Ordering::SeqCst)
    }

    /// Broadcast a metrics update to all connected clients
    pub fn broadcast_metrics(&self, metrics: SystemMetrics) {
        let sequence = self.next_sequence();
        let message = WebSocketMessage::metrics_update(sequence, metrics);

        match self.tx.send(message) {
            Ok(count) => {
                debug!("Broadcasted metrics to {} clients", count);
            }
            Err(e) => {
                warn!("Failed to broadcast metrics: {}", e);
            }
        }
    }

    /// Get current sequence number
    pub fn current_sequence(&self) -> u64 {
        self.sequence.load(Ordering::SeqCst)
    }

    /// Get server uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

impl Default for WebSocketState {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<crate::handlers::AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state.ws_state))
}

/// Handle a WebSocket connection
async fn handle_socket(socket: WebSocket, state: WebSocketState) {
    info!("New WebSocket connection established");

    let mut rx = state.tx.subscribe();

    let (mut sender, mut receiver) = socket.split();

    // Spawn a task to handle incoming messages (pings, etc.)
    let recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(msg) => {
                    if let Message::Text(text) = msg {
                        debug!("Received WebSocket message: {}", text);

                        // Handle client messages (ping responses, etc.)
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(msg_type) = json.get("type").and_then(|t| t.as_str()) {
                                match msg_type {
                                    "pong" => {
                                        debug!("Received pong from client");
                                    }
                                    "subscribe" => {
                                        debug!("Client subscribed to updates");
                                    }
                                    _ => {
                                        debug!("Unknown message type: {}", msg_type);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("WebSocket receive error: {}", e);
                    break;
                }
            }
        }
    });

    // Send initial ping
    let ping = WebSocketMessage::ping(0);
    if let Ok(ping_json) = serde_json::to_string(&ping) {
        let _ = sender.send(Message::Text(Utf8Bytes::from(ping_json))).await;
    }

    // Main loop: broadcast messages to client
    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(message) => {
                        match serde_json::to_string(&message) {
                            Ok(json) => {
                                if let Err(e) = sender.send(Message::Text(Utf8Bytes::from(json))).await {
                                    error!("Failed to send WebSocket message: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Failed to serialize message: {}", e);
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebSocket client lagged behind by {} messages", n);
                        // Continue receiving, the lagged messages are dropped
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Broadcast channel closed");
                        break;
                    }
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                // Send periodic ping to keep connection alive
                let ping = WebSocketMessage::ping(state.current_sequence());
                if let Ok(ping_json) = serde_json::to_string(&ping) {
                    if let Err(e) = sender.send(Message::Text(Utf8Bytes::from(ping_json))).await {
                        error!("Failed to send ping: {}", e);
                        break;
                    }
                }
            }
        }
    }

    // Clean up
    recv_task.abort();
    info!("WebSocket connection closed");
}


