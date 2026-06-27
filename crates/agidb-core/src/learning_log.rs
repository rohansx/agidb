//! Phase 10 (introspection subset) — the self-model audit log.
//!
//! Every introspectable state change emits a [`LearningEvent`] to an
//! append-only `learning_events` table. The agent can then ask
//! "what did I learn this week?" via [`Store::what_did_i_learn`].
//!
//! Constitution article XV (implication): `LearningEvent` is a **closed
//! enum** — new variants require an ADR. The variants below cover every
//! state-changing operation the engine ships today.
//!
//! The sensory ring buffer (phase 10 week 19-20) and attention-trace
//! recording (week 21) are deferred for the MVP; the events they would
//! emit are reserved as variants so the enum stays forward-compatible.

use chrono::{DateTime, Utc};
use redb::{ReadableTable, ReadableTableMetadata, TableDefinition};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::store::{decode, encode, Store, MANIFEST};
use crate::types::*;

/// Append-only audit trail of every introspectable state change.
pub const LEARNING_EVENTS: TableDefinition<u64, Vec<u8>> = TableDefinition::new("learning_events");

/// Manifest key for the monotonic learning-event-id counter.
const KEY_NEXT_LEARNING_EVENT_ID: &str = "next_learning_event_id";

/// One introspectable event. Closed enum (constitution article XV
/// implication) — new variants require an ADR.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LearningEvent {
    EpisodeStored {
        id: EpisodeId,
        at: DateTime<Utc>,
    },
    GoalSet {
        id: GoalId,
        description: String,
        at: DateTime<Utc>,
    },
    GoalStateChanged {
        id: GoalId,
        from: String,
        to: String,
        at: DateTime<Utc>,
    },
    BeliefAsserted {
        id: BeliefId,
        claim: String,
        confidence: f32,
        at: DateTime<Utc>,
    },
    BeliefRevised {
        id: BeliefId,
        previous_confidence: f32,
        new_confidence: f32,
        reason: String,
        at: DateTime<Utc>,
    },
    BeliefWithdrawn {
        id: BeliefId,
        reason: String,
        at: DateTime<Utc>,
    },
    SemanticAtomFormed {
        atom_id: SemanticAtomId,
        evidence_count: u32,
        at: DateTime<Utc>,
    },
    ContradictionDetected {
        count: u32,
        at: DateTime<Utc>,
    },
    ConsolidationRun {
        atoms_created: u32,
        contradictions: u32,
        at: DateTime<Utc>,
    },
    Unlearned {
        target: String,
        cascade_size: usize,
        self_vector_drift: u32,
        reason: String,
        at: DateTime<Utc>,
    },
    SelfVectorUpdated {
        drift_hamming: u32,
        at: DateTime<Utc>,
    },
}

impl LearningEvent {
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::EpisodeStored { at, .. }
            | Self::GoalSet { at, .. }
            | Self::GoalStateChanged { at, .. }
            | Self::BeliefAsserted { at, .. }
            | Self::BeliefRevised { at, .. }
            | Self::BeliefWithdrawn { at, .. }
            | Self::SemanticAtomFormed { at, .. }
            | Self::ContradictionDetected { at, .. }
            | Self::ConsolidationRun { at, .. }
            | Self::Unlearned { at, .. }
            | Self::SelfVectorUpdated { at, .. } => *at,
        }
    }

    /// A short human-readable label for the event kind.
    pub fn kind_label(&self) -> &'static str {
        match self {
            Self::EpisodeStored { .. } => "episode_stored",
            Self::GoalSet { .. } => "goal_set",
            Self::GoalStateChanged { .. } => "goal_state_changed",
            Self::BeliefAsserted { .. } => "belief_asserted",
            Self::BeliefRevised { .. } => "belief_revised",
            Self::BeliefWithdrawn { .. } => "belief_withdrawn",
            Self::SemanticAtomFormed { .. } => "semantic_atom_formed",
            Self::ContradictionDetected { .. } => "contradiction_detected",
            Self::ConsolidationRun { .. } => "consolidation_run",
            Self::Unlearned { .. } => "unlearned",
            Self::SelfVectorUpdated { .. } => "self_vector_updated",
        }
    }
}

impl Store {
    /// Append a learning event to the audit log. Called from every
    /// state-changing operation.
    pub fn record_event(&mut self, event: LearningEvent) -> Result<u64> {
        let id = self.next_learning_event_id()?;
        let tx = self.db.begin_write()?;
        {
            let mut table = tx.open_table(LEARNING_EVENTS)?;
            table.insert(id, encode(&event)?)?;
        }
        tx.commit()?;
        Ok(id)
    }

    /// Every event with `timestamp >= since`, in chronological order.
    /// Constitution article XV: the agent can always answer "what did I
    /// learn since <date>?"
    pub fn what_did_i_learn(&self, since: DateTime<Utc>) -> Result<Vec<LearningEvent>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(LEARNING_EVENTS)?;
        let mut out: Vec<LearningEvent> = Vec::new();
        for entry in table.iter()? {
            let (_, v) = entry?;
            let e: LearningEvent = decode(&v.value())?;
            if e.timestamp() >= since {
                out.push(e);
            }
        }
        out.sort_by_key(|e| e.timestamp());
        Ok(out)
    }

    /// Every event in chronological order (no time filter).
    pub fn all_learning_events(&self) -> Result<Vec<LearningEvent>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(LEARNING_EVENTS)?;
        let mut out: Vec<LearningEvent> = Vec::new();
        for entry in table.iter()? {
            let (_, v) = entry?;
            out.push(decode(&v.value())?);
        }
        out.sort_by_key(|e| e.timestamp());
        Ok(out)
    }

    /// Total count of learning events (for stats).
    pub fn learning_event_count(&self) -> Result<u64> {
        let tx = self.db.begin_read()?;
        Ok(tx.open_table(LEARNING_EVENTS)?.len()?)
    }

    fn next_learning_event_id(&mut self) -> Result<u64> {
        let tx = self.db.begin_write()?;
        let id;
        {
            let mut manifest = tx.open_table(MANIFEST)?;
            let raw = manifest.get(KEY_NEXT_LEARNING_EVENT_ID)?.map(|v| v.value());
            let current: u64 = match raw {
                Some(bytes) => decode(&bytes)?,
                None => 1,
            };
            manifest.insert(KEY_NEXT_LEARNING_EVENT_ID, encode(&(current + 1))?)?;
            id = current;
        }
        tx.commit()?;
        Ok(id)
    }
}
