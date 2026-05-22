//! `store` — the redb-backed metadata layer.
//!
//! Layer 3 plumbing. Every Episode, Concept, and SemanticAtom row
//! lives here; HVs themselves live in [`crate::signatures`] and are
//! referenced by `signature_offset`.

use crate::error::{AgidbError, Result};
use crate::hdc::{D_BYTES, HV};
use crate::signatures::SignatureFile;
use crate::types::*;
use chrono::{DateTime, Duration, Utc};
use redb::{Database, MultimapTableDefinition, ReadableTable, TableDefinition};
use roaring::RoaringBitmap;
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Table definitions — the redb schema.
//
// Values are bincode-serialized blobs so we can evolve the in-Rust types
// without rewriting the on-disk format on every change. The
// `format_version` in the manifest table gates breaking changes.
// ---------------------------------------------------------------------------

/// Primary table — every Episode by id.
pub const EPISODES: TableDefinition<u64, Vec<u8>> = TableDefinition::new("episodes");

/// Every Concept by id.
pub const CONCEPTS: TableDefinition<u64, Vec<u8>> = TableDefinition::new("concepts");

/// `entity_name → ConceptId.raw()` lookup. Includes canonical names
/// *and* aliases (layer-2 alias resolution writes both forms here).
pub const CONCEPT_BY_NAME: TableDefinition<&str, u64> = TableDefinition::new("concept_by_name");

/// `ConceptId → many EpisodeId`. Drives tier-A exact recall.
pub const CONCEPT_EPISODES: MultimapTableDefinition<u64, u64> =
    MultimapTableDefinition::new("concept_episodes");

/// Inverted index from an HV active-dim index to a roaring bitmap of
/// `EpisodeId` low 32 bits (sufficient for v0.1 single-node scale).
pub const INVERTED_INDEX: TableDefinition<u32, Vec<u8>> = TableDefinition::new("inverted_index");

/// Every SemanticAtom by id.
pub const SEMANTIC_ATOMS: TableDefinition<u64, Vec<u8>> = TableDefinition::new("semantic_atoms");

/// Append-only audit trail of every consolidation pass.
pub const CONSOLIDATION_LOG: TableDefinition<u64, Vec<u8>> =
    TableDefinition::new("consolidation_log");

/// Manifest values (`format_version`, monotonic counters, …).
pub const MANIFEST: TableDefinition<&str, Vec<u8>> = TableDefinition::new("manifest");

/// Manifest key for the format version u32.
const KEY_FORMAT_VERSION: &str = "format_version";

/// Manifest key for the next-concept-id counter (u64).
const KEY_NEXT_CONCEPT_ID: &str = "next_concept_id";

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

/// Owning handle to a agidb store — the redb database + the mmap'd
/// signatures file held together.
pub struct Store {
    pub db: Database,
    pub signatures: SignatureFile,
    pub config: StoreConfig,
}

impl Store {
    /// Open or create the store at `config.root`. Idempotent — opening
    /// an existing store verifies the manifest's format version.
    pub fn open(config: StoreConfig) -> Result<Self> {
        std::fs::create_dir_all(&config.root)?;
        let db_path = config.root.join("meta.redb");
        let sig_path = config.root.join("signatures.dat");

        let db = Database::create(&db_path)?;
        let signatures = SignatureFile::open(&sig_path)?;

        // Initialize / verify manifest + create every table so later
        // read-only transactions don't trip the "table does not exist"
        // error on an empty store.
        {
            let tx = db.begin_write()?;
            {
                let mut manifest = tx.open_table(MANIFEST)?;
                let stored_version = manifest.get(KEY_FORMAT_VERSION)?.map(|v| v.value());
                match stored_version {
                    Some(bytes) => {
                        let stored: u32 = decode(&bytes)?;
                        if stored != config.format_version {
                            return Err(AgidbError::FormatVersion {
                                got: stored,
                                expected: config.format_version,
                            });
                        }
                    }
                    None => {
                        manifest.insert(KEY_FORMAT_VERSION, encode(&config.format_version)?)?;
                    }
                }
                let has_counter = manifest.get(KEY_NEXT_CONCEPT_ID)?.is_some();
                if !has_counter {
                    manifest.insert(KEY_NEXT_CONCEPT_ID, encode(&1u64)?)?;
                }
                // Touch every table so it exists. redb materializes
                // a table on the first open_table inside a write tx.
                let _ = tx.open_table(EPISODES)?;
                let _ = tx.open_table(CONCEPTS)?;
                let _ = tx.open_table(CONCEPT_BY_NAME)?;
                let _ = tx.open_multimap_table(CONCEPT_EPISODES)?;
                let _ = tx.open_table(INVERTED_INDEX)?;
                let _ = tx.open_table(SEMANTIC_ATOMS)?;
                let _ = tx.open_table(CONSOLIDATION_LOG)?;
            }
            tx.commit()?;
        }

        Ok(Self {
            db,
            signatures,
            config,
        })
    }

    /// Persist an Episode + its HV signature in one transactional unit.
    /// Updates the concept index, the concept-by-name lookup, the
    /// concept→episodes multimap, and the inverted index in the same
    /// commit.
    ///
    /// The caller's `episode.id` is used as-is — phase 2 trusts the
    /// caller to supply unique ids. Collisions overwrite (last-writer-
    /// wins) until a phase-4 sequence-counter lands.
    pub fn observe(&mut self, mut episode: Episode, signature: &HV) -> Result<EpisodeId> {
        // 1. Append the signature outside the redb tx — the mmap and
        //    redb have independent commit cycles, but the offset is
        //    only "live" once the redb row that references it commits,
        //    and a crash before commit leaves at most a junk HV at the
        //    tail of signatures.dat (no dangling reference).
        let offset = self.signatures.append(signature)?;
        episode.signature_offset = offset;
        let episode_id = episode.id;
        let active_dims: Vec<u32> = signature.active_dims().collect();

        // 2. One redb transaction for the row + every index update.
        let tx = self.db.begin_write()?;
        {
            let mut episodes = tx.open_table(EPISODES)?;
            let mut concepts = tx.open_table(CONCEPTS)?;
            let mut concept_by_name = tx.open_table(CONCEPT_BY_NAME)?;
            let mut concept_episodes = tx.open_multimap_table(CONCEPT_EPISODES)?;
            let mut inverted = tx.open_table(INVERTED_INDEX)?;
            let mut manifest = tx.open_table(MANIFEST)?;

            episodes.insert(episode_id.raw(), encode(&episode)?)?;

            // For each subject and object in each triple, ensure the
            // corresponding Concept exists and link it to this episode.
            let mut seen: BTreeSet<String> = BTreeSet::new();
            for tr in &episode.triples {
                for entity_name in [&tr.subject, &tr.object] {
                    if !seen.insert(entity_name.clone()) {
                        continue;
                    }
                    // Materialize the looked-up value (or None) before
                    // any mutating call on the same table — redb's
                    // AccessGuard borrows the table immutably and would
                    // collide with the next insert otherwise.
                    let existing = concept_by_name
                        .get(entity_name.as_str())?
                        .map(|v| v.value());
                    let concept_id = match existing {
                        Some(raw) => ConceptId::new(raw),
                        None => {
                            let new_id = next_concept_id(&mut manifest)?;
                            let concept = Concept {
                                id: new_id,
                                canonical_name: entity_name.clone(),
                                aliases: vec![],
                                entity_type: "unknown".into(),
                            };
                            concepts.insert(new_id.raw(), encode(&concept)?)?;
                            concept_by_name.insert(entity_name.as_str(), new_id.raw())?;
                            new_id
                        }
                    };
                    concept_episodes.insert(concept_id.raw(), episode_id.raw())?;
                }
            }

            // Inverted index: each active dim of the HV gains a
            // pointer to this episode. Roaring bitmaps keep the index
            // compact even with millions of episodes.
            for dim in active_dims {
                let existing = inverted.get(dim)?.map(|v| v.value());
                let mut bitmap = match existing {
                    Some(bytes) => RoaringBitmap::deserialize_from(bytes.as_slice())
                        .map_err(|e| AgidbError::Internal(format!("roaring decode: {e}")))?,
                    None => RoaringBitmap::new(),
                };
                bitmap.insert(episode_id.raw() as u32);
                let mut bytes = Vec::with_capacity(bitmap.serialized_size());
                bitmap
                    .serialize_into(&mut bytes)
                    .map_err(|e| AgidbError::Internal(format!("roaring encode: {e}")))?;
                inverted.insert(dim, bytes)?;
            }
        }
        tx.commit()?;
        self.signatures.flush()?;

        Ok(episode_id)
    }

    /// Fetch an Episode by id.
    pub fn get_episode(&self, id: EpisodeId) -> Result<Option<Episode>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(EPISODES)?;
        match table.get(id.raw())? {
            Some(v) => Ok(Some(decode::<Episode>(&v.value())?)),
            None => Ok(None),
        }
    }

    /// Look up a ConceptId by canonical name or alias.
    pub fn concept_id_for(&self, name: &str) -> Result<Option<ConceptId>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(CONCEPT_BY_NAME)?;
        match table.get(name)? {
            Some(v) => Ok(Some(ConceptId::new(v.value()))),
            None => Ok(None),
        }
    }

    /// Tier-A exact recall. Returns every Episode that references
    /// `concept`, optionally filtered to the bi-temporal slice valid
    /// at `as_of`. Order is unspecified (callers sort if they need a
    /// specific ordering).
    pub fn recall_exact(
        &self,
        concept: ConceptId,
        as_of: Option<DateTime<Utc>>,
    ) -> Result<Vec<Episode>> {
        let tx = self.db.begin_read()?;
        let concept_episodes = tx.open_multimap_table(CONCEPT_EPISODES)?;
        let episodes = tx.open_table(EPISODES)?;

        let mut results = Vec::new();
        for raw in concept_episodes.get(concept.raw())? {
            let raw_id = raw?.value();
            if let Some(v) = episodes.get(raw_id)? {
                let ep: Episode = decode(&v.value())?;
                if let Some(t) = as_of {
                    if !ep.valid_time.contains(t) {
                        continue;
                    }
                }
                results.push(ep);
            }
        }
        Ok(results)
    }

    /// Mark `older` as superseded by `newer`. Closes the old
    /// `valid_time` interval at `newer.valid_time.start - 1ms` and
    /// writes the `superseded_by` link in one transaction.
    pub fn supersede(&mut self, older: EpisodeId, newer: EpisodeId) -> Result<()> {
        let tx = self.db.begin_write()?;
        {
            let mut episodes = tx.open_table(EPISODES)?;

            // Read both in scope of the same write tx so we can't
            // observe a stale `newer.valid_time.start`.
            let newer_bytes = episodes
                .get(newer.raw())?
                .ok_or(AgidbError::UnknownEpisode(newer.raw()))?
                .value();
            let newer_ep: Episode = decode(&newer_bytes)?;

            let older_bytes = episodes
                .get(older.raw())?
                .ok_or(AgidbError::UnknownEpisode(older.raw()))?
                .value();
            let mut older_ep: Episode = decode(&older_bytes)?;

            older_ep.superseded_by = Some(newer);
            older_ep.valid_time.end = Some(newer_ep.valid_time.start - Duration::milliseconds(1));

            episodes.insert(older.raw(), encode(&older_ep)?)?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Flush in-memory state to disk. redb commits are already durable;
    /// this just flushes the signatures mmap.
    pub fn flush(&self) -> Result<()> {
        self.signatures.flush()
    }

    /// Dump every Episode (with its HV) as JSON lines. Round-trips
    /// through `import_jsonl` into a fresh store.
    pub fn export_jsonl(&self, mut writer: impl Write) -> Result<()> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(EPISODES)?;
        for entry in table.iter()? {
            let (_, v) = entry?;
            let episode: Episode = decode(&v.value())?;
            let hv = self.signatures.read(episode.signature_offset)?;
            let record = ExportRecord {
                episode,
                hv: hv.0.to_vec(),
            };
            let line = serde_json::to_string(&record)
                .map_err(|e| AgidbError::Internal(format!("json encode: {e}")))?;
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }
        writer.flush()?;
        Ok(())
    }

    /// Import JSON lines produced by `export_jsonl`. Returns the count
    /// of episodes imported.
    pub fn import_jsonl(&mut self, reader: impl std::io::Read) -> Result<u32> {
        let reader = BufReader::new(reader);
        let mut count = 0u32;
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let record: ExportRecord = serde_json::from_str(&line)
                .map_err(|e| AgidbError::Internal(format!("json decode: {e}")))?;
            if record.hv.len() != D_BYTES {
                return Err(AgidbError::Internal(format!(
                    "import: expected {} hv bytes, got {}",
                    D_BYTES,
                    record.hv.len()
                )));
            }
            let mut bytes = [0u8; D_BYTES];
            bytes.copy_from_slice(&record.hv);
            let hv = HV(bytes);
            self.observe(record.episode, &hv)?;
            count += 1;
        }
        Ok(count)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize)]
struct ExportRecord {
    episode: Episode,
    hv: Vec<u8>,
}

fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
    bincode::serialize(value).map_err(|e| AgidbError::Internal(format!("bincode encode: {e}")))
}

fn decode<T: for<'de> serde::Deserialize<'de>>(bytes: &[u8]) -> Result<T> {
    bincode::deserialize(bytes).map_err(|e| AgidbError::Internal(format!("bincode decode: {e}")))
}

/// Read-modify-write the monotonic concept-id counter inside the
/// caller's open write transaction.
fn next_concept_id(manifest: &mut redb::Table<&str, Vec<u8>>) -> Result<ConceptId> {
    let raw = manifest.get(KEY_NEXT_CONCEPT_ID)?.map(|v| v.value());
    let current: u64 = match raw {
        Some(bytes) => decode(&bytes)?,
        None => 1,
    };
    manifest.insert(KEY_NEXT_CONCEPT_ID, encode(&(current + 1))?)?;
    Ok(ConceptId::new(current))
}
