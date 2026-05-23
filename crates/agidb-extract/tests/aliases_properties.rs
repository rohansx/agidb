//! Alias resolver: exact (case-folded) → existing ConceptId.
//! Else Levenshtein <= 3 against canonical names; unique match wins.
//! Else create a new Concept via Store::create_concept with the
//! NER-derived entity_type.

use agidb_core::store::{Store, StoreConfig};
use agidb_extract::aliases::AliasResolver;
use tempfile::TempDir;

fn fresh_store() -> (Store, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let cfg = StoreConfig::at(dir.path());
    (Store::open(cfg).expect("open"), dir)
}

#[test]
fn exact_match_returns_existing_id() {
    let (mut store, _d) = fresh_store();
    let original = store.create_concept("Sarah", "Person").unwrap();
    let resolver = AliasResolver::new();
    let resolved = resolver
        .resolve(&mut store, "Sarah", "Person")
        .expect("resolve");
    assert_eq!(resolved, original);
}

#[test]
fn case_insensitive_match() {
    let (mut store, _d) = fresh_store();
    let original = store.create_concept("Bawri", "Place").unwrap();
    let resolver = AliasResolver::new();
    let resolved = resolver
        .resolve(&mut store, "bawri", "Place")
        .expect("resolve");
    assert_eq!(resolved, original);
}

#[test]
fn levenshtein_one_typo_matches() {
    let (mut store, _d) = fresh_store();
    let original = store.create_concept("Bandra", "Place").unwrap();
    let resolver = AliasResolver::new();
    // "Bandar" vs "Bandra": distance 2 (swap a/r → still <= 3)
    let resolved = resolver
        .resolve(&mut store, "Bandar", "Place")
        .expect("resolve");
    assert_eq!(resolved, original, "single typo should match");
}

#[test]
fn no_match_creates_new_concept() {
    let (mut store, _d) = fresh_store();
    let resolver = AliasResolver::new();
    let new_id = resolver
        .resolve(&mut store, "Quetzalcoatl", "Person")
        .expect("resolve");
    let again = resolver
        .resolve(&mut store, "Quetzalcoatl", "Person")
        .expect("resolve");
    assert_eq!(new_id, again, "second resolve hits the row we just created");
}

#[test]
fn ambiguous_fuzzy_match_creates_new_rather_than_picking_one() {
    let (mut store, _d) = fresh_store();
    let alice = store.create_concept("Alice", "Person").unwrap();
    let allie = store.create_concept("Allie", "Person").unwrap();
    let resolver = AliasResolver::new();
    // "Alise" is within distance 2 of both — ambiguous → don't merge.
    let resolved = resolver
        .resolve(&mut store, "Alise", "Person")
        .expect("resolve");
    assert_ne!(resolved, alice);
    assert_ne!(resolved, allie);
}
