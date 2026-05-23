//! Store::next_episode_id mints fresh, monotonically increasing ids.

use agidb_core::store::{Store, StoreConfig};
use tempfile::TempDir;

fn fresh_store() -> (Store, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let cfg = StoreConfig::at(dir.path());
    (Store::open(cfg).expect("open"), dir)
}

#[test]
fn next_episode_id_starts_at_one() {
    let (mut store, _d) = fresh_store();
    let a = store.next_episode_id().expect("first");
    assert_eq!(a.raw(), 1);
}

#[test]
fn next_episode_id_is_monotonic() {
    let (mut store, _d) = fresh_store();
    let a = store.next_episode_id().expect("a");
    let b = store.next_episode_id().expect("b");
    let c = store.next_episode_id().expect("c");
    assert!(a.raw() < b.raw());
    assert!(b.raw() < c.raw());
    assert_eq!(b.raw(), a.raw() + 1);
    assert_eq!(c.raw(), b.raw() + 1);
}

#[test]
fn next_episode_id_persists_across_reopen() {
    let dir = TempDir::new().expect("tempdir");
    let cfg = StoreConfig::at(dir.path());

    let a = {
        let mut store = Store::open(cfg.clone()).expect("open 1");
        store.next_episode_id().expect("a")
    };

    let b = {
        let mut store = Store::open(cfg).expect("open 2");
        store.next_episode_id().expect("b")
    };

    assert_eq!(b.raw(), a.raw() + 1, "counter survives reopen");
}
