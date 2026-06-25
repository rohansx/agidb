//! Integration test for the `Agidb` facade: observe → recall →
//! consolidate → stats round-trip on a temp store with the null
//! extractor (deterministic, no network).

use agidb::{Agidb, AgidbConfig, ExtractorSetup, Tier};

#[tokio::test]
async fn observe_recall_consolidate_round_trip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = AgidbConfig::new(dir.path()).with_extractor(ExtractorSetup::Null);
    let db = Agidb::open_with(cfg).await.expect("open");

    // text-only episodes still get gist signatures → tier C/D recall.
    let id1 = db.observe("Sarah recommended Bawri in Bandra").await.unwrap();
    let _id2 = db.observe("Sarah said Bawri is a thai place").await.unwrap();
    assert_eq!(id1.raw(), 1);

    // constitution article VI: recall never returns the empty set.
    let r = db.recall_cue("what thai place did sarah mention?").await.unwrap();
    assert!(!r.matches.is_empty(), "recall must never return empty");
    // elapsed_ms is a u32 wall-clock measurement.
    let _ = r.elapsed_ms;

    // consolidate is idempotent and safe on a tiny store.
    let c = db.consolidate().await.unwrap();
    assert_eq!(c.episodes_scanned, 2);

    // stats reflect what was written.
    let s = db.stats().await.unwrap();
    assert_eq!(s.episodes, 2);
    assert_eq!(s.signatures, 2);
    assert_eq!(s.consolidation_passes, 1);
}

#[tokio::test]
async fn get_episode_and_list_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = AgidbConfig::new(dir.path()).with_extractor(ExtractorSetup::Null);
    let db = Agidb::open_with(cfg).await.unwrap();

    db.observe("alice likes rust").await.unwrap();
    db.observe("bob likes rust").await.unwrap();

    let got = db.get_episode(1).await.unwrap().expect("ep1 exists");
    assert_eq!(got.text, "alice likes rust");

    let listed = db.list_episodes(10).await.unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].id.raw(), 1);
}

#[tokio::test]
async fn export_import_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = AgidbConfig::new(dir.path()).with_extractor(ExtractorSetup::Null);
    let db = Agidb::open_with(cfg).await.unwrap();
    db.observe("a fact about the world").await.unwrap();
    db.observe("another fact").await.unwrap();

    let path = dir.path().join("dump.jsonl");
    db.export_jsonl(&path).await.unwrap();

    let dir2 = tempfile::tempdir().unwrap();
    let cfg2 = AgidbConfig::new(dir2.path()).with_extractor(ExtractorSetup::Null);
    let db2 = Agidb::open_with(cfg2).await.unwrap();
    let n = db2.import_jsonl(&path).await.unwrap();
    assert_eq!(n, 2);
    let s = db2.stats().await.unwrap();
    assert_eq!(s.episodes, 2);
}

#[tokio::test]
async fn tier_floor_caps_the_cascade() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = AgidbConfig::new(dir.path()).with_extractor(ExtractorSetup::Null);
    let db = Agidb::open_with(cfg).await.unwrap();
    db.observe("something unrelated to the cue").await.unwrap();

    use agidb::Query;
    let q = Query::cue("zzz no match").with_tier_floor(Tier::Gist);
    let r = db.recall(q).await.unwrap();
    // floor at Gist forbids the NearestNeighbor fallback; with no gist
    // match, the cascade returns no matches (the never-empty guarantee
    // only holds under the default NearestNeighbor floor).
    assert!(r.matches.is_empty() || r.tier_used.depth() <= Tier::Gist.depth());
}
