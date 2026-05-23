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
//! Algorithm:
//!   1. Split the text into sentences (on `.` / `?` / `!`).
//!   2. For each adjacent entity pair (sorted by span), skip pairs that
//!      cross a sentence boundary.
//!   3. Inspect the text *between* the two entities; skip if it has more
//!      than [`MAX_BETWEEN_WORDS`] words (likely too far apart to be a
//!      real relation).
//!   4. Try the whole between-phrase against the predicate table first
//!      (catches multi-word predicates like "told me about"); else
//!      token-by-token. First match wins; emit a triple at
//!      `confidence = 0.5`.
//!
//! No ML. No model. No network. Just `PredicateTable` lookup.

use agidb_core::types::{Entity, ExtractedTriple};

use crate::predicates::{canonicalize, PredicateTable};

/// Conservative confidence for heuristic-derived relations.
const HEURISTIC_CONFIDENCE: f32 = 0.5;

/// Cap on how many words may sit between two entities for a relation to
/// be considered. Real binary relations sit close together; long spans
/// almost always cross some narrative boundary.
const MAX_BETWEEN_WORDS: usize = 8;

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

    let sentences = split_sentences(text);

    // Sort by span start so the windows() walk goes in text order.
    let mut sorted: Vec<&Entity> = entities.iter().collect();
    sorted.sort_by_key(|e| e.span.0);

    let mut triples = Vec::new();
    for window in sorted.windows(2) {
        let a = window[0];
        let b = window[1];

        // Skip pairs that cross a sentence boundary — relations rarely do.
        let a_sent = sentence_id(&sentences, a.span.0);
        let b_sent = sentence_id(&sentences, b.span.0);
        if a_sent != b_sent {
            continue;
        }

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
/// `b.span.0`, if any.
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
    if trimmed.split_whitespace().count() > MAX_BETWEEN_WORDS {
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

/// Sentence-split on `.`, `?`, `!`. The returned ranges are
/// `[start, end)` byte offsets into `text`, where `end` includes the
/// terminator. If there are no terminators, returns a single range
/// covering the whole text.
fn split_sentences(text: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut sentences = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if matches!(bytes[i], b'.' | b'!' | b'?') {
            sentences.push((start, i + 1));
            // Skip whitespace between sentences.
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            start = j;
            i = j;
        } else {
            i += 1;
        }
    }
    if start < bytes.len() {
        sentences.push((start, bytes.len()));
    }
    if sentences.is_empty() {
        sentences.push((0, text.len()));
    }
    sentences
}

/// Return the index of the sentence containing byte offset `pos`, or
/// `None` if `pos` falls outside any sentence range.
fn sentence_id(sentences: &[(usize, usize)], pos: usize) -> Option<usize> {
    sentences.iter().position(|&(s, e)| pos >= s && pos < e)
}
