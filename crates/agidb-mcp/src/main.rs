//! agidb-mcp binary entrypoint.
//!
//! Usage:
//!   agidb-mcp <db_path>
//!
//! Opens or creates a store at `<db_path>`, attempts to load the layer-2
//! Extractor (falls back to `NullExtractor` if the model cache is cold),
//! and runs the JSON-RPC server over stdio. Clients (Claude Desktop,
//! Cursor, etc.) drive it via the MCP `tools/call` mechanism.

use agidb_mcp::{AgidbContext, McpServer};
use anyhow::Result;

fn main() -> Result<()> {
    // Logs go to stderr; stdout is reserved for JSON-RPC frames.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let db_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./agidb-data".to_string());
    tracing::info!(db_path = %db_path, "starting agidb-mcp");

    let ctx = AgidbContext::open(&db_path)?;
    let server = McpServer::new(ctx);
    server.run_stdio()
}
