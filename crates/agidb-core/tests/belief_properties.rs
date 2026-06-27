//! Phase 9 — Belief revision invariants.
//!
//! Constitution article XVII: beliefs are revised, never overwritten.
//! The append-only revision log captures every change; replaying it
//! reconstructs the current confidence. Withdrawal is non-destructive
//! (valid-time closed, row preserved).

use agidb_core::belief::{revise_confidence, WITHDRAWAL_THRESHOLD};
use agidb_core::store::{Store, StoreConfig};
use agidb_core::types::{Belief, BeliefId, EpisodeId, Provenance};
use chrono::Utc;
use tempfile::TempDir;

fn fresh_store() -> (Store, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let store = Store::open(StoreConfig::at(dir.path())).expect("open");
    (store, dir)
}

fn ep(n: u64) -> EpisodeId {
    EpisodeId::new(n)
}

#[test]
fn assert_belief_persists_with_id_and_signature() {
    let (mut store, _d) = fresh_store();
    let id = store
        .assert_belief(
            Belief::new("Bawri is a thai restaurant")
                .with_confidence(0.8)
                .with_triple("Bawri", "is_a", "thai restaurant")
                .with_evidence(vec![ep(1), ep(2)]),
        )
        .expect("assert");

    let b = store.get_belief(id).expect("get").expect("present");
    assert_eq!(b.id, id);
    assert_eq!(b.claim, "Bawri is a thai restaurant");
    assert_eq!(b.subject, "Bawri");
    assert!((b.confidence - 0.8).abs() < 1e-6);
    assert_eq!(b.evidence, vec![ep(1), ep(2)]);
    assert!(b.signature_offset > 0);
    assert!(!b.is_withdrawn());
}

#[test]
fn revise_with_supporting_evidence_raises_confidence_and_logs() {
    let (mut store, _d) = fresh_store();
    let id = store
        .assert_belief(
            Belief::new("Sarah likes thai food")
                .with_triple("Sarah", "likes", "thai food")
                .with_confidence(0.5),
        )
        .expect("assert");
    let before = store.get_belief(id).unwrap().unwrap().confidence;

    let report = store
        .revise_belief(id, ep(10), true, "sarah picked a thai place again")
        .expect("revise");
    assert!(report.new_confidence > before);
    assert!(!report.withdrawn);

    let b = store.get_belief(id).unwrap().unwrap();
    assert!(b.evidence.contains(&ep(10)));
    assert_eq!(b.revision_log.len(), 1);
    assert!((b.revision_log[0].previous_confidence - before).abs() < 1e-6);
}

#[test]
fn revise_with_contradicting_evidence_lowers_confidence_and_logs() {
    let (mut store, _d) = fresh_store();
    let id = store
        .assert_belief(
            Belief::new("Marco likes rust")
                .with_triple("Marco", "likes", "rust")
                .with_confidence(0.7),
        )
        .expect("assert");

    let report = store
        .revise_belief(id, ep(20), false, "marco said he prefers go")
        .expect("revise");
    assert!(report.new_confidence < 0.7);
    let b = store.get_belief(id).unwrap().unwrap();
    assert!(b.contradictions.contains(&ep(20)));
    assert_eq!(b.revision_log.len(), 1);
}

#[test]
fn confidence_drop_below_threshold_withdraws_belief() {
    let (mut store, _d) = fresh_store();
    let id = store
        .assert_belief(
            Belief::new("X is true")
                .with_triple("X", "is", "true")
                .with_confidence(0.5),
        )
        .expect("assert");
    // Hammer with contradictions until it withdraws.
    let mut withdrawn = false;
    for i in 1..=20 {
        let r = store
            .revise_belief(id, ep(i), false, "contradiction")
            .expect("revise");
        if r.withdrawn {
            withdrawn = true;
            break;
        }
    }
    assert!(
        withdrawn,
        "belief should withdraw under enough contradiction"
    );

    let b = store.get_belief(id).unwrap().unwrap();
    assert!(b.is_withdrawn(), "valid-time must be closed on withdraw");
    assert!(b.confidence < WITHDRAWAL_THRESHOLD);
    // Further revisions on a withdrawn belief are rejected.
    let err = store
        .revise_belief(id, ep(99), true, "late support")
        .unwrap_err();
    assert!(matches!(
        err,
        agidb_core::AgidbError::InvalidGoalTransition(_)
    ));
}

#[test]
fn belief_history_replays_to_current_confidence() {
    let (mut store, _d) = fresh_store();
    let id = store
        .assert_belief(
            Belief::new("Hypothesis H")
                .with_triple("H", "is", "tested")
                .with_confidence(0.5),
        )
        .expect("assert");
    store
        .revise_belief(id, ep(1), true, "support 1")
        .expect("r");
    store
        .revise_belief(id, ep(2), true, "support 2")
        .expect("r");
    store
        .revise_belief(id, ep(3), false, "contradiction 1")
        .expect("r");

    let history = store.belief_history(id).expect("history");
    assert_eq!(history.len(), 3, "every revision must be logged");
    // Replay: start at the initial confidence and apply each revision.
    let replayed = history.iter().fold(0.5f32, |acc, r| {
        assert!(
            (r.previous_confidence - acc).abs() < 1e-5,
            "log chain broken"
        );
        r.new_confidence
    });
    let current = store.get_belief(id).unwrap().unwrap().confidence;
    assert!(
        (replayed - current).abs() < 1e-5,
        "replay must match current"
    );
}

#[test]
fn what_do_i_believe_filters_by_subject_and_excludes_withdrawn() {
    let (mut store, _d) = fresh_store();
    let a = store
        .assert_belief(
            Belief::new("Sarah likes thai")
                .with_triple("Sarah", "likes", "thai")
                .with_confidence(0.8),
        )
        .expect("assert");
    store
        .assert_belief(
            Belief::new("Sarah lives in Bandra")
                .with_triple("Sarah", "lives_in", "Bandra")
                .with_confidence(0.7),
        )
        .expect("assert");
    store
        .assert_belief(
            Belief::new("Marco likes go")
                .with_triple("Marco", "likes", "go")
                .with_confidence(0.6),
        )
        .expect("assert");

    let about_sarah = store.what_do_i_believe("Sarah").expect("beliefs");
    assert_eq!(about_sarah.len(), 2);
    assert!(about_sarah.iter().all(|b| b.subject == "Sarah"));

    // Withdraw one of Sarah's beliefs; it must drop out of the query.
    store
        .withdraw_belief(a, "no longer confident")
        .expect("withdraw");
    let about_sarah = store.what_do_i_believe("Sarah").expect("beliefs");
    assert_eq!(about_sarah.len(), 1, "withdrawn belief must be excluded");
}

#[test]
fn withdraw_is_idempotent_and_non_destructive() {
    let (mut store, _d) = fresh_store();
    let id = store
        .assert_belief(
            Belief::new("temp hypothesis")
                .with_triple("T", "is", "temp")
                .with_confidence(0.6),
        )
        .expect("assert");
    store.withdraw_belief(id, "first reason").expect("withdraw");
    let after_first = store.get_belief(id).unwrap().unwrap();
    assert!(after_first.is_withdrawn());
    let log_len = after_first.revision_log.len();

    // Second withdraw is a no-op (idempotent), no new revision logged.
    store
        .withdraw_belief(id, "second reason")
        .expect("withdraw");
    let after_second = store.get_belief(id).unwrap().unwrap();
    assert_eq!(after_second.revision_log.len(), log_len);
    // Row preserved — not deleted.
    assert!(store.get_belief(id).unwrap().is_some());
}

#[test]
fn re_asserting_same_subject_predicate_revises_existing_belief() {
    let (mut store, _d) = fresh_store();
    let first = store
        .assert_belief(
            Belief::new("Bawri is good")
                .with_triple("Bawri", "is", "good")
                .with_confidence(0.5)
                .with_evidence(vec![ep(1)]),
        )
        .expect("assert");
    let second = store
        .assert_belief(
            Belief::new("Bawri is good")
                .with_triple("Bawri", "is", "good")
                .with_confidence(0.6)
                .with_evidence(vec![ep(2)]),
        )
        .expect("assert");
    // Same (subject, predicate) → dedup merges into the existing belief.
    assert_eq!(first, second, "re-assertion must revise, not duplicate");
    let b = store.get_belief(first).unwrap().unwrap();
    assert!(b.evidence.contains(&ep(1)));
    assert!(b.evidence.contains(&ep(2)));
    assert!(!b.revision_log.is_empty(), "merge must append a revision");
}

#[test]
fn revise_confidence_stays_in_open_unit_interval() {
    let mut c = 0.5f32;
    for _ in 0..1000 {
        c = revise_confidence(c, true);
        assert!(c > 0.0 && c < 1.0, "support overflow: {c}");
    }
    let mut c = 0.5f32;
    for _ in 0..1000 {
        c = revise_confidence(c, false);
        assert!(c > 0.0 && c < 1.0, "contradiction overflow: {c}");
    }
}

#[test]
fn get_unknown_belief_returns_none() {
    let (store, _d) = fresh_store();
    assert!(store.get_belief(BeliefId::new(999)).expect("get").is_none());
}

#[test]
fn belief_carries_provenance() {
    let (mut store, _d) = fresh_store();
    let prov = Provenance {
        source: "tool:slack".into(),
        session_id: Some("s1".into()),
        trace_id: None,
        metadata: std::collections::BTreeMap::new(),
    };
    let id = store
        .assert_belief(
            Belief::new("standup claim")
                .with_triple("Team", "has", "standup")
                .with_provenance(prov),
        )
        .expect("assert");
    let b = store.get_belief(id).unwrap().unwrap();
    assert_eq!(b.provenance.source, "tool:slack");
    assert_eq!(b.provenance.session_id.as_deref(), Some("s1"));
    // touch Utc to keep the import live.
    let _ = Utc::now();
}
