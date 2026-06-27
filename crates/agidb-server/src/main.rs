//! agidb-server — WebSocket HTTP bridge to the agidb MCP tool surface.
//!
//! Reuses the existing `agidb_mcp::McpServer::handle_request` pure
//! dispatcher — every JSON-RPC frame the browser sends is just forwarded
//! to the MCP layer, which already implements initialize /
//! notifications/initialized / tools/list / tools/call over JSON-RPC 2.0.
//!
//! Wire shape (browser ↔ server, on the WebSocket):
//!   { "jsonrpc": "2.0", "id": 1, "method": "tools/call",
//!     "params": { "name": "memory_observe", "arguments": { "text": "…" } } }
//!   ← { "jsonrpc": "2.0", "id": 1,
//!        "result": { "content": [{"type":"text","text":"…"}], "isError": false } }
//!
//! HTTP routes:
//!   GET  /            — health + version
//!   GET  /healthz     — same
//!   GET  /ws          — upgrade to WebSocket
//!   GET  /tools       — list the registered MCP tools (JSON, for the
//!                       landing page's static chip renderer)
//!
//! CORS: open in dev, locked to the landing origin in prod via
//! `AGIDB_ALLOWED_ORIGIN` env var.

use std::net::SocketAddr;
use std::sync::Arc;

use agidb_mcp::{AgidbContext, McpServer};
use anyhow::{Context, Result};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

mod dispatch;

use dispatch::handle_ws_frame;

/// One store + one dispatcher, shared across all WebSocket connections.
/// We serialize writes through the Mutex so the on-disk store never
/// sees two concurrent transactions.
#[derive(Clone)]
struct AppState {
    server: Arc<McpServer>,
    /// Per-session scratch (the MCP server itself is sync; we wrap it in
    /// a Mutex so concurrent WS frames don't interleave store writes).
    write_lock: Arc<Mutex<()>>,
}

impl AppState {
    fn new(ctx: AgidbContext) -> Self {
        Self {
            server: Arc::new(McpServer::new(ctx)),
            write_lock: Arc::new(Mutex::new(())),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let db_path =
        std::env::var("AGIDB_DB_PATH").unwrap_or_else(|_| "./agidb-demo-data".to_string());
    let bind: SocketAddr = std::env::var("AGIDB_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8765".to_string())
        .parse()
        .context("AGIDB_BIND must be host:port")?;

    info!(db_path = %db_path, bind = %bind, "starting agidb-server");

    // Null extractor by default — the demo doesn't need GLiNER
    // (downloads ~250MB on first call). The /demo page uses
    // text-only observations so tier-C gist recall still works.
    let use_null = std::env::var("AGIDB_NULL_EXTRACTOR")
        .map(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
        .unwrap_or(true);

    let ctx = if use_null {
        info!("using NullExtractor (set AGIDB_NULL_EXTRACTOR=0 to load GLiNER)");
        AgidbContext::open_null(&db_path)
    } else {
        AgidbContext::open(&db_path)
    }
    .context("opening agidb store")?;

    let state = AppState::new(ctx);
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .with_context(|| format!("binding {bind}"))?;
    info!("listening on http://{bind}");
    info!("websocket: ws://{bind}/ws");
    info!("tools list: http://{bind}/tools");
    axum::serve(listener, app).await.context("axum serve")?;
    Ok(())
}

fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any) // dev: open. In prod, set AGIDB_ALLOWED_ORIGIN.
        .allow_headers(Any);

    Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz))
        .route("/tools", get(list_tools))
        .route("/ws", get(ws_upgrade))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

async fn root() -> impl IntoResponse {
    Json(json!({
        "name": "agidb-server",
        "version": env!("CARGO_PKG_VERSION"),
        "endpoints": {
            "websocket": "/ws",
            "tools":     "/tools",
            "healthz":   "/healthz"
        }
    }))
}

async fn healthz() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

/// Static tool list. The landing page's chip renderer hits this on load
/// so the suggested-command chips reflect the actual MCP tool surface
/// (rather than being hard-coded in the page markup).
async fn list_tools(State(state): State<AppState>) -> impl IntoResponse {
    let _ = state; // state kept for future per-session tool listing
    let mut out: Vec<Value> = Vec::new();
    for tool in agidb_mcp::tools::registry() {
        out.push(json!({
            "name":        tool.name,
            "description": tool.description,
            "inputSchema": (tool.schema)(),
        }));
    }
    Json(json!({
        "count": out.len(),
        "tools": out,
        "phase_aliases": dispatch::phase_aliases_json(),
    }))
}

async fn ws_upgrade(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_loop(socket, state))
}

async fn ws_loop(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    info!("ws: client connected");

    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                warn!(?e, "ws: receive error");
                break;
            }
        };
        match msg {
            Message::Text(t) => {
                let response = handle_ws_frame(&state, t.as_str()).await;
                if let Some(resp) = response {
                    match serde_json::to_string(&resp) {
                        Ok(s) => {
                            if sender.send(Message::Text(s.into())).await.is_err() {
                                warn!("ws: send failed, dropping");
                                break;
                            }
                        }
                        Err(e) => {
                            warn!(?e, "ws: failed to serialize response");
                        }
                    }
                }
                // Notifications get no reply — match the JSON-RPC spec.
            }
            Message::Close(_) => {
                info!("ws: client closed");
                break;
            }
            Message::Ping(p) => {
                if sender.send(Message::Pong(p)).await.is_err() {
                    break;
                }
            }
            Message::Pong(_) => {}
            Message::Binary(_) => {
                warn!("ws: ignoring binary frame (JSON-RPC is text-only)");
            }
        }
    }
    info!("ws: connection closed");
}
