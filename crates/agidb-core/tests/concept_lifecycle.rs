//! Store::create_concept assigns a fresh ConceptId, persists the row,
//! and is idempotent on canonical name.

use agidb_core::store::{Store, StoreConfig};
use tempfile::TempDir;

fn fresh_store() -> (Store, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let cfg = StoreConfig::at(dir.path());
    let store = Store::open(cfg).expect("open");
    (store, dir)
}

#[test]
fn create_concept_assigns_id_and_persists() {
    let (mut store, _dir) = fresh_store();
    let id = store.create_concept("Sarah", "Person").expect("create");
    let looked_up = store
        .concept_id_for("Sarah")
        .expect("lookup")
        .expect("found");
    assert_eq!(id, looked_up);
    assert!(id.raw() > 0, "ids start at 1");
}

#[test]
fn create_concept_is_idempotent_on_canonical_name() {
    let (mut store, _dir) = fresh_store();
    let first = store.create_concept("Bawri", "Place").expect("first");
    let second = store.create_concept("Bawri", "Place").expect("second");
    assert_eq!(first, second, "second create returns the same id");
}

#[test]
fn create_concept_distinct_names_get_distinct_ids() {
    let (mut store, _dir) = fresh_store();
    let a = store.create_concept("Sarah", "Person").expect("a");
    let b = store.create_concept("Bawri", "Place").expect("b");
    assert_ne!(a, b);
}
