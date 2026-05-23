//! Heuristic relation extractor — pure-logic tests.

use agidb_core::types::Entity;
use agidb_extract::heuristic_relations::extract_heuristic_relations;
use agidb_extract::predicates::PredicateTable;

fn entity(text: &str, ty: &str, span: (usize, usize)) -> Entity {
    Entity {
        text: text.into(),
        entity_type: ty.into(),
        span,
        confidence: 0.9,
        canonical_name: None,
    }
}

#[test]
fn detects_recommends_between_two_entities() {
    let t = PredicateTable::default();
    let text = "Sarah recommended Bawri";
    let ents = vec![
        entity("Sarah", "Person", (0, 5)),
        entity("Bawri", "Place", (18, 23)),
    ];
    let triples = extract_heuristic_relations(text, &ents, &t);
    assert_eq!(triples.len(), 1);
    assert_eq!(triples[0].subject, "Sarah");
    assert_eq!(triples[0].predicate, "recommends");
    assert_eq!(triples[0].object, "Bawri");
    assert!((triples[0].confidence - 0.5).abs() < 1e-6);
}

#[test]
fn matches_multi_word_predicate_phrase() {
    let t = PredicateTable::default();
    let text = "Sarah told me about Bawri";
    let ents = vec![
        entity("Sarah", "Person", (0, 5)),
        entity("Bawri", "Place", (20, 25)),
    ];
    let triples = extract_heuristic_relations(text, &ents, &t);
    assert_eq!(triples.len(), 1);
    assert_eq!(triples[0].predicate, "recommends");
}

#[test]
fn handles_three_entities_two_pairs() {
    let t = PredicateTable::default();
    let text = "Sarah met Alice visited Bawri";
    let ents = vec![
        entity("Sarah", "Person", (0, 5)),
        entity("Alice", "Person", (10, 15)),
        entity("Bawri", "Place", (24, 29)),
    ];
    let triples = extract_heuristic_relations(text, &ents, &t);
    assert_eq!(triples.len(), 2);
    assert_eq!(triples[0].predicate, "met");
    assert_eq!(triples[1].predicate, "visited");
}

#[test]
fn no_known_verb_means_no_triple() {
    let t = PredicateTable::default();
    let text = "Sarah frobnicated Bawri";
    let ents = vec![
        entity("Sarah", "Person", (0, 5)),
        entity("Bawri", "Place", (18, 23)),
    ];
    let triples = extract_heuristic_relations(text, &ents, &t);
    assert!(triples.is_empty(), "got {triples:?}");
}

#[test]
fn handles_zero_or_one_entity() {
    let t = PredicateTable::default();
    assert!(extract_heuristic_relations("hello", &[], &t).is_empty());
    let ents = vec![entity("Sarah", "Person", (0, 5))];
    assert!(extract_heuristic_relations("Sarah said hi", &ents, &t).is_empty());
}

#[test]
fn handles_entities_with_no_text_between() {
    let t = PredicateTable::default();
    // entities touching — start of b equals end of a
    let text = "SarahAlice";
    let ents = vec![
        entity("Sarah", "Person", (0, 5)),
        entity("Alice", "Person", (5, 10)),
    ];
    let triples = extract_heuristic_relations(text, &ents, &t);
    assert!(triples.is_empty(), "no text between → no triple");
}
