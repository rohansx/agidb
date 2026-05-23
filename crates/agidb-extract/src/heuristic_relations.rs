//! Heuristic relation extractor — phase-3 v1 stub.
//!
//! **This is NOT the final design.** It's the smallest thing that
//! produces real `ExtractedTriple`s from real NER entities, so the
//! end-to-end pipeline (`observe_text(extractor)`) works without
//! depending on the GLiREL / relex ONNX port (which is heavier and
//! deferred to phase-3 v2). F1 from this extractor alone will be well
//! below the 0.85 exit gate — replacing it with a proper port is what
//! gets us there.
//!
//! Algorithm: walk adjacent entity pairs sorted by span; look at the
//! text **between** the two entities. Try the whole between-phrase
//! against the predicate table first (catches multi-word predicates
//! like "told me about"); else token-by-token. First match wins; emit a
//! triple at `confidence = 0.5`.
//!
//! No ML. No model. No network. Just `PredicateTable` lookup.

use agidb_core::types::{Entity, ExtractedTriple};

use crate::predicates::{canonicalize, PredicateTable};

/// Conservative confidence for heuristic-derived relations.
const HEURISTIC_CONFIDENCE: f32 = 0.5;

/// Extract `(subject, predicate, object)` triples by scanning the text
/// between adjacent entities for known predicate surface forms.
pub fn extract_heuristic_relations(
    text: &str,
    entities: &[Entity],
    predicates: &PredicateTable,
) -> Vec<ExtractedTriple> {
    if entities.len() < 2 {
        return Vec::new();
    }
    // Sort by span start so adjacency walks the text in order.
    let mut sorted: Vec<&Entity> = entities.iter().collect();
    sorted.sort_by_key(|e| e.span.0);

    let mut triples = Vec::new();
    for window in sorted.windows(2) {
        let a = window[0];
        let b = window[1];
        if let Some(predicate) = predicate_between(text, a, b, predicates) {
            triples.push(ExtractedTriple {
                subject: a.text.clone(),
                predicate,
                object: b.text.clone(),
                confidence: HEURISTIC_CONFIDENCE,
            });
        }
    }
    triples
}

/// Return the canonical predicate for the text between `a.span.1` and
/// `b.span.0`, if any. Whole-phrase match first; falls back to
/// token-by-token to catch single-word verbs separated by stopwords.
fn predicate_between(
    text: &str,
    a: &Entity,
    b: &Entity,
    predicates: &PredicateTable,
) -> Option<String> {
    let start = a.span.1.min(text.len());
    let end = b.span.0.min(text.len());
    if start >= end {
        return None;
    }
    let between = &text[start..end];
    let trimmed = between.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(canonical) = canonicalize(predicates, trimmed) {
        return Some(canonical);
    }
    for tok in trimmed.split_whitespace() {
        let clean = tok.trim_matches(|c: char| !c.is_alphabetic());
        if clean.is_empty() {
            continue;
        }
        if let Some(canonical) = canonicalize(predicates, clean) {
            return Some(canonical);
        }
    }
    None
}
