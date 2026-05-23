//! MCP server core: dispatch + stdio loop.
//!
//! `handle_request` is pure (request in → response out); tests call it
//! directly. `run_stdio` is the production driver — reads line-delimited
//! JSON from stdin, dispatches, writes responses to stdout.

use std::io::{self, BufRead, Write};

use serde_json::{json, Value};

use crate::context::AgidbContext;
use crate::protocol::{
    error_code, Capabilities, InitializeResult, JsonRpcRequest, JsonRpcResponse, McpError,
    ServerInfo, ToolListing, ToolsCapability, JSONRPC_VERSION, MCP_PROTOCOL_VERSION,
};
use crate::tools;

const SERVER_NAME: &str = "agidb-mcp";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct McpServer {
    ctx: AgidbContext,
}

impl McpServer {
    pub fn new(ctx: AgidbContext) -> Self {
        Self { ctx }
    }

    /// Pure dispatch. Returns `Some(response)` for requests and `None`
    /// for notifications (which the JSON-RPC spec says receive no reply).
    pub fn handle_request(&self, request: JsonRpcRequest) -> Option<JsonRpcResponse> {
        if request.jsonrpc != JSONRPC_VERSION {
            return Some(JsonRpcResponse::err(
                request.id.unwrap_or(Value::Null),
                error_code::INVALID_REQUEST,
                format!("unsupported jsonrpc version: {}", request.jsonrpc),
            ));
        }

        // Notifications carry no `id`; we honor them but emit no reply.
        let is_notification = request.id.is_none();
        let id = request.id.clone().unwrap_or(Value::Null);

        let result = match request.method.as_str() {
            "initialize" => Ok(handle_initialize()),
            "notifications/initialized" => {
                // Client signaled it's ready. No reply, no work needed.
                return None;
            }
            "ping" => Ok(json!({})),
            "tools/list" => Ok(handle_tools_list()),
            "tools/call" => handle_tools_call(&self.ctx, request.params),
            other => Err(McpError::InvalidRequest(format!(
                "method not found: {other}"
            ))),
        };

        if is_notification {
            return None;
        }

        match result {
            Ok(value) => Some(JsonRpcResponse::ok(id, value)),
            Err(err) => Some(JsonRpcResponse::err(id, err.code(), err.to_string())),
        }
    }

    /// Read JSON-RPC messages from stdin (one per line), dispatch each,
    /// write responses to stdout. Loops until EOF.
    pub fn run_stdio(&self) -> anyhow::Result<()> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = stdin.lock();
        let mut out = stdout.lock();
        let mut line = String::new();

        loop {
            line.clear();
            let n = reader.read_line(&mut line)?;
            if n == 0 {
                break; // EOF
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let request = match serde_json::from_str::<JsonRpcRequest>(trimmed) {
                Ok(r) => r,
                Err(e) => {
                    let resp = JsonRpcResponse::err(
                        Value::Null,
                        error_code::PARSE_ERROR,
                        format!("invalid JSON: {e}"),
                    );
                    writeln!(out, "{}", serde_json::to_string(&resp)?)?;
                    out.flush()?;
                    continue;
                }
            };

            if let Some(response) = self.handle_request(request) {
                writeln!(out, "{}", serde_json::to_string(&response)?)?;
                out.flush()?;
            }
        }
        Ok(())
    }
}

fn handle_initialize() -> Value {
    let result = InitializeResult {
        protocol_version: MCP_PROTOCOL_VERSION,
        server_info: ServerInfo {
            name: SERVER_NAME,
            version: SERVER_VERSION,
        },
        capabilities: Capabilities {
            tools: ToolsCapability {
                list_changed: false,
            },
        },
    };
    serde_json::to_value(result).expect("static struct serializes")
}

fn handle_tools_list() -> Value {
    serde_json::to_value(ToolListing {
        tools: tools::list(),
    })
    .expect("static struct serializes")
}

fn handle_tools_call(ctx: &AgidbContext, params: Option<Value>) -> Result<Value, McpError> {
    let params = params.ok_or_else(|| McpError::InvalidParams("missing params".into()))?;
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("missing tools/call params.name".into()))?
        .to_string();
    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
    let result = tools::call(ctx, &name, arguments)?;
    serde_json::to_value(result).map_err(|e| McpError::Internal(e.to_string()))
}
