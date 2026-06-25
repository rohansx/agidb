//! Phase 9 — Goals (floor 6).
//!
//! First-class typed goal storage with a validated state machine,
//! optional parent-child hierarchy, HDC signatures for goal-biased
//! retrieval, and append-only provenance. Constitution article XV:
//! goals are a substrate primitive, not text inside an episode.
//!
//! State machine (article XV):
//! ```text
//!     Active  ⇄  Paused
//!        │
//!        ├──→ Completed   (terminal)
//!        └──→ Abandoned   (terminal)
//! ```
//! `Completed` and `Abandoned` accept no further transitions.

use chrono::Utc;
use redb::{ReadableTable, TableDefinition};

use crate::episode::tokenize;
use crate::error::{AgidbError, Result};
use crate::hdc::HV;
use crate::store::{decode, encode, Store, MANIFEST};
use crate::types::*;

/// Primary table — every [`Goal`] by id. Phase 9.
pub const GOALS: TableDefinition<u64, Vec<u8>> = TableDefinition::new("goals");

/// Manifest key for the monotonic goal-id counter.
const KEY_NEXT_GOAL_ID: &str = "next_goal_id";

/// A coarse label for a [`GoalState`], used in error messages and
/// `LearningEvent` payloads without carrying the full payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoalStateKind {
    Active,
    Paused,
    Completed,
    Abandoned,
}

impl GoalState {
    pub fn kind(&self) -> GoalStateKind {
        match self {
            GoalState::Active => GoalStateKind::Active,
            GoalState::Paused { .. } => GoalStateKind::Paused,
            GoalState::Completed { .. } => GoalStateKind::Completed,
            GoalState::Abandoned { .. } => GoalStateKind::Abandoned,
        }
    }
}

// ---------------------------------------------------------------------------
// HDC signature derivation.
// ---------------------------------------------------------------------------

/// Derive a goal's HDC signature from its description. Uses the same
/// token-bundle scheme as gist episode signatures so goal-biased
/// retrieval is a plain `similarity(episode_sig, goal_sig)` call.
///
/// If the goal has a parent, the parent's signature is bound in so
/// subgoals cluster near their parent in HV space.
pub fn goal_signature(description: &str, parent_sig: Option<&HV>) -> HV {
    let tokens = tokenize(description);
    if tokens.is_empty() {
        return HV::from_name(description);
    }
    let hvs: Vec<HV> = tokens.iter().map(|t| HV::from_name(t)).collect();
    let bundle = HV::bundle(&hvs);
    match parent_sig {
        Some(parent) => bundle.bind(parent),
        None => bundle,
    }
}

// ---------------------------------------------------------------------------
// Store API.
// ---------------------------------------------------------------------------

impl Store {
    /// Persist a new goal. Mints a `GoalId`, derives the HDC signature
    /// (binding the parent's signature when `parent_id` is set and the
    /// parent exists), appends it to `signatures.dat`, and writes the
    /// row to the `goals` table. Returns the new id.
    pub fn set_goal(&mut self, mut goal: Goal) -> Result<GoalId> {
        let parent_sig = match goal.parent_id {
            Some(pid) => self
                .get_goal(pid)?
                .and_then(|p| self.signatures.read(p.signature_offset).ok()),
            None => None,
        };
        let sig = goal_signature(&goal.description, parent_sig.as_ref());
        let offset = self.signatures.append(&sig)?;
        goal.signature_offset = offset;

        let now = Utc::now();
        goal.created_at = now;
        goal.updated_at = now;

        let id = self.next_goal_id()?;
        goal.id = id;

        let tx = self.db.begin_write()?;
        {
            let mut goals = tx.open_table(GOALS)?;
            goals.insert(id.raw(), encode(&goal)?)?;
        }
        tx.commit()?;
        let _ = self.record_event(crate::learning_log::LearningEvent::GoalSet {
            id,
            description: goal.description.clone(),
            at: Utc::now(),
        });
        Ok(id)
    }

    /// Apply a partial update to a goal. `description` changes recompute
    /// the HDC signature (parent context preserved). State transitions
    /// are validated; terminal goals reject every patch.
    pub fn revise_goal(&mut self, id: GoalId, patch: GoalPatch) -> Result<()> {
        let mut goal = self.get_goal(id)?.ok_or(AgidbError::UnknownGoal(id.raw()))?;
        if goal.state.is_terminal() {
            return Err(AgidbError::InvalidGoalTransition(format!(
                "goal {} is in terminal state {:?} — no further transitions allowed",
                id,
                goal.state.kind()
            )));
        }

        let mut recompute_sig = false;
        if let Some(desc) = patch.description {
            if desc != goal.description {
                goal.description = desc;
                recompute_sig = true;
            }
        }
        if let Some(deadline) = patch.deadline {
            goal.deadline = deadline;
        }
        if let Some(criteria) = patch.success_criteria {
            goal.success_criteria = criteria;
        }

        if recompute_sig {
            let parent_sig = match goal.parent_id {
                Some(pid) => self
                    .get_goal(pid)?
                    .and_then(|p| self.signatures.read(p.signature_offset).ok()),
                None => None,
            };
            let sig = goal_signature(&goal.description, parent_sig.as_ref());
            // Append a fresh signature rather than rewriting in place —
            // the old slot is orphaned but never read again. Compaction
            // (phase 6 follow-up) reclaims it.
            goal.signature_offset = self.signatures.append(&sig)?;
        }
        goal.updated_at = Utc::now();

        let tx = self.db.begin_write()?;
        {
            let mut goals = tx.open_table(GOALS)?;
            goals.insert(id.raw(), encode(&goal)?)?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Transition a goal to `Completed`, attaching the evidence episodes
    /// that satisfied it. Terminal after this call.
    pub fn complete_goal(&mut self, id: GoalId, evidence: Vec<EpisodeId>) -> Result<()> {
        let mut goal = self.get_goal(id)?.ok_or(AgidbError::UnknownGoal(id.raw()))?;
        if goal.state.is_terminal() {
            return Err(AgidbError::InvalidGoalTransition(format!(
                "goal {} is in terminal state {:?} — no further transitions allowed",
                id,
                goal.state.kind()
            )));
        }
        let from = format!("{:?}", goal.state.kind());
        goal.state = GoalState::Completed {
            at: Utc::now(),
            evidence,
        };
        goal.updated_at = Utc::now();
        let tx = self.db.begin_write()?;
        {
            let mut goals = tx.open_table(GOALS)?;
            goals.insert(id.raw(), encode(&goal)?)?;
        }
        tx.commit()?;
        let _ = self.record_event(crate::learning_log::LearningEvent::GoalStateChanged {
            id,
            from,
            to: "Completed".into(),
            at: Utc::now(),
        });
        Ok(())
    }

    /// Transition a goal to `Abandoned` with a reason. Terminal.
    pub fn abandon_goal(&mut self, id: GoalId, reason: impl Into<String>) -> Result<()> {
        let mut goal = self.get_goal(id)?.ok_or(AgidbError::UnknownGoal(id.raw()))?;
        if goal.state.is_terminal() {
            return Err(AgidbError::InvalidGoalTransition(format!(
                "goal {} is in terminal state {:?} — no further transitions allowed",
                id,
                goal.state.kind()
            )));
        }
        let from = format!("{:?}", goal.state.kind());
        goal.state = GoalState::Abandoned {
            at: Utc::now(),
            reason: reason.into(),
        };
        goal.updated_at = Utc::now();
        let tx = self.db.begin_write()?;
        {
            let mut goals = tx.open_table(GOALS)?;
            goals.insert(id.raw(), encode(&goal)?)?;
        }
        tx.commit()?;
        let _ = self.record_event(crate::learning_log::LearningEvent::GoalStateChanged {
            id,
            from,
            to: "Abandoned".into(),
            at: Utc::now(),
        });
        Ok(())
    }

    /// Pause an active goal with a reason.
    pub fn pause_goal(&mut self, id: GoalId, reason: impl Into<String>) -> Result<()> {
        let mut goal = self.get_goal(id)?.ok_or(AgidbError::UnknownGoal(id.raw()))?;
        if goal.state.is_terminal() {
            return Err(AgidbError::InvalidGoalTransition(format!(
                "goal {} is in terminal state {:?} — no further transitions allowed",
                id,
                goal.state.kind()
            )));
        }
        if matches!(goal.state, GoalState::Paused { .. }) {
            return Err(AgidbError::InvalidGoalTransition(format!(
                "goal {} already paused — transition is a no-op",
                id
            )));
        }
        let from = format!("{:?}", goal.state.kind());
        goal.state = GoalState::Paused {
            since: Utc::now(),
            reason: reason.into(),
        };
        goal.updated_at = Utc::now();
        let tx = self.db.begin_write()?;
        {
            let mut goals = tx.open_table(GOALS)?;
            goals.insert(id.raw(), encode(&goal)?)?;
        }
        tx.commit()?;
        let _ = self.record_event(crate::learning_log::LearningEvent::GoalStateChanged {
            id,
            from,
            to: "Paused".into(),
            at: Utc::now(),
        });
        Ok(())
    }

    /// Resume a paused goal back to `Active`.
    pub fn resume_goal(&mut self, id: GoalId) -> Result<()> {
        let mut goal = self.get_goal(id)?.ok_or(AgidbError::UnknownGoal(id.raw()))?;
        if goal.state.is_terminal() {
            return Err(AgidbError::InvalidGoalTransition(format!(
                "goal {} is in terminal state {:?} — no further transitions allowed",
                id,
                goal.state.kind()
            )));
        }
        if matches!(goal.state, GoalState::Active) {
            return Err(AgidbError::InvalidGoalTransition(format!(
                "goal {} already active — transition is a no-op",
                id
            )));
        }
        let from = format!("{:?}", goal.state.kind());
        goal.state = GoalState::Active;
        goal.updated_at = Utc::now();
        let tx = self.db.begin_write()?;
        {
            let mut goals = tx.open_table(GOALS)?;
            goals.insert(id.raw(), encode(&goal)?)?;
        }
        tx.commit()?;
        let _ = self.record_event(crate::learning_log::LearningEvent::GoalStateChanged {
            id,
            from,
            to: "Active".into(),
            at: Utc::now(),
        });
        Ok(())
    }

    /// Fetch a single goal by id.
    pub fn get_goal(&self, id: GoalId) -> Result<Option<Goal>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(GOALS)?;
        Ok(table
            .get(id.raw())?
            .map(|v| decode(&v.value()).expect("goal decode")))
    }

    /// Every goal in `Active` state, in id order. These are the goals
    /// that bias recall.
    pub fn active_goals(&self) -> Result<Vec<Goal>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(GOALS)?;
        let mut out = Vec::new();
        for entry in table.iter()? {
            let (_, v) = entry?;
            let g: Goal = decode(&v.value())?;
            if g.state.is_active() {
                out.push(g);
            }
        }
        Ok(out)
    }

    /// Every goal in id order (any state).
    pub fn all_goals(&self) -> Result<Vec<Goal>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(GOALS)?;
        let mut out = Vec::new();
        for entry in table.iter()? {
            let (_, v) = entry?;
            out.push(decode(&v.value())?);
        }
        Ok(out)
    }

    /// Read a goal's stored HDC signature.
    pub fn goal_signature(&self, id: GoalId) -> Result<Option<HV>> {
        match self.get_goal(id)? {
            Some(g) => Ok(Some(self.signatures.read(g.signature_offset)?)),
            None => Ok(None),
        }
    }

    // --- internal helpers ------------------------------------------------

    /// Monotonic goal-id counter, persisted in the manifest.
    fn next_goal_id(&mut self) -> Result<GoalId> {
        let tx = self.db.begin_write()?;
        let id;
        {
            let mut manifest = tx.open_table(MANIFEST)?;
            let raw = manifest.get(KEY_NEXT_GOAL_ID)?.map(|v| v.value());
            let current: u64 = match raw {
                Some(bytes) => decode(&bytes)?,
                None => 1,
            };
            manifest.insert(KEY_NEXT_GOAL_ID, encode(&(current + 1))?)?;
            id = GoalId::new(current);
        }
        tx.commit()?;
        Ok(id)
    }
}
