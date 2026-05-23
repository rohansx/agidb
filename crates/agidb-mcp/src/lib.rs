//! agidb MCP server — a stdio JSON-RPC server exposing the agidb engine
//! as MCP tools (`memory_observe`, `memory_recall`, `memory_consolidate`,
//! `memory_get_episode`) to Claude Desktop, Cursor, and other
//! MCP-compatible agents.
//!
//! Phase 5 of the agidb v2 build. See
//! `docs/phases/phase-5-mcp-python.md`.
//!
//! Layout:
//! - [`protocol`] — JSON-RPC + MCP message types (parsing, errors).
//! - [`context`] — `AgidbContext`: Store + Extractor wrapper, the surface tools dispatch through.
//! - [`tools`] — tool registry + per-tool schema + handler.
//! - [`server`] — `McpServer`: pure `handle_request` + stdio driver.

pub mod context;
pub mod protocol;
pub mod server;
pub mod tools;

pub use crate::context::{AgidbContext, AgidbExtractor};
pub use crate::server::McpServer;
