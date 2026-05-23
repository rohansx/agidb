//! GLiNER NER wrapper.
//!
//! Ported from `/home/rsx/Desktop/projx/ctxgraph/crates/ctxgraph-extract/src/ner.rs`
//! and adapted to:
//!   - return `agidb_core::types::Entity` (not ctxgraph's own type)
//!   - use `f32` confidence (matching `Entity.confidence`)
//!   - load model + tokenizer via `crate::model_manager::ModelManager`
//!     instead of taking pre-resolved paths
//!   - emit `crate::error::ExtractError` at the boundary
//!
//! Phase 3 plan task 9.

use std::path::PathBuf;

use gliner::model::input::text::TextInput;
use gliner::model::params::Parameters;
use gliner::model::pipeline::span::SpanMode;
use gliner::model::GLiNER;
use orp::params::RuntimeParameters;

use agidb_core::types::Entity;

use crate::error::{ExtractError, Result};
use crate::model_manager::ModelManager;
use crate::models::ModelRef;

/// GLiNER-based entity extractor.
///
/// One process-wide instance per (model, tokenizer) pair is recommended:
/// `new()` loads the ONNX session (~100-500ms) and the tokenizer (~10ms);
/// `extract()` is then a single forward pass.
pub struct NerExtractor {
    model: GLiNER<SpanMode>,
    entity_types: Vec<String>,
}

impl NerExtractor {
    /// Load (or download via [`ModelManager`]) the GLiNER ONNX + tokenizer
    /// and prepare an extractor.
    ///
    /// `threshold` is the GLiNER span probability cutoff (0.0–1.0).
    /// `Parameters::default()` uses 0.5 which is conservative; values
    /// around 0.1–0.3 are typical for tier-B recall use.
    pub fn new(
        model_cache: PathBuf,
        model_ref: ModelRef,
        tokenizer_ref: ModelRef,
        entity_types: Vec<String>,
        threshold: f32,
    ) -> Result<Self> {
        let offline = std::env::var("AGIDB_OFFLINE").is_ok();
        let mgr = ModelManager::new(model_cache, offline);
        let model_path = mgr.ensure_cached(&model_ref)?;
        let tokenizer_path = mgr.ensure_cached(&tokenizer_ref)?;

        let params = Parameters::default().with_threshold(threshold);
        let runtime_params = RuntimeParameters::default();

        let tok_str = tokenizer_path.to_str().ok_or_else(|| {
            ExtractError::InvalidArtifact(format!(
                "non-utf8 tokenizer path: {}",
                tokenizer_path.display()
            ))
        })?;
        let mdl_str = model_path.to_str().ok_or_else(|| {
            ExtractError::InvalidArtifact(format!("non-utf8 model path: {}", model_path.display()))
        })?;

        let model = GLiNER::<SpanMode>::new(params, runtime_params, tok_str, mdl_str)
            .map_err(|e| ExtractError::ModelLoad(format!("gliner load: {e}")))?;

        Ok(Self {
            model,
            entity_types,
        })
    }

    /// Run NER on `text` against the configured entity-type vocabulary.
    /// Returns `Entity` rows with `canonical_name = None` — the alias
    /// resolver in `observe_text` populates that downstream.
    pub fn extract(&self, text: &str) -> Result<Vec<Entity>> {
        let labels: Vec<&str> = self.entity_types.iter().map(String::as_str).collect();
        let input = TextInput::from_str(&[text], &labels)
            .map_err(|e| ExtractError::Inference(format!("gliner input: {e}")))?;
        let output = self
            .model
            .inference(input)
            .map_err(|e| ExtractError::Inference(format!("gliner infer: {e}")))?;

        // `output.spans` is `Vec<Vec<Span>>` — outer per sequence, inner
        // per detected span. We passed one sequence (`&[text]`) so we
        // expect at most one outer entry, but iterate defensively.
        let mut entities = Vec::new();
        for sequence_spans in &output.spans {
            for span in sequence_spans {
                let (start, end) = span.offsets();
                entities.push(Entity {
                    text: span.text().to_string(),
                    entity_type: span.class().to_string(),
                    span: (start, end),
                    confidence: span.probability(),
                    canonical_name: None,
                });
            }
        }
        Ok(entities)
    }
}
