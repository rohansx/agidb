//! End-to-end integration: text → extract → store. Uses MockExtractor
//! so this test runs on every PR with zero model inference.

use agidb_core::store::{Store, StoreConfig};
use agidb_core::types::{
    Entity, ExtractContext, ExtractedTriple, Extraction, Provenance, TextExtractor,
};
use agidb_core::Result;
use agidb_extract::{observe_text, ObserveContext};
use chrono::{TimeZone, Utc};
use tempfile::TempDir;

struct MockExtractor;
impl TextExtractor for MockExtractor {
    fn extract(&self, _text: &str, _ctx: &ExtractContext) -> Result<Extraction> {
        Ok(Extraction {
            triples: vec![ExtractedTriple {
                subject: "Sarah".into(),
                predicate: "recommends".into(),
                object: "Bawri".into(),
                confidence: 0.91,
            }],
            valid_time: None,
            raw_entities: vec![
                Entity {
                    text: "Sarah".into(),
                    entity_type: "Person".into(),
                    span: (0, 5),
                    confidence: 0.93,
                    canonical_name: None,
                },
                Entity {
                    text: "Bawri".into(),
                    entity_type: "Place".into(),
                    span: (17, 22),
                    confidence: 0.88,
                    canonical_name: None,
                },
            ],
        })
    }
}

struct EmptyExtractor;
impl TextExtractor for EmptyExtractor {
    fn extract(&self, _text: &str, _ctx: &ExtractContext) -> Result<Extraction> {
        Ok(Extraction {
            triples: vec![],
            valid_time: None,
            raw_entities: vec![],
        })
    }
}

fn fresh_store() -> (Store, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let cfg = StoreConfig::at(dir.path());
    (Store::open(cfg).expect("open"), dir)
}

#[test]
fn observe_text_stores_episode_with_triple() {
    let (mut store, _d) = fresh_store();
    let extractor = MockExtractor;
    let ctx = ObserveContext {
        observation_time: Utc.with_ymd_and_hms(2026, 5, 23, 12, 0, 0).unwrap(),
        provenance: Provenance::default(),
    };

    let id = observe_text(&mut store, &extractor, "Sarah recommended Bawri", ctx)
        .expect("observe");

    let ep = store.get_episode(id).expect("get").expect("found");
    assert_eq!(ep.id, id);
    assert_eq!(ep.text, "Sarah recommended Bawri");
    assert_eq!(ep.triples.len(), 1);
    assert_eq!(ep.triples[0].predicate, "recommends");
    assert_eq!(ep.triples[0].subject, "Sarah");
    assert_eq!(ep.triples[0].object, "Bawri");
    assert_eq!(ep.triples[0].episode_id, id, "triple.episode_id points at its episode");
    assert!(ep.confidence > 0.8, "confidence from extracted triple, got {}", ep.confidence);
}

#[test]
fn observe_text_with_empty_extraction_still_stores_with_gist_signature() {
    let (mut store, _d) = fresh_store();
    let extractor = EmptyExtractor;
    let ctx = ObserveContext::default();

    let id = observe_text(&mut store, &extractor, "frobnicated", ctx).expect("observe");

    let ep = store.get_episode(id).expect("get").expect("found");
    assert_eq!(ep.text, "frobnicated");
    assert!(ep.triples.is_empty(), "no triples extracted");
    assert!((ep.confidence - 0.5).abs() < 1e-6, "neutral confidence for empty extraction");
}

#[test]
fn observe_text_pre_creates_concepts_with_ner_entity_types() {
    let (mut store, _d) = fresh_store();
    let extractor = MockExtractor;
    let ctx = ObserveContext::default();

    let _id = observe_text(&mut store, &extractor, "Sarah recommended Bawri", ctx)
        .expect("observe");

    // Alias resolver should have pre-created the concepts so observe's
    // auto-creation (with entity_type="unknown") doesn't run.
    let sarah_id = store
        .concept_id_for("Sarah")
        .expect("lookup")
        .expect("Sarah exists");
    let bawri_id = store
        .concept_id_for("Bawri")
        .expect("lookup")
        .expect("Bawri exists");
    assert_ne!(sarah_id, bawri_id);
}

#[test]
fn observe_text_two_calls_get_distinct_episode_ids() {
    let (mut store, _d) = fresh_store();
    let extractor = MockExtractor;
    let id1 = observe_text(&mut store, &extractor, "obs 1", ObserveContext::default())
        .expect("first");
    let id2 = observe_text(&mut store, &extractor, "obs 2", ObserveContext::default())
        .expect("second");
    assert_ne!(id1, id2);
    assert!(id2.raw() > id1.raw());
}
