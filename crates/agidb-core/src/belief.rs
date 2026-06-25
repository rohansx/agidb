//! Phase 9 — Beliefs (floor 6).
//!
//! First-class revisable beliefs with graded confidence, supporting and
//! contradicting evidence, an append-only revision log, and bi-temporal
//! withdrawal. Constitution article XVII: beliefs are revised, never
//! overwritten. Replaying the revision log reconstructs the current
//! confidence.
//!
//! Revision math (v0.1, pure — no LLM): a Bayesian-style update over
//! evidence count. Supporting evidence nudges confidence up toward 1.0;
//! contradicting evidence nudges it down. When confidence drops below
//! [`WITHDRAWAL_THRESHOLD`] the belief is withdrawn (valid-time closed
//! with `t_valid_end = now`), but the row and its full revision log are
//! preserved — the agent can always answer "what did I believe, and
//! what changed my mind?" The LLM-assisted contradiction-judgement hook
//! (constitution article IV amendment) lands behind a trait in a
//! follow-up; the read path stays LLM-free.

use chrono::Utc;
use redb::{ReadableTable, TableDefinition};

use crate::episode::tokenize;
use crate::error::{AgidbError, Result};
use crate::hdc::HV;
use crate::store::{decode, encode, Store, MANIFEST};
use crate::types::*;

/// Primary table — every [`Belief`] by id. Phase 9.
pub const BELIEFS: TableDefinition<u64, Vec<u8>> = TableDefinition::new("beliefs");

/// Append-only revision log — `(belief_id, seq) → BeliefRevision`.
/// Stored separately so `belief_history` is a cheap range scan without
/// deserializing the whole belief row.
pub const BELIEF_REVISIONS: TableDefinition<(u64, u64), Vec<u8>> =
    TableDefinition::new("belief_revisions");

/// Manifest key for the monotonic belief-id counter.
const KEY_NEXT_BELIEF_ID: &str = "next_belief_id";

/// Confidence floor below which a belief is withdrawn. Constitution
/// article XVII: withdrawal is non-destructive (valid-time closed, row
/// preserved). Configurable in v0.2; 0.5 is the spec default.
pub const WITHDRAWAL_THRESHOLD: f32 = 0.5;

/// How strongly each new piece of evidence moves the confidence. With
/// `ε = 0.15`, three supporting episodes take a 0.5 belief to ~0.79,
/// three contradicting episodes take it to ~0.21 → withdrawn.
const EVIDENCE_STEP: f32 = 0.15;

// ---------------------------------------------------------------------------
// HDC signature derivation.
// ---------------------------------------------------------------------------

/// Derive a belief's HDC signature from its claim text (token-bundle
/// scheme, same as gist episode signatures) so beliefs can be surfaced
/// by cue similarity in a future belief-context recall lane.
pub fn belief_signature(claim: &str) -> HV {
    let tokens = tokenize(claim);
    if tokens.is_empty() {
        return HV::from_name(claim);
    }
    let hvs: Vec<HV> = tokens.iter().map(|t| HV::from_name(t)).collect();
    HV::bundle(&hvs)
}

// ---------------------------------------------------------------------------
// Revision math.
// ---------------------------------------------------------------------------

/// One step of the Bayesian-style confidence update.
///
/// `supports = true` nudges confidence up toward 1.0; `false` nudges it
/// down toward 0.0. The step shrinks as confidence approaches the
/// bounds, so the value stays in `(0, 1)` and converges.
pub fn revise_confidence(current: f32, supports: bool) -> f32 {
    let step = EVIDENCE_STEP * (1.0 - 2.0 * current * (1.0 - current));
    let next = if supports {
        current + step
    } else {
        current - step
    };
    next.clamp(0.01, 0.99)
}

// ---------------------------------------------------------------------------
// Store API.
// ---------------------------------------------------------------------------

impl Store {
    /// Persist a new belief. Mints a `BeliefId`, derives the HDC
    /// signature, appends it to `signatures.dat`, and writes the row.
    /// The initial confidence is taken from `belief.confidence` (clamped
    /// to `[0, 1]`). Returns the new id.
    ///
    /// If a non-withdrawn belief with the same `(subject, predicate)`
    /// already exists, this is treated as a revision of that belief
    /// rather than a new assertion — see [`Self::revise_belief`].
    pub fn assert_belief(&mut self, mut belief: Belief) -> Result<BeliefId> {
        belief.confidence = belief.confidence.clamp(0.0, 1.0);
        belief.t_valid_start = Utc::now();
        belief.t_tx_start = Utc::now();

        // Dedup by (subject, predicate): if an active belief exists,
        // route through revise instead of creating a duplicate.
        if !belief.subject.is_empty() && !belief.predicate.is_empty() {
            if let Some(existing) = self.find_active_belief(&belief.subject, &belief.predicate)? {
                // Merge the incoming evidence into the existing belief.
                return self.merge_into_belief(existing, belief);
            }
        }

        let sig = belief_signature(&belief.claim);
        let offset = self.signatures.append(&sig)?;
        belief.signature_offset = offset;

        let id = self.next_belief_id()?;
        belief.id = id;

        let tx = self.db.begin_write()?;
        {
            let mut beliefs = tx.open_table(BELIEFS)?;
            beliefs.insert(id.raw(), encode(&belief)?)?;
        }
        tx.commit()?;
        let _ = self.record_event(crate::learning_log::LearningEvent::BeliefAsserted {
            id,
            claim: belief.claim.clone(),
            confidence: belief.confidence,
            at: Utc::now(),
        });
        Ok(id)
    }

    /// Revise a belief with new evidence. The evidence episode is
    /// classified as supporting or contradicting via the caller-supplied
    /// `supports` flag (the LLM-assisted auto-classification hook lands
    /// behind a trait in a follow-up). Confidence is updated with
    /// [`revise_confidence`], the evidence id is appended to the right
    /// list, and a `BeliefRevision` is appended to both the belief row
    /// and the separate `BELIEF_REVISIONS` table. If confidence drops
    /// below [`WITHDRAWAL_THRESHOLD`] the belief is withdrawn.
    pub fn revise_belief(
        &mut self,
        id: BeliefId,
        new_evidence: EpisodeId,
        supports: bool,
        reason: impl Into<String>,
    ) -> Result<RevisionReport> {
        let mut belief = self.get_belief(id)?.ok_or(AgidbError::UnknownBelief(id.raw()))?;
        if belief.is_withdrawn() {
            return Err(AgidbError::InvalidGoalTransition(format!(
                "belief {id} is withdrawn — no further revisions"
            )));
        }

        let previous = belief.confidence;
        let next = revise_confidence(previous, supports);
        belief.confidence = next;
        if supports {
            if !belief.evidence.contains(&new_evidence) {
                belief.evidence.push(new_evidence);
            }
        } else if !belief.contradictions.contains(&new_evidence) {
            belief.contradictions.push(new_evidence);
        }

        let revision = BeliefRevision {
            timestamp: Utc::now(),
            previous_confidence: previous,
            new_confidence: next,
            triggering_evidence: Some(new_evidence),
            reason: reason.into(),
        };
        belief.revision_log.push(revision.clone());

        let withdrawn = next < WITHDRAWAL_THRESHOLD;
        if withdrawn {
            belief.t_valid_end = Some(Utc::now());
        }

        // Persist the belief row + append the revision to the log table.
        let seq = belief.revision_log.len() as u64 - 1;
        let tx = self.db.begin_write()?;
        {
            let mut beliefs = tx.open_table(BELIEFS)?;
            beliefs.insert(id.raw(), encode(&belief)?)?;
            let mut revs = tx.open_table(BELIEF_REVISIONS)?;
            revs.insert((id.raw(), seq), encode(&revision)?)?;
        }
        tx.commit()?;

        let _ = self.record_event(crate::learning_log::LearningEvent::BeliefRevised {
            id,
            previous_confidence: previous,
            new_confidence: next,
            reason: revision.reason.clone(),
            at: Utc::now(),
        });

        Ok(RevisionReport {
            belief_id: id,
            previous_confidence: previous,
            new_confidence: next,
            triggering_evidence: Some(new_evidence),
            reason: revision.reason,
            withdrawn,
        })
    }

    /// Withdraw a belief explicitly with a reason (non-destructive:
    /// valid-time closed, row + revision log preserved). Constitution
    /// article XVII.
    pub fn withdraw_belief(&mut self, id: BeliefId, reason: impl Into<String>) -> Result<()> {
        let mut belief = self.get_belief(id)?.ok_or(AgidbError::UnknownBelief(id.raw()))?;
        if belief.is_withdrawn() {
            return Ok(()); // idempotent
        }
        let previous = belief.confidence;
        let revision = BeliefRevision {
            timestamp: Utc::now(),
            previous_confidence: previous,
            new_confidence: 0.0,
            triggering_evidence: None,
            reason: reason.into(),
        };
        belief.revision_log.push(revision.clone());
        belief.t_valid_end = Some(Utc::now());
        belief.confidence = 0.0;
        let seq = belief.revision_log.len() as u64 - 1;
        let tx = self.db.begin_write()?;
        {
            let mut beliefs = tx.open_table(BELIEFS)?;
            beliefs.insert(id.raw(), encode(&belief)?)?;
            let mut revs = tx.open_table(BELIEF_REVISIONS)?;
            revs.insert((id.raw(), seq), encode(&revision)?)?;
        }
        tx.commit()?;
        let _ = self.record_event(crate::learning_log::LearningEvent::BeliefWithdrawn {
            id,
            reason: revision.reason,
            at: Utc::now(),
        });
        Ok(())
    }

    /// Fetch a single belief by id (including its full revision log).
    pub fn get_belief(&self, id: BeliefId) -> Result<Option<Belief>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(BELIEFS)?;
        Ok(table
            .get(id.raw())?
            .map(|v| decode(&v.value()).expect("belief decode")))
    }

    /// Every non-withdrawn belief about `subject` (case-sensitive match
    /// against canonical concept names). O(N) over beliefs in v0.1; a
    /// subject index is a phase-9 follow-up.
    pub fn what_do_i_believe(&self, subject: &str) -> Result<Vec<Belief>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(BELIEFS)?;
        let mut out = Vec::new();
        for entry in table.iter()? {
            let (_, v) = entry?;
            let b: Belief = decode(&v.value())?;
            if b.is_withdrawn() || b.subject != subject {
                continue;
            }
            // Phase 11 — skip tombstoned beliefs.
            if self.is_belief_tombstoned(b.id)? {
                continue;
            }
            out.push(b);
        }
        Ok(out)
    }

    /// Every belief in id order (any state, including withdrawn).
    pub fn all_beliefs(&self) -> Result<Vec<Belief>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(BELIEFS)?;
        let mut out = Vec::new();
        for entry in table.iter()? {
            let (_, v) = entry?;
            out.push(decode(&v.value())?);
        }
        Ok(out)
    }

    /// The append-only revision log for a belief, in order. Replaying it
    /// reconstructs the current confidence (constitution article XVII).
    pub fn belief_history(&self, id: BeliefId) -> Result<Vec<BeliefRevision>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(BELIEF_REVISIONS)?;
        let mut out = Vec::new();
        let mut row = (id.raw(), 0u64);
        while let Some(v) = table.get(&row)? {
            out.push(decode(&v.value())?);
            row.1 += 1;
        }
        Ok(out)
    }

    /// Read a belief's stored HDC signature.
    pub fn belief_signature(&self, id: BeliefId) -> Result<Option<HV>> {
        match self.get_belief(id)? {
            Some(b) => Ok(Some(self.signatures.read(b.signature_offset)?)),
            None => Ok(None),
        }
    }

    // --- internal helpers ------------------------------------------------

    fn next_belief_id(&mut self) -> Result<BeliefId> {
        let tx = self.db.begin_write()?;
        let id;
        {
            let mut manifest = tx.open_table(MANIFEST)?;
            let raw = manifest.get(KEY_NEXT_BELIEF_ID)?.map(|v| v.value());
            let current: u64 = match raw {
                Some(bytes) => decode(&bytes)?,
                None => 1,
            };
            manifest.insert(KEY_NEXT_BELIEF_ID, encode(&(current + 1))?)?;
            id = BeliefId::new(current);
        }
        tx.commit()?;
        Ok(id)
    }

    /// Find the first non-withdrawn belief matching `(subject, predicate)`.
    fn find_active_belief(&self, subject: &str, predicate: &str) -> Result<Option<BeliefId>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(BELIEFS)?;
        for entry in table.iter()? {
            let (k, v) = entry?;
            let b: Belief = decode(&v.value())?;
            if !b.is_withdrawn() && b.subject == subject && b.predicate == predicate {
                return Ok(Some(BeliefId::new(k.value())));
            }
        }
        Ok(None)
    }

    /// Merge an incoming `assert_belief` call into an existing belief:
    /// fold the new evidence in, revise confidence up, append a revision.
    fn merge_into_belief(&mut self, id: BeliefId, incoming: Belief) -> Result<BeliefId> {
        let mut existing = self.get_belief(id)?.ok_or(AgidbError::UnknownBelief(id.raw()))?;
        let previous = existing.confidence;
        let mut next = previous;
        for ev in &incoming.evidence {
            if !existing.evidence.contains(ev) {
                existing.evidence.push(*ev);
                next = revise_confidence(next, true);
            }
        }
        existing.confidence = next;
        let revision = BeliefRevision {
            timestamp: Utc::now(),
            previous_confidence: previous,
            new_confidence: next,
            triggering_evidence: incoming.evidence.first().copied(),
            reason: format!("re-asserted with {} evidence episodes", incoming.evidence.len()),
        };
        existing.revision_log.push(revision.clone());
        let seq = existing.revision_log.len() as u64 - 1;
        let tx = self.db.begin_write()?;
        {
            let mut beliefs = tx.open_table(BELIEFS)?;
            beliefs.insert(id.raw(), encode(&existing)?)?;
            let mut revs = tx.open_table(BELIEF_REVISIONS)?;
            revs.insert((id.raw(), seq), encode(&revision)?)?;
        }
        tx.commit()?;
        Ok(id)
    }
}
