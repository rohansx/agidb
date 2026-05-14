//! `store` — the redb-backed metadata layer.
//!
//! Layer 3 plumbing. Every Episode, Triple, Concept, and SemanticAtom
//! row lives here; HVs themselves live in [`crate::signatures`] and
//! are referenced by `signature_offset`.
//!
//! Phase 2 lands the real schema + crash-safety code. The public
//! surface in this module is final; the implementations are `todo!()`
//! so phase 2 work has a concrete RED state.

use crate::error::Result;
use crate::signatures::SignatureFile;
use crate::types::*;
use chrono::{DateTime, Utc};
use redb::{Database, TableDefinition};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Table definitions — the redb schema.
//
// Values are bincode-serialized blobs so we can evolve the in-Rust
// types without rewriting the on-disk format on every change. The
// `format_version` in `manifest.toml` gates breaking changes.
// ---------------------------------------------------------------------------

/// Primary table — every Episode by id.
pub const EPISODES: TableDefinition<u64, Vec<u8>> = TableDefinition::new("episodes");

/// Every Triple by id.
pub const TRIPLES: TableDefinition<u64, Vec<u8>> = TableDefinition::new("triples");

/// Every Concept by id.
pub const CONCEPTS: TableDefinition<u64, Vec<u8>> = TableDefinition::new("concepts");

/// `entity_name → ConceptId` lookup. Includes canonical names *and*
/// aliases (the alias resolver writes both forms here).
pub const CONCEPT_BY_NAME: TableDefinition<&str, u64> = TableDefinition::new("concept_by_name");

/// Inverted index from an HV active-dim index to a roaring-bitmap of
/// EpisodeIds whose signature has that bit set.
pub const INVERTED_INDEX: TableDefinition<u32, Vec<u8>> = TableDefinition::new("inverted_index");

/// Every SemanticAtom by id.
pub const SEMANTIC_ATOMS: TableDefinition<u64, Vec<u8>> = TableDefinition::new("semantic_atoms");

/// Append-only audit trail of every consolidation pass.
pub const CONSOLIDATION_LOG: TableDefinition<u64, Vec<u8>> =
    TableDefinition::new("consolidation_log");

/// Manifest values (format_version, created_at, last_compacted_at, …).
pub const MANIFEST: TableDefinition<&str, Vec<u8>> = TableDefinition::new("manifest");

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Configuration for opening a store. Defaults match the v0.1 targets
/// in [`crate::types`] / `docs/spec/tech-spec.md`.
#[derive(Clone, Debug)]
pub struct StoreConfig {
    pub root: PathBuf,
    pub strict: bool,
    pub format_version: u32,
}

impl StoreConfig {
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            strict: false,
            format_version: crate::signatures::FORMAT_VERSION,
        }
    }
}

/// One log entry per consolidation pass; written into `CONSOLIDATION_LOG`.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConsolidationLogEntry {
    pub at: DateTime<Utc>,
    pub episodes_scanned: u32,
    pub atoms_created: u32,
    pub contradictions: u32,
    pub atoms_decayed: u32,
    pub bytes_reclaimed: u64,
}

/// Owning handle to a sochdb store — the redb database + the mmap'd
/// signatures file held together. Phase 2 wires the bi-temporal write
/// path inside `observe`, `supersede`, and `recall_exact`.
pub struct Store {
    pub db: Database,
    pub signatures: SignatureFile,
    pub config: StoreConfig,
}

impl Store {
    /// Open or create the store at `config.root`. Idempotent — opening
    /// an existing store verifies the manifest and replays the WAL.
    pub fn open(_config: StoreConfig) -> Result<Self> {
        todo!("phase 2: open redb at root/meta.redb, open signatures.dat, verify manifest")
    }

    /// Persist an Episode + its HV signature in one transactional unit.
    /// Updates the concept index, the concept-by-name lookup, and the
    /// inverted index in the same commit.
    pub fn observe(&mut self, _episode: Episode, _signature: &crate::hdc::HV) -> Result<EpisodeId> {
        todo!("phase 2: write episode + signature + indexes inside one redb tx")
    }

    /// Fetch an Episode by id.
    pub fn get_episode(&self, _id: EpisodeId) -> Result<Option<Episode>> {
        todo!("phase 2: redb read of EPISODES table + bincode decode")
    }

    /// Look up a ConceptId by canonical name or alias.
    pub fn concept_id_for(&self, _name: &str) -> Result<Option<ConceptId>> {
        todo!("phase 2: redb read of CONCEPT_BY_NAME table")
    }

    /// Tier-A exact recall — given a query that resolved to a
    /// ConceptId, return every Episode that references the concept,
    /// optionally filtered to the bi-temporal slice valid at `as_of`.
    pub fn recall_exact(
        &self,
        _concept: ConceptId,
        _as_of: Option<DateTime<Utc>>,
    ) -> Result<Vec<Episode>> {
        todo!("phase 2: concept_index → episodes, filter by valid_time")
    }

    /// Mark `older` as superseded by `newer`. Writes the supersession
    /// link, closes the old `valid_time` interval, and commits both
    /// changes atomically.
    pub fn supersede(&mut self, _older: EpisodeId, _newer: EpisodeId) -> Result<()> {
        todo!("phase 2: bi-temporal supersession write")
    }

    /// Flush in-memory state to disk. Called automatically on `Drop`
    /// but also exposed so callers can checkpoint deterministically
    /// (mainly for tests).
    pub fn flush(&self) -> Result<()> {
        todo!("phase 2: db.flush + signatures.flush")
    }

    /// Dump every Episode and SemanticAtom as JSON lines. The export
    /// is stable enough to reimport into a fresh store — round-trip
    /// covered by the property tests.
    pub fn export_jsonl(&self, _writer: impl std::io::Write) -> Result<()> {
        todo!("phase 2: stream-write episodes + atoms as jsonl")
    }

    /// Import JSON lines produced by `export_jsonl`. Each line is one
    /// record; format dictated by `manifest.format_version`.
    pub fn import_jsonl(&mut self, _reader: impl std::io::BufRead) -> Result<u32> {
        todo!("phase 2: stream-parse jsonl, observe each line")
    }
}
