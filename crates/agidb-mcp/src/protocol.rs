//! JSON-RPC 2.0 + MCP message types.
//!
//! Implements only the subset of the [Model Context Protocol] this server
//! needs: `initialize`, `notifications/initialized`, `tools/list`,
//! `tools/call`. Any unknown method returns `-32601 Method not found`.
//!
//! [Model Context Protocol]: https://modelcontextprotocol.io

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const JSONRPC_VERSION: &str = "2.0";
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// Standard JSON-RPC error codes plus the MCP-specific ones we use.
pub mod error_code {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    /// MCP-specific: tool execution failed (caller's fault, e.g. bad args
    /// the schema didn't catch).
    pub const TOOL_ERROR: i32 = -32000;
}

/// One JSON-RPC request as we accept it on stdin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    /// Notifications omit `id`; we treat `null` and missing the same way.
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

/// A JSON-RPC response. Either `result` xor `error` is set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn ok(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn err(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// MCP-specific result shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: &'static str,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    pub name: &'static str,
    pub version: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct Capabilities {
    pub tools: ToolsCapability,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolsCapability {
    /// Server pushes notifications/tools/list_changed if the tool list
    /// changes at runtime; we never do, so this is always `false`.
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolListing {
    pub tools: Vec<ToolDescriptor>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Per the MCP spec, a tool result is a list of content blocks. We
/// always return one block — either text (for human-readable summaries)
/// or a JSON-stringified payload (for structured data the agent can
/// re-parse).
#[derive(Debug, Clone, Serialize)]
pub struct ToolResult {
    pub content: Vec<ContentBlock>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ContentBlock {
    Text { text: String },
}

impl ToolResult {
    pub fn text(s: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::Text { text: s.into() }],
            is_error: None,
        }
    }

    pub fn json(value: &Value) -> Self {
        Self::text(value.to_string())
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::Text { text: msg.into() }],
            is_error: Some(true),
        }
    }
}

// ---------------------------------------------------------------------------
// Typed errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("invalid params: {0}")]
    InvalidParams(String),

    #[error("tool error: {0}")]
    Tool(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl McpError {
    pub fn code(&self) -> i32 {
        match self {
            Self::InvalidRequest(_) => error_code::INVALID_REQUEST,
            Self::InvalidParams(_) => error_code::INVALID_PARAMS,
            Self::Tool(_) => error_code::TOOL_ERROR,
            Self::Internal(_) => error_code::INTERNAL_ERROR,
        }
    }
}

impl From<serde_json::Error> for McpError {
    fn from(e: serde_json::Error) -> Self {
        Self::InvalidParams(e.to_string())
    }
}

impl From<agidb_core::AgidbError> for McpError {
    fn from(e: agidb_core::AgidbError) -> Self {
        Self::Tool(e.to_string())
    }
}
