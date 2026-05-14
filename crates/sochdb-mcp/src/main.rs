//! sochdb MCP server — exposes memory_observe, memory_recall, memory_what_about,
//! memory_between, memory_consolidate as MCP tools.
//!
//! Phase 5 lands the full server. This stub exists so the workspace compiles.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    eprintln!("sochdb-mcp — pre-alpha. See docs/phases/phase-5-mcp-python.md.");
    Ok(())
}
