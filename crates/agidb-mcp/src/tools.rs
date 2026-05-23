//! Tool definitions for the agidb MCP server.
//!
//! Each tool has a JSON-Schema input shape, a stable name, and a handler
//! that takes the unified [`AgidbContext`] (store + extractor) plus the
//! caller's parsed args, and returns a structured result.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use agidb_core::types::Query;

use crate::context::AgidbContext;
use crate::protocol::{McpError, ToolDescriptor, ToolResult};

pub type ToolFn = fn(&AgidbContext, Value) -> Result<ToolResult, McpError>;

pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub schema: fn() -> Value,
    pub handler: ToolFn,
}

/// The full tool registry exposed by this server. Order is the order
/// `tools/list` returns; stable so clients can cache.
pub fn registry() -> Vec<Tool> {
    vec![
        Tool {
            name: "memory_observe",
            description:
                "Record a new observation. Runs layer-2 extraction (or stores text-only when no \
                 model is loaded) and persists an Episode with bi-temporal stamps.",
            schema: observe_schema,
            handler: observe,
        },
        Tool {
            name: "memory_recall",
            description:
                "Tiered recall against the store. Never returns the empty set; the deepest tier \
                 (NearestNeighbor) always emits at least one match unless `tier_floor` caps it.",
            schema: recall_schema,
            handler: recall,
        },
        Tool {
            name: "memory_consolidate",
            description:
                "Run the consolidation worker once: cluster recent episodes into SemanticAtoms, \
                 detect contradictions, write an audit-log entry.",
            schema: consolidate_schema,
            handler: consolidate,
        },
        Tool {
            name: "memory_get_episode",
            description: "Fetch a single Episode by id.",
            schema: get_episode_schema,
            handler: get_episode,
        },
    ]
}

/// Render the registry as the `tools/list` MCP response payload.
pub fn list() -> Vec<ToolDescriptor> {
    registry()
        .into_iter()
        .map(|t| ToolDescriptor {
            name: t.name.to_string(),
            description: t.description.to_string(),
            input_schema: (t.schema)(),
        })
        .collect()
}

/// Dispatch a `tools/call` request to the appropriate handler.
pub fn call(ctx: &AgidbContext, name: &str, args: Value) -> Result<ToolResult, McpError> {
    for tool in registry() {
        if tool.name == name {
            return (tool.handler)(ctx, args);
        }
    }
    Err(McpError::InvalidParams(format!("unknown tool: {name}")))
}

// ---------------------------------------------------------------------------
// memory_observe
// ---------------------------------------------------------------------------

fn observe_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "text": {
                "type": "string",
                "description": "The raw observation to record."
            },
            "source": {
                "type": "string",
                "description": "Caller-supplied provenance label (e.g. 'user', 'tool:gmail').",
                "default": "mcp"
            }
        },
        "required": ["text"]
    })
}

#[derive(Deserialize)]
struct ObserveArgs {
    text: String,
    #[serde(default = "default_source")]
    source: String,
}

fn default_source() -> String {
    "mcp".into()
}

#[derive(Serialize)]
struct ObserveResult {
    episode_id: u64,
}

fn observe(ctx: &AgidbContext, args: Value) -> Result<ToolResult, McpError> {
    let args: ObserveArgs = serde_json::from_value(args)?;
    let id = ctx.observe_text(&args.text, &args.source)?;
    Ok(ToolResult::json(&serde_json::to_value(ObserveResult {
        episode_id: id.raw(),
    })?))
}

// ---------------------------------------------------------------------------
// memory_recall
// ---------------------------------------------------------------------------

fn recall_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "cue": {
                "type": "string",
                "description": "The retrieval cue. Tokenized for tier-A concept lookup and \
                                encoded into a gist signature for tier-C/D fallback."
            },
            "k": {
                "type": "integer",
                "minimum": 1,
                "description": "Maximum matches to return.",
                "default": 10
            },
            "min_confidence": {
                "type": "number",
                "minimum": 0.0,
                "maximum": 1.0,
                "description": "Confidence floor; matches below this are dropped.",
                "default": 0.0
            }
        },
        "required": ["cue"]
    })
}

#[derive(Deserialize)]
struct RecallArgs {
    cue: String,
    #[serde(default = "default_k")]
    k: usize,
    #[serde(default)]
    min_confidence: f32,
}

fn default_k() -> usize {
    10
}

#[derive(Serialize)]
struct RecallResult {
    tier_used: String,
    elapsed_ms: u32,
    matches: Vec<RecallMatchOut>,
    semantic_atoms: Vec<SemanticOut>,
}

#[derive(Serialize)]
struct RecallMatchOut {
    episode_id: u64,
    text: String,
    confidence: f32,
    tier: String,
}

#[derive(Serialize)]
struct SemanticOut {
    atom_id: u64,
    statement: String,
    confidence: f32,
}

fn recall(ctx: &AgidbContext, args: Value) -> Result<ToolResult, McpError> {
    let args: RecallArgs = serde_json::from_value(args)?;
    let query = Query::cue(args.cue)
        .with_k(args.k)
        .with_min_confidence(args.min_confidence);
    let r = ctx.recall(&query)?;
    let payload = RecallResult {
        tier_used: format!("{:?}", r.tier_used),
        elapsed_ms: r.elapsed_ms,
        matches: r
            .matches
            .into_iter()
            .map(|m| RecallMatchOut {
                episode_id: m.episode_id.raw(),
                text: m.text,
                confidence: m.confidence,
                tier: format!("{:?}", m.source_tier),
            })
            .collect(),
        semantic_atoms: r
            .semantic_atoms
            .into_iter()
            .map(|a| SemanticOut {
                atom_id: a.atom_id.raw(),
                statement: a.statement,
                confidence: a.confidence,
            })
            .collect(),
    };
    Ok(ToolResult::json(&serde_json::to_value(payload)?))
}

// ---------------------------------------------------------------------------
// memory_consolidate
// ---------------------------------------------------------------------------

fn consolidate_schema() -> Value {
    json!({ "type": "object", "properties": {} })
}

#[derive(Serialize)]
struct ConsolidateResult {
    episodes_scanned: u32,
    semantic_atoms_created: u32,
    contradictions_detected: u32,
}

fn consolidate(ctx: &AgidbContext, _args: Value) -> Result<ToolResult, McpError> {
    let r = ctx.consolidate()?;
    let payload = ConsolidateResult {
        episodes_scanned: r.episodes_scanned,
        semantic_atoms_created: r.semantic_atoms_created,
        contradictions_detected: r.contradictions_detected,
    };
    Ok(ToolResult::json(&serde_json::to_value(payload)?))
}

// ---------------------------------------------------------------------------
// memory_get_episode
// ---------------------------------------------------------------------------

fn get_episode_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": { "type": "integer", "minimum": 0, "description": "EpisodeId" }
        },
        "required": ["id"]
    })
}

#[derive(Deserialize)]
struct GetEpisodeArgs {
    id: u64,
}

fn get_episode(ctx: &AgidbContext, args: Value) -> Result<ToolResult, McpError> {
    let args: GetEpisodeArgs = serde_json::from_value(args)?;
    match ctx.get_episode(args.id)? {
        Some(ep) => Ok(ToolResult::json(&json!({
            "id": ep.id.raw(),
            "text": ep.text,
            "confidence": ep.confidence,
            "triples": ep.triples.iter().map(|t| json!({
                "subject": t.subject,
                "predicate": t.predicate,
                "object": t.object,
                "confidence": t.confidence,
            })).collect::<Vec<_>>(),
            "valid_time": {
                "start": ep.valid_time.start.to_rfc3339(),
                "end": ep.valid_time.end.map(|e| e.to_rfc3339()),
            },
        }))),
        None => Ok(ToolResult::error(format!("episode {} not found", args.id))),
    }
}
