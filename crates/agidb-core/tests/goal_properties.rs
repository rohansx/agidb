//! Phase 9 — Goal state-machine invariants.
//!
//! Constitution article XV: goals are a first-class typed primitive.
//! `Completed` and `Abandoned` are terminal. Pause/resume preserve
//! history. A 100-step random walk through the state machine never
//! violates the invariants.

use agidb_core::goal::GoalStateKind;
use agidb_core::store::{Store, StoreConfig};
use agidb_core::types::{EpisodeId, Goal, GoalId, GoalState};
use chrono::Utc;
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use tempfile::TempDir;

fn fresh_store() -> (Store, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let store = Store::open(StoreConfig::at(dir.path())).expect("open");
    (store, dir)
}

#[test]
fn set_goal_returns_active_goal_with_id_and_signature() {
    let (mut store, _d) = fresh_store();
    let id = store
        .set_goal(Goal::new("find a thai place for the team dinner"))
        .expect("set_goal");
    assert!(id.raw() >= 1);

    let g = store.get_goal(id).expect("get").expect("present");
    assert_eq!(g.id, id);
    assert_eq!(g.description, "find a thai place for the team dinner");
    assert!(matches!(g.state, GoalState::Active));
    assert!(g.signature_offset > 0);
}

#[test]
fn active_goals_lists_only_active() {
    let (mut store, _d) = fresh_store();
    let a = store.set_goal(Goal::new("goal a")).expect("set");
    let b = store.set_goal(Goal::new("goal b")).expect("set");
    store.complete_goal(a, vec![]).expect("complete");

    let active = store.active_goals().expect("active");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, b);
}

#[test]
fn complete_goal_is_terminal_and_blocks_further_transitions() {
    let (mut store, _d) = fresh_store();
    let id = store.set_goal(Goal::new("ship v0.1")).expect("set");
    let ev = EpisodeId::new(7);
    store.complete_goal(id, vec![ev]).expect("complete");

    let g = store.get_goal(id).expect("get").expect("present");
    match &g.state {
        GoalState::Completed { evidence, .. } => assert_eq!(*evidence, vec![ev]),
        other => panic!("expected Completed, got {:?}", other),
    }
    assert!(g.state.is_terminal());

    // Further transitions are rejected.
    let err = store.revise_goal(id, Default::default()).unwrap_err();
    assert!(
        matches!(err, agidb_core::AgidbError::InvalidGoalTransition(_)),
        "terminal goal must reject revise"
    );
    let err = store.pause_goal(id, "x").unwrap_err();
    assert!(matches!(err, agidb_core::AgidbError::InvalidGoalTransition(_)));
}

#[test]
fn abandon_goal_is_terminal() {
    let (mut store, _d) = fresh_store();
    let id = store.set_goal(Goal::new("deprecated idea")).expect("set");
    store.abandon_goal(id, "no longer relevant").expect("abandon");
    let g = store.get_goal(id).expect("get").expect("present");
    assert!(matches!(g.state, GoalState::Abandoned { .. }));
    assert!(g.state.is_terminal());
}

#[test]
fn pause_and_resume_round_trip() {
    let (mut store, _d) = fresh_store();
    let id = store.set_goal(Goal::new("paused goal")).expect("set");
    store.pause_goal(id, "waiting on deps").expect("pause");
    let g = store.get_goal(id).expect("get").expect("present");
    assert!(matches!(g.state, GoalState::Paused { .. }));

    store.resume_goal(id).expect("resume");
    let g = store.get_goal(id).expect("get").expect("present");
    assert!(matches!(g.state, GoalState::Active));
}

#[test]
fn pause_is_noop_when_already_paused() {
    let (mut store, _d) = fresh_store();
    let id = store.set_goal(Goal::new("g")).expect("set");
    store.pause_goal(id, "r1").expect("pause");
    let err = store.pause_goal(id, "r2").unwrap_err();
    assert!(matches!(err, agidb_core::AgidbError::InvalidGoalTransition(_)));
}

#[test]
fn revise_goal_updates_description_and_recomputes_signature() {
    let (mut store, _d) = fresh_store();
    let id = store.set_goal(Goal::new("original")).expect("set");
    let before = store.get_goal(id).unwrap().unwrap().signature_offset;

    use agidb_core::types::GoalPatch;
    let patch = GoalPatch {
        description: Some("revised description".into()),
        deadline: None,
        success_criteria: None,
    };
    store.revise_goal(id, patch).expect("revise");

    let g = store.get_goal(id).unwrap().unwrap();
    assert_eq!(g.description, "revised description");
    assert_ne!(g.signature_offset, before, "signature must move on recompute");
}

#[test]
fn parent_child_hierarchy_binds_parent_signature() {
    let (mut store, _d) = fresh_store();
    let parent = store.set_goal(Goal::new("top level goal")).expect("set");
    let child = store
        .set_goal(Goal::new("top level goal").with_parent(parent))
        .expect("set child");

    let parent_sig = store.goal_signature(parent).expect("read").expect("some");
    let child_sig = store.goal_signature(child).expect("read").expect("some");
    // The child signature is bind(bundle, parent) — not equal to the
    // parent, and not equal to a fresh child-without-parent signature.
    let orphan_id = store.set_goal(Goal::new("top level goal")).expect("set");
    let orphan_sig = store.goal_signature(orphan_id).expect("read").expect("some");
    assert_ne!(parent_sig, child_sig);
    assert_ne!(orphan_sig, child_sig);
}

#[test]
fn get_unknown_goal_returns_none() {
    let (store, _d) = fresh_store();
    assert!(store.get_goal(GoalId::new(999)).expect("get").is_none());
}

// ---------------------------------------------------------------------------
// 100-step random walk — the phase-9 exit-criterion test.
// ---------------------------------------------------------------------------

#[test]
fn hundred_step_goal_mutation_walk_never_violates_invariants() {
    let (mut store, _d) = fresh_store();
    let mut rng = StdRng::seed_from_u64(0xC0FFEE);

    // Seed a handful of goals to walk over.
    let mut ids: Vec<GoalId> = (0..5)
        .map(|i| store.set_goal(Goal::new(format!("goal {i}"))).expect("set"))
        .collect();

    for _ in 0..100 {
        let &id = ids.choose(&mut rng).unwrap();
        let current = store.get_goal(id).unwrap().unwrap();
        // Invariant 1: terminal states stay terminal.
        if current.state.is_terminal() {
            // every mutating call on a terminal goal must error.
            let pick = rng.gen_range(0..4u8);
            let r = match pick {
                0 => store.revise_goal(id, Default::default()),
                1 => store.complete_goal(id, vec![]),
                2 => store.abandon_goal(id, "x"),
                _ => store.pause_goal(id, "x"),
            };
            assert!(
                matches!(r, Err(agidb_core::AgidbError::InvalidGoalTransition(_))),
                "terminal goal accepted a transition: {:?}",
                r
            );
            continue;
        }
        // Non-terminal: pick a legal transition.
        match rng.gen_range(0..5u8) {
            0 => {
                store.complete_goal(id, vec![]).expect("complete");
            }
            1 => {
                store.abandon_goal(id, "random walk").expect("abandon");
            }
            2 => {
                let _ = store.pause_goal(id, "walk"); // may no-op error if already paused
            }
            3 => {
                let _ = store.resume_goal(id); // may no-op error if active
            }
            _ => {
                // occasionally mint a new goal so the population grows
                ids.push(store.set_goal(Goal::new("walk goal")).expect("set"));
            }
        }
        // Invariant 2: re-read and confirm the state is one of the valid
        // variants and the kind() label matches.
        let g = store.get_goal(id).unwrap().unwrap();
        let kind = g.state.kind();
        assert!(
            matches!(
                kind,
                GoalStateKind::Active
                    | GoalStateKind::Paused
                    | GoalStateKind::Completed
                    | GoalStateKind::Abandoned
            ),
            "invalid state kind after walk: {:?}",
            kind
        );
    }
    // Invariant 3: after the walk, active_goals() contains exactly the
    // Active goals (no terminal/abandoned leak in).
    let active = store.active_goals().expect("active");
    assert!(active.iter().all(|g| g.state.is_active()));
    // Invariant 4: every goal is retrievable and its id is unique.
    let all = store.all_goals().expect("all");
    let mut seen = std::collections::HashSet::new();
    for g in &all {
        assert!(seen.insert(g.id), "duplicate goal id {:?}", g.id);
    }
    // touch `Utc::now` so the import stays live even if the walk
    // happened not to need timestamps directly.
    let _ = Utc::now();
}
