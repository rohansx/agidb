//! Domain types stored by layer 3 and exposed through the public API.
//!
//! Every term here corresponds to a glossary entry in `CONTEXT.md`. Drift
//! between this file and the glossary is a signal for `/grill-with-docs`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Identifiers
// ---------------------------------------------------------------------------

macro_rules! id_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(
            Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
        )]
        #[repr(transparent)]
        pub struct $name(pub u64);

        impl $name {
            pub const fn new(raw: u64) -> Self {
                Self(raw)
            }

            pub const fn raw(self) -> u64 {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

id_newtype!(
    /// Identifier for an [`Episode`].
    EpisodeId
);

id_newtype!(
    /// Identifier for a [`Concept`].
    ConceptId
);

id_newtype!(
    /// Identifier for a [`SemanticAtom`].
    SemanticAtomId
);

id_newtype!(
    /// Identifier for a [`Triple`] row.
    TripleId
);

// ---------------------------------------------------------------------------
// Time & provenance
// ---------------------------------------------------------------------------

/// Closed-open valid-time interval `[start, end)`. `end = None` means
/// "currently valid"; the consolidation loop sets it when a fact is
/// superseded.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}

impl TimeRange {
    pub fn point(t: DateTime<Utc>) -> Self {
        Self { start: t, end: None }
    }

    /// `true` iff `at` is inside `[start, end)`.
    pub fn contains(&self, at: DateTime<Utc>) -> bool {
        at >= self.start && self.end.is_none_or(|e| at < e)
    }
}

/// Attribution for a write. Every Episode and SemanticAtom traces back
/// to a Provenance record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub source: String,
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
    /// Free-form bag for caller-defined annotations. Kept as
    /// `BTreeMap` for deterministic serialization.
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

impl Default for Provenance {
    fn default() -> Self {
        Self {
            source: "unknown".into(),
            session_id: None,
            trace_id: None,
            metadata: BTreeMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Triples, episodes, concepts, semantic atoms
// ---------------------------------------------------------------------------

/// A `(subject, predicate, object)` tuple extracted by layer 2. Each
/// triple carries its own confidence score and a back-reference to the
/// source episode.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Triple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f32,
    pub episode_id: EpisodeId,
}

/// A single stored observation. The fundamental unit of episodic memory.
///
/// Bi-temporal: `valid_time` says when the fact was true in the world,
/// `t_tx_start` says when agidb learned it. `superseded_by` is set by
/// the consolidation loop when a later contradicting fact replaces this
/// one — the row is preserved, never deleted (constitution article V).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    pub id: EpisodeId,
    pub text: String,
    /// Byte offset into `signatures.dat` where this episode's HV lives.
    pub signature_offset: u64,
    pub triples: Vec<Triple>,
    pub valid_time: TimeRange,
    /// Transaction time — when agidb learned this fact.
    pub t_tx_start: DateTime<Utc>,
    pub provenance: Provenance,
    pub confidence: f32,
    /// If non-empty, this episode was superseded by the listed one. The
    /// `t_valid_end` of `valid_time` is set in the same write.
    pub superseded_by: Option<EpisodeId>,
}

/// A canonical entity. Multiple aliases collapse to the same `ConceptId`
/// so "Sarah" and "sarah_kelly" point at the same vector.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Concept {
    pub id: ConceptId,
    pub canonical_name: String,
    pub aliases: Vec<String>,
    pub entity_type: String,
}

/// A consolidated fact produced by the consolidation loop. Always links
/// back to the source episodes via `evidence` for provenance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SemanticAtom {
    pub id: SemanticAtomId,
    pub statement: String,
    pub concept: ConceptId,
    pub evidence: Vec<EpisodeId>,
    pub evidence_count: u32,
    pub confidence: f32,
    pub last_referenced: DateTime<Utc>,
    /// Byte offset into `signatures.dat` where the atom's bundle HV lives.
    pub signature_offset: u64,
}

// ---------------------------------------------------------------------------
// Procedural memory
// ---------------------------------------------------------------------------

/// A typed episode shape representing a workflow or skill — procedural
/// memory. Answers "how do I do X?"
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Procedure {
    pub name: String,
    pub description: String,
    pub trigger: String,
    pub preconditions: Vec<String>,
    pub steps: Vec<ProcedureStep>,
    pub postconditions: Vec<String>,
    pub provenance: Option<Provenance>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProcedureStep {
    pub description: String,
    pub tool: Option<String>,
    /// Tool-specific args, opaque to agidb. Encoded as JSON-compatible
    /// text so we don't pull `serde_json::Value` into the type for
    /// callers that don't need it.
    pub args: Option<String>,
}

// ---------------------------------------------------------------------------
// Retrieval
// ---------------------------------------------------------------------------

/// Which recall tier produced a match. Lower variants are higher-quality.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Tier {
    /// Canonical entity match via the concept index.
    Exact,
    /// HDC signature similarity (POPCOUNT-driven).
    Similarity,
    /// Raw-text gist fallback.
    Gist,
    /// Best-effort nearest neighbor with `low_confidence` flag.
    NearestNeighbor,
}

impl Tier {
    /// Tier depth used for fall-through ordering. Lower depth = higher
    /// quality; recall starts at depth 0 and descends until the depth
    /// matches `tier_floor` or matches are found.
    pub const fn depth(self) -> u8 {
        match self {
            Tier::Exact => 0,
            Tier::Similarity => 1,
            Tier::Gist => 2,
            Tier::NearestNeighbor => 3,
        }
    }
}

// ---------------------------------------------------------------------------
// Recall
// ---------------------------------------------------------------------------

/// A retrieval request. Only `cue` is required; the rest have defaults
/// that match the v0.1 targets in [`crate::store`].
#[derive(Clone, Debug)]
pub struct Query {
    /// Free-text cue. Tokenized for tier-A concept lookup and encoded
    /// into a gist signature for tier-C/D fallback.
    pub cue: String,
    /// Maximum number of matches to return.
    pub k: usize,
    /// Bi-temporal "as-of" filter. Episodes whose valid_time does not
    /// contain `as_of` are excluded.
    pub as_of: Option<DateTime<Utc>>,
    /// Confidence floor — matches below this are dropped.
    pub min_confidence: f32,
    /// Deepest tier the cascade is allowed to fall through to. Default
    /// is [`Tier::NearestNeighbor`] (i.e. anything goes).
    pub tier_floor: Tier,
}

impl Query {
    /// Construct a query with sensible defaults: `k = 10`,
    /// `min_confidence = 0.0`, `tier_floor = NearestNeighbor`.
    pub fn cue(text: impl Into<String>) -> Self {
        Self {
            cue: text.into(),
            k: 10,
            as_of: None,
            min_confidence: 0.0,
            tier_floor: Tier::NearestNeighbor,
        }
    }

    pub fn with_k(mut self, k: usize) -> Self {
        self.k = k;
        self
    }

    pub fn with_as_of(mut self, as_of: DateTime<Utc>) -> Self {
        self.as_of = Some(as_of);
        self
    }

    pub fn with_min_confidence(mut self, c: f32) -> Self {
        self.min_confidence = c;
        self
    }

    pub fn with_tier_floor(mut self, t: Tier) -> Self {
        self.tier_floor = t;
        self
    }
}

/// The result of a recall. Per [constitution](../../.specify/memory/constitution.md)
/// article VI, `Recall::matches` is never empty under the default
/// `tier_floor`; the deepest tier always returns nearest neighbors.
///
/// `semantic_atoms` carries any consolidated `SemanticAtom` rows
/// matching the cue's concept tokens — phase 6 surfaces these as a
/// parallel result lane to the episodic matches.
#[derive(Clone, Debug)]
pub struct Recall {
    pub matches: Vec<RecallMatch>,
    pub semantic_atoms: Vec<SemanticMatch>,
    /// The shallowest tier that contributed at least one match.
    pub tier_used: Tier,
    /// Wall-clock elapsed time of the recall call, in milliseconds.
    pub elapsed_ms: u32,
}

/// One row in a `Recall`. Tier-specific confidence calibration is
/// applied by the tier that produced the match.
#[derive(Clone, Debug, PartialEq)]
pub struct RecallMatch {
    pub episode_id: EpisodeId,
    pub text: String,
    pub triples: Vec<Triple>,
    pub confidence: f32,
    pub valid_time: TimeRange,
    pub provenance: Provenance,
    /// `true` iff this episode has a `superseded_by` link set.
    pub superseded: bool,
    pub source_tier: Tier,
}

/// A consolidated semantic atom surfaced as part of a `Recall`. Always
/// carries the list of source `EpisodeId`s in `evidence` so callers
/// can drill back to the verbatim observations behind the atom (per
/// constitution article VII — provenance always).
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticMatch {
    pub atom_id: SemanticAtomId,
    pub statement: String,
    pub concept: ConceptId,
    pub evidence: Vec<EpisodeId>,
    pub evidence_count: u32,
    pub confidence: f32,
    pub last_referenced: DateTime<Utc>,
}

impl From<SemanticAtom> for SemanticMatch {
    fn from(a: SemanticAtom) -> Self {
        Self {
            atom_id: a.id,
            statement: a.statement,
            concept: a.concept,
            evidence: a.evidence,
            evidence_count: a.evidence_count,
            confidence: a.confidence,
            last_referenced: a.last_referenced,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests — round-trip the schema invariants
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_range_contains_point_inside() {
        let start = "2026-05-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let end = "2026-06-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let range = TimeRange {
            start,
            end: Some(end),
        };
        let probe = "2026-05-14T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(range.contains(probe));
    }

    #[test]
    fn time_range_unbounded_end_contains_future() {
        let start = "2026-05-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let range = TimeRange { start, end: None };
        let probe = "2099-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(range.contains(probe));
    }

    #[test]
    fn id_newtype_display_round_trips() {
        let e = EpisodeId::new(42);
        assert_eq!(format!("{e}"), "EpisodeId(42)");
        assert_eq!(e.raw(), 42);
    }
}
