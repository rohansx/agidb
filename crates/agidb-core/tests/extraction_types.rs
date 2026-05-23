//! Smoke tests for the layer-2-facing types added in phase 3.
//! Real extraction is tested in `agidb-extract`; this file just pins
//! the shape of the `agidb-core` surface that `agidb-extract` depends on.

use agidb_core::types::{Entity, ExtractContext, ExtractedTriple, Extraction, TextExtractor};
use agidb_core::Result;
use chrono::Utc;

struct DummyExtractor;
impl TextExtractor for DummyExtractor {
    fn extract(&self, _text: &str, _ctx: &ExtractContext) -> Result<Extraction> {
        Ok(Extraction {
            triples: vec![],
            valid_time: None,
            raw_entities: vec![],
        })
    }
}

#[test]
fn extract_context_carries_observation_time() {
    let now = Utc::now();
    let ctx = ExtractContext {
        observation_time: now,
        relation_hint_types: vec![],
    };
    assert_eq!(ctx.observation_time, now);
    assert!(ctx.relation_hint_types.is_empty());
}

#[test]
fn entity_carries_optional_canonical_name() {
    let e = Entity {
        text: "Sarah".into(),
        entity_type: "Person".into(),
        span: (0, 5),
        confidence: 0.93,
        canonical_name: None,
    };
    assert!(e.canonical_name.is_none());
    assert_eq!(e.span, (0, 5));
    assert_eq!(e.entity_type, "Person");
}

#[test]
fn extracted_triple_has_no_episode_id() {
    // ExtractedTriple is the layer-2 shape — no EpisodeId yet, because
    // extraction happens BEFORE the episode is minted in observe_text.
    let t = ExtractedTriple {
        subject: "Sarah".into(),
        predicate: "recommends".into(),
        object: "Bawri".into(),
        confidence: 0.91,
    };
    assert_eq!(t.predicate, "recommends");
    assert!((t.confidence - 0.91).abs() < 1e-6);
}

#[test]
fn extraction_holds_triples_and_optional_time() {
    let ex = Extraction {
        triples: vec![ExtractedTriple {
            subject: "Bawri".into(),
            predicate: "located_in".into(),
            object: "Bandra".into(),
            confidence: 0.83,
        }],
        valid_time: None,
        raw_entities: vec![],
    };
    assert_eq!(ex.triples.len(), 1);
    assert!(ex.valid_time.is_none());
}

#[test]
fn dummy_extractor_satisfies_trait() {
    let ext = DummyExtractor;
    let ctx = ExtractContext {
        observation_time: Utc::now(),
        relation_hint_types: vec![],
    };
    let r = ext.extract("hello", &ctx).expect("dummy never fails");
    assert!(r.triples.is_empty());
    assert!(r.valid_time.is_none());
}
