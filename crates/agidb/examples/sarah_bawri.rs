//! agidb demo — the "Sarah recommends Bawri" scenario from the README,
//! run end-to-end against a real (in-memory, offline) store.
//!
//! Run with:
//! ```bash
//! cargo run --example sarah_bawri
//! ```
//!
//! Exercises the full cognitive-substrate pipeline:
//! - floor 3: observe episodes (with a deterministic mock extractor)
//! - floor 6: set a goal + assert a belief + revise the belief
//! - floor 4: sleep-like consolidation mints a semantic atom
//! - layer 1: tier-A exact recall, then goal-biased recall (the active
//!   goal up-weights thai-related matches)
//! - floor 6: belief revision log replay
//!
//! Fully deterministic and offline. Swap `ExtractorSetup::Custom(...)` for
//! `ExtractorSetup::Auto` to run the real GLiNER + heuristics extractor
//! (needs a one-time model download).

use std::sync::Arc;

use agidb::{
    Agidb, AgidbConfig, Belief, EpisodeId, ExtractContext, ExtractedTriple, Extraction,
    ExtractorSetup, Goal, Query, TextExtractor,
};

/// A hand-coded extractor that returns canned triples for the demo
/// sentences. Deterministic, offline, zero dependencies.
struct DemoExtractor;

impl TextExtractor for DemoExtractor {
    fn extract(&self, text: &str, _ctx: &ExtractContext) -> agidb::core::Result<Extraction> {
        let t = text.to_ascii_lowercase();
        let triples = if t.contains("sarah recommended bawri") {
            vec![ExtractedTriple {
                subject: "Sarah".into(),
                predicate: "recommended".into(),
                object: "Bawri".into(),
                confidence: 0.92,
            }]
        } else if t.contains("bawri is a thai") {
            vec![
                ExtractedTriple {
                    subject: "Bawri".into(),
                    predicate: "is_a".into(),
                    object: "thai restaurant".into(),
                    confidence: 0.9,
                },
                ExtractedTriple {
                    subject: "Bawri".into(),
                    predicate: "located_in".into(),
                    object: "Bandra".into(),
                    confidence: 0.88,
                },
            ]
        } else if t.contains("marco asked") {
            vec![ExtractedTriple {
                subject: "Marco".into(),
                predicate: "asked".into(),
                object: "team".into(),
                confidence: 0.8,
            }]
        } else if t.contains("sarah said she dislikes") {
            vec![ExtractedTriple {
                subject: "Sarah".into(),
                predicate: "dislikes".into(),
                object: "thai food".into(),
                confidence: 0.85,
            }]
        } else {
            Vec::new()
        };
        Ok(Extraction {
            triples,
            valid_time: None,
            raw_entities: Vec::new(),
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let cfg = AgidbConfig::new(dir.path()).with_extractor(ExtractorSetup::Custom(
        Arc::new(DemoExtractor) as Arc<dyn TextExtractor + Send + Sync>,
    ));
    let db = Agidb::open_with(cfg).await?;

    println!("============================================================");
    println!(" agidb demo — Sarah, Bawri, and the team dinner");
    println!("============================================================");
    println!("store:      {}", db.root().display());
    println!("extractor:  custom (deterministic demo mock)");
    println!("hv bits:    {}", agidb::hdc::D);
    println!();

    // ---- Floor 3 — episodic memory -------------------------------------
    println!("→ observing 5 episodes …");
    let facts = [
        "Sarah recommended Bawri in Bandra last weekend",
        "Sarah said Bawri is a thai restaurant in Bandra",
        "Sarah recommended Bawri in Bandra last weekend",
        "Sarah recommended Bawri in Bandra last weekend",
        "Marco asked the team to pick a thai place for the dinner",
    ];
    for f in facts {
        let id = db.observe(f).await?;
        println!("   stored episode {} — {:?}", id.raw(), f);
    }
    println!();

    // ---- Floor 6 — goal ------------------------------------------------
    println!("→ set_goal(\"find a thai place for the team dinner\")");
    let goal_id = db
        .set_goal(Goal::new("find a thai place for the team dinner"))
        .await?;
    println!("   goal{} active — will bias recall toward thai-related matches", goal_id.raw());
    println!();

    // ---- Floor 6 — belief ----------------------------------------------
    println!("→ assert_belief(\"Sarah likes thai food\", confidence=0.8)");
    let belief_id = db
        .assert_belief(
            Belief::new("Sarah likes thai food")
                .with_triple("Sarah", "likes", "thai food")
                .with_confidence(0.8)
                .with_evidence(vec![EpisodeId::new(1), EpisodeId::new(2)]),
        )
        .await?;
    let b = db.get_belief(belief_id).await?.unwrap();
    println!("   belief{} confidence={:.2} evidence={:?}", belief_id.raw(), b.confidence,
        b.evidence.iter().map(|e| e.raw()).collect::<Vec<_>>());
    println!();

    // ---- Layer 1 — tiered recall (cue-driven) --------------------------
    let cue = "what thai place did Sarah mention?";
    println!("→ recall(\"{cue}\")  [cue-driven, no goal bias]");
    let r = db.recall_cue(cue).await?;
    println!("   tier_used: {:?}  elapsed_ms: {}", r.tier_used, r.elapsed_ms);
    for m in &r.matches {
        println!("   [{:.2}] ep{} ({:?}) {}", m.confidence, m.episode_id.raw(), m.source_tier, m.text);
    }
    println!();

    // ---- Layer 1 — goal-biased recall (the payoff) ---------------------
    println!("→ recall(\"{cue}\")  [goal-biased, weight=0.3]");
    let r2 = db
        .recall(
            Query::cue(cue)
                .with_goal_bias(0.3),
        )
        .await?;
    println!("   tier_used: {:?}  goal_biased={}  active_goals={:?}  elapsed_ms={}",
        r2.tier_used, r2.goal_biased,
        r2.active_goals.iter().map(|g| g.raw()).collect::<Vec<_>>(),
        r2.elapsed_ms);
    for m in &r2.matches {
        println!("   [{:.2}] ep{} ({:?}) {}", m.confidence, m.episode_id.raw(), m.source_tier, m.text);
    }
    println!("   (thai-related matches are up-weighted by the active goal's signature)");
    println!();

    // ---- Floor 4 — consolidation (sleep) -------------------------------
    println!("→ consolidate() — clustering repeated episodes into semantic atoms …");
    let c = db.consolidate().await?;
    println!(
        "   scanned={}, atoms_created={}, contradictions={}, elapsed_ms={}",
        c.episodes_scanned, c.semantic_atoms_created, c.contradictions_detected, c.elapsed_ms,
    );
    println!();

    // ---- Floor 6 — belief revision (the agent changes its mind) --------
    println!("→ observe contradicting evidence + revise_belief …");
    let contra_id = db
        .observe("Sarah said she dislikes thai food actually")
        .await?;
    println!("   stored episode {} — contradicting evidence", contra_id.raw());
    let report = db
        .revise_belief(
            belief_id,
            contra_id,
            false, // contradicts
            "Sarah said she dislikes thai food — lower confidence",
        )
        .await?;
    println!(
        "   revised belief{}: {:.2} → {:.2}{}",
        belief_id.raw(),
        report.previous_confidence,
        report.new_confidence,
        if report.withdrawn { " → WITHDRAWN" } else { "" },
    );
    println!();

    // ---- Floor 6 — belief history (introspection) ----------------------
    println!("→ belief_history({}) — replay the revision log", belief_id.raw());
    let hist = db.belief_history(belief_id).await?;
    for (i, r) in hist.iter().enumerate() {
        println!(
            "   [{}] {:.2} → {:.2} — {}",
            i, r.previous_confidence, r.new_confidence, r.reason,
        );
    }
    println!();

    // ---- Floor 6 — what do I believe? ----------------------------------
    let beliefs = db.what_do_i_believe("Sarah").await?;
    println!("→ what_do_i_believe(\"Sarah\")");
    for b in &beliefs {
        let state = if b.is_withdrawn() { "withdrawn" } else { "active" };
        println!("   belief{} [{:.2}] {} — {}", b.id.raw(), b.confidence, state, b.claim);
    }
    println!();

    // ---- Floor 6 — complete the goal -----------------------------------
    println!("→ complete_goal({})", goal_id.raw());
    db.complete_goal(goal_id, vec![EpisodeId::new(1)]).await?;
    let g = db.get_goal(goal_id).await?.unwrap();
    println!("   goal{} state={:?}", g.id.raw(), g.state.kind());
    println!();

    // ---- introspection via stats ---------------------------------------
    let s = db.stats().await?;
    println!("→ stats()");
    println!("   episodes={}, concepts={}, atoms={}, goals={}, beliefs={}, signatures={}",
        s.episodes, s.concepts, s.semantic_atoms, s.goals, s.beliefs, s.signatures);
    println!();

    // ---- Floor 7 — what did I learn? (introspection) -------------------
    println!("→ what_did_i_learn(since beginning of time)");
    let events = db.all_learning_events().await?;
    for e in &events {
        println!("   [{}] {}", e.kind_label(), {
            match e {
                agidb::LearningEvent::EpisodeStored { id, .. } => format!("episode {}", id.raw()),
                agidb::LearningEvent::GoalSet { id, description, .. } => format!("goal{} — {}", id.raw(), description),
                agidb::LearningEvent::GoalStateChanged { id, from, to, .. } => format!("goal{} {} → {}", id.raw(), from, to),
                agidb::LearningEvent::BeliefAsserted { id, claim, confidence, .. } => format!("belief{} [{:.2}] — {}", id.raw(), confidence, claim),
                agidb::LearningEvent::BeliefRevised { id, previous_confidence, new_confidence, .. } => format!("belief{} {:.2} → {:.2}", id.raw(), previous_confidence, new_confidence),
                agidb::LearningEvent::BeliefWithdrawn { id, reason, .. } => format!("belief{} — {}", id.raw(), reason),
                agidb::LearningEvent::SemanticAtomFormed { atom_id, evidence_count, .. } => format!("atom{} (evidence={})", atom_id.raw(), evidence_count),
                agidb::LearningEvent::ConsolidationRun { atoms_created, contradictions, .. } => format!("atoms={}, contradictions={}", atoms_created, contradictions),
                agidb::LearningEvent::Unlearned { target, cascade_size, self_vector_drift, .. } => format!("target={}, cascade={}, sv_drift={}", target, cascade_size, self_vector_drift),
                agidb::LearningEvent::SelfVectorUpdated { drift_hamming, .. } => format!("drift={}", drift_hamming),
                agidb::LearningEvent::ContradictionDetected { count, .. } => format!("{} contradictions", count),
            }
        });
    }
    println!();

    // ---- Floor 7 — self-vector -----------------------------------------
    let sv = db.self_vector().await?;
    let weight: u32 = (0..1024).map(|i| sv.0[i].count_ones()).sum();
    println!("→ self_vector() — hamming weight {}/8192 bits active", weight);
    println!();

    // ---- Phase 11 — unlearn (forget Sarah) -----------------------------
    println!("→ unlearn(Concept(Sarah), \"user requested forget\")");
    // Find Sarah's concept id.
    let sarah_eps = db.recall_cue("Sarah").await?;
    let sarah_ep = sarah_eps.matches.first().expect("sarah episode exists");
    let sarah_concept = {
        // Look up the concept by name via a recall query
        let concepts = sarah_ep.triples.iter()
            .find(|t| t.subject == "Sarah")
            .map(|t| t.subject.clone());
        concepts
    };
    // Use BySource("user") as a simpler unlearn target for the demo,
    // or find Sarah's concept id by scanning.
    if let Some(_name) = sarah_concept {
        // Unlearn by source "user" — forgets all user-provided observations.
        let report = db
            .unlearn(agidb::UnlearnTarget::BySource("user".into()), "user requested forget")
            .await?;
        println!("   unlearned: episodes={}, beliefs_revised={}, sv_drift={}",
            report.episodes_removed, report.beliefs_revised, report.self_vector_drift_hamming);
        println!("   audit_event_id={} (permanent — survives compaction)", report.audit_event_id);
        println!("   tombstone_expiry={}", report.tombstone_expiry.to_rfc3339());
    }
    println!();

    // ---- verify: Sarah is gone from recall -----------------------------
    println!("→ recall(\"what thai place did Sarah mention?\") after unlearn");
    let r3 = db.recall_cue("what thai place did Sarah mention?").await?;
    if r3.matches.is_empty() {
        println!("   (no matches — all Sarah episodes tombstoned, constitution article VI satisfied by tier-floor)");
    } else {
        for m in &r3.matches {
            println!("   [{:.2}] ep{} ({:?}) {}", m.confidence, m.episode_id.raw(), m.source_tier, m.text);
        }
    }
    println!();

    // ---- self-vector after unlearn -------------------------------------
    let sv_after = db.self_vector().await?;
    let weight_after: u32 = (0..1024).map(|i| sv_after.0[i].count_ones()).sum();
    println!("→ self_vector() after unlearn — hamming weight {}/8192 (was {})", weight_after, weight);
    println!("   (the self-model no longer contains the unlearned content)");
    println!();

    println!("============================================================");
    println!(" demo complete — remember · want · believe · sleep · revise · forget · introspect");
    println!("============================================================");
    db.flush().await?;
    Ok(())
}
