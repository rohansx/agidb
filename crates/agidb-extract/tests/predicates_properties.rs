//! Surface verb → canonical predicate mapping.

use agidb_extract::predicates::{canonicalize, PredicateTable};

fn table() -> PredicateTable {
    PredicateTable::default()
}

#[test]
fn exact_match_recommends() {
    let t = table();
    assert_eq!(canonicalize(&t, "recommended"), Some("recommends".into()));
    assert_eq!(canonicalize(&t, "Recommended"), Some("recommends".into()));
    assert_eq!(canonicalize(&t, "suggested"), Some("recommends".into()));
    assert_eq!(canonicalize(&t, "told me about"), Some("recommends".into()));
}

#[test]
fn located_in_family() {
    let t = table();
    assert_eq!(canonicalize(&t, "in"), Some("located_in".into()));
    assert_eq!(canonicalize(&t, "based in"), Some("located_in".into()));
    assert_eq!(canonicalize(&t, "lives in"), Some("located_in".into()));
}

#[test]
fn works_at_family() {
    let t = table();
    assert_eq!(canonicalize(&t, "works at"), Some("works_at".into()));
    assert_eq!(canonicalize(&t, "is employed by"), Some("works_at".into()));
}

#[test]
fn unknown_returns_none() {
    let t = table();
    assert_eq!(canonicalize(&t, "frobnicated"), None);
    assert_eq!(canonicalize(&t, ""), None);
}

#[test]
fn custom_synonyms_extend_defaults() {
    let mut t = table();
    t.add_synonym("frobnicates", "frobnicated");
    t.add_synonym("frobnicates", "twiddled the knobs on");
    assert_eq!(canonicalize(&t, "frobnicated"), Some("frobnicates".into()));
    assert_eq!(
        canonicalize(&t, "twiddled the knobs on"),
        Some("frobnicates".into())
    );
}

#[test]
fn case_insensitive_lookup() {
    let t = table();
    assert_eq!(canonicalize(&t, "RECOMMENDED"), Some("recommends".into()));
    assert_eq!(canonicalize(&t, "Works At"), Some("works_at".into()));
}

#[test]
fn expanded_vocabulary_v1() {
    let t = table();
    // A sampling across the v1 polish additions; full table in predicates.rs.
    assert_eq!(canonicalize(&t, "chose"), Some("chose".into()));
    assert_eq!(canonicalize(&t, "selected"), Some("chose".into()));
    assert_eq!(canonicalize(&t, "opted for"), Some("chose".into()));
    assert_eq!(canonicalize(&t, "avoided"), Some("avoided".into()));
    assert_eq!(canonicalize(&t, "introduced"), Some("introduced".into()));
    assert_eq!(canonicalize(&t, "arrived at"), Some("arrived_at".into()));
    assert_eq!(canonicalize(&t, "wrote"), Some("wrote".into()));
    assert_eq!(canonicalize(&t, "discussed"), Some("discussed".into()));
    assert_eq!(canonicalize(&t, "bought"), Some("bought".into()));
    assert_eq!(canonicalize(&t, "founded"), Some("founded".into()));
    assert_eq!(canonicalize(&t, "knows"), Some("knows".into()));
    assert_eq!(canonicalize(&t, "married"), Some("married".into()));
    assert_eq!(canonicalize(&t, "hired"), Some("hired".into()));
    assert_eq!(canonicalize(&t, "thanked"), Some("thanked".into()));
}
