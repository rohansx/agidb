//! Phase-3 v1 `Extractor`: orchestrates NER + heuristic relation
//! extraction + temporal parsing into a single `TextExtractor` impl.
//!
//! Phase-3 v2 will replace [`crate::heuristic_relations`] with a
//! proper ONNX-based relation extractor (GLiREL or relex). The shape
//! here — load models in `new()`, run a fast forward pass in
//! `extract()` — stays the same.

use std::path::PathBuf;

use agidb_core::types::{ExtractContext, Extraction, TextExtractor};
use agidb_core::AgidbError;

use crate::error::Result;
use crate::heuristic_relations::extract_heuristic_relations;
use crate::models::{ModelRef, GLINER_DEFAULT, GLINER_TOKENIZER_DEFAULT};
use crate::ner::NerExtractor;
use crate::predicates::PredicateTable;
use crate::temporal::parse_time_anchor;

/// Configuration for [`Extractor::new`].
#[derive(Clone, Debug)]
pub struct ExtractorConfig {
    pub model_cache: PathBuf,
    pub gliner_model: ModelRef,
    pub gliner_tokenizer: ModelRef,
    pub entity_types: Vec<String>,
    pub predicate_synonyms: PredicateTable,
    /// GLiNER span-probability threshold (0.0–1.0). Lower = more recall,
    /// more noise. The phase-3 design spec settled on 0.3 as a starting
    /// point; tune during the F1 iteration loop.
    pub ner_threshold: f32,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            model_cache: dirs::cache_dir()
                .map(|d| d.join("agidb/models"))
                .unwrap_or_else(|| PathBuf::from("./.agidb-models")),
            gliner_model: GLINER_DEFAULT.clone(),
            gliner_tokenizer: GLINER_TOKENIZER_DEFAULT.clone(),
            entity_types: vec![
                "Person".into(),
                "Place".into(),
                "Organization".into(),
                "Thing".into(),
                "Event".into(),
            ],
            predicate_synonyms: PredicateTable::default(),
            ner_threshold: 0.3,
        }
    }
}

/// The phase-3 v1 extractor. Holds one GLiNER session + a predicate
/// table; runs NER + heuristic relations + temporal in `extract()`.
pub struct Extractor {
    ner: NerExtractor,
    predicates: PredicateTable,
}

impl Extractor {
    /// Load the GLiNER ONNX + tokenizer (downloads to the cache on first
    /// use; honors `AGIDB_OFFLINE`) and build the extractor.
    pub fn new(cfg: ExtractorConfig) -> Result<Self> {
        let ner = NerExtractor::new(
            cfg.model_cache,
            cfg.gliner_model,
            cfg.gliner_tokenizer,
            cfg.entity_types,
            cfg.ner_threshold,
        )?;
        Ok(Self {
            ner,
            predicates: cfg.predicate_synonyms,
        })
    }
}

impl TextExtractor for Extractor {
    fn extract(&self, text: &str, ctx: &ExtractContext) -> agidb_core::Result<Extraction> {
        let raw_entities = self.ner.extract(text).map_err(AgidbError::from)?;
        let triples = extract_heuristic_relations(text, &raw_entities, &self.predicates);
        let valid_time = parse_time_anchor(text, ctx.observation_time);
        Ok(Extraction {
            triples,
            valid_time,
            raw_entities,
        })
    }
}
