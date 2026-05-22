# agidb — Layer 3: Storage

> The plumbing. redb for metadata + bi-temporal indexes. mmap'd flat files
> for signatures. Append-only logs for the self-model audit trail. Crash-safe,
> ACID, pure Rust. 16 tables in v2.1.

## What layer 3 is

Layer 3 persists everything reliably. It exposes a typed API to layers 1 and 2, knows nothing about cognition or extraction. Its job: store, retrieve, and never lose data.

The constraint: pure Rust, embedded, ACID, single binary. No server, no FFI, no garbage collector.

The components: **redb** for metadata + indexes, **mmap'd flat files** for HDC signatures, **append-only logs** for audit and self-model history.

## The directory layout (v2.1)

An agidb database is a directory:

```
memory.agidb/
├── meta.redb              redb database — 16 tables (see below)
├── signatures.dat         mmap'd flat file — 1024-byte HV slots
├── manifest.toml          format version, hyperparams, schema version, encoder configs
├── audit.log              append-only signed audit log (optional, v0.3+)
└── encoders/              v2.1: ONNX weights, projection matrices
    ├── vjepa2-gigantic-256.onnx
    ├── wav2vec-bert-2.0.onnx
    ├── llama-3.2-3b.onnx
    ├── gliner-small-v2.5.onnx
    └── projections.bin    seeded random projection matrices (per modality)
```

For v2.0 deployments (text-only), the `encoders/` directory contains only GLiNER. For v2.1, it adds the three multimodal encoders.

Encoder weights downloaded on first use, not bundled in the binary. Manifest pins HuggingFace SHA hashes for reproducibility.

## The redb schema (16 tables in v2.1)

### Tables inherited from sochdb v1 (8 tables)

#### 1. `episodes` — primary episode storage

```rust
table_name: "episodes"
key:        EpisodeId (u64)
value:      EpisodeRow {
              text: String,
              triples: Vec<Triple>,
              signature_offset: u64,         // offset into signatures.dat
              gist_signature_offset: u64,    // offset into signatures.dat
              provenance: Provenance,
              confidence: f32,
              t_valid_start: DateTime<Utc>,
              t_valid_end: Option<DateTime<Utc>>,
              t_tx_start: DateTime<Utc>,
              t_tx_end: Option<DateTime<Utc>>,
              superseded_by: Option<EpisodeId>,
              tombstoned_at: Option<DateTime<Utc>>,
              session_id: Option<SessionId>,
              // v2.1 additions:
              modalities: Vec<Modality>,      // [Text], or [Video, Audio, Text], etc.
              modality_signature_offsets: Option<ModalitySignatureOffsets>,
            }
```

#### 2. `concepts` — canonical entity storage

```rust
table_name: "concepts"
key:        ConceptId (u64)
value:      Concept {
              canonical_name: String,
              aliases: Vec<String>,
              concept_type: ConceptType,
              signature_offset: u64,
              created_at: DateTime<Utc>,
              withdrawn_at: Option<DateTime<Utc>>,
            }
```

#### 3. `concept_by_name` — name → ConceptId lookup

```rust
table_name: "concept_by_name"
key:        String (canonical_name or alias)
value:      ConceptId
```

#### 4. `concept_episodes` — multimap concept → episodes

```rust
table_name: "concept_episodes"
key:        ConceptId
value:      Vec<EpisodeId>  // multi-valued
```

#### 5. `inverted_index` — bit → episodes containing that bit set

```rust
table_name: "inverted_index"
key:        u16 (bit index 0..8192)
value:      RoaringBitmap<EpisodeId>
```

Used by tier B retrieval to intersect candidate episodes by bit-set membership before POPCOUNT scoring.

#### 6. `semantic_atoms` — consolidated facts

```rust
table_name: "semantic_atoms"
key:        AtomId (u64)
value:      SemanticAtom {
              subject: ConceptId,
              predicate: String,
              object: Value,
              signature_offset: u64,
              evidence_count: u32,
              source_episodes: Vec<EpisodeId>,
              t_valid_start: DateTime<Utc>,
              t_valid_end: Option<DateTime<Utc>>,
              confidence: f32,
              last_referenced: DateTime<Utc>,
              tombstoned_at: Option<DateTime<Utc>>,
            }
```

#### 7. `consolidation_log` — audit of consolidation events

```rust
table_name: "consolidation_log"
key:        ConsolidationId (u64)
value:      ConsolidationEntry {
              ran_at: DateTime<Utc>,
              atoms_created: Vec<AtomId>,
              atoms_updated: Vec<AtomId>,
              contradictions_resolved: Vec<(EpisodeId, EpisodeId)>,
              episodes_scanned: u64,
              duration_ms: u32,
            }
```

#### 8. `manifest` — schema version, hyperparams

```rust
table_name: "manifest"
key:        String
value:      Value
```

### Tables added in v2.0 (6 tables)

#### 9. `goals` — first-class goal storage

```rust
table_name: "goals"
key:        GoalId (u64)
value:      Goal {
              parent_id: Option<GoalId>,
              description: String,
              state: GoalState,
              success_criteria: Vec<SuccessCriterion>,
              deadline: Option<DateTime<Utc>>,
              signature_offset: u64,
              created_at: DateTime<Utc>,
              updated_at: DateTime<Utc>,
              provenance: Provenance,
            }
```

#### 10. `beliefs` — first-class belief storage

```rust
table_name: "beliefs"
key:        BeliefId (u64)
value:      Belief {
              claim: String,
              subject: ConceptId,
              predicate: String,
              object: Value,
              confidence: f32,
              evidence: Vec<EpisodeId>,
              contradictions: Vec<EpisodeId>,
              revision_log_ref: BeliefRevisionLogId,
              signature_offset: u64,
              t_valid_start, t_valid_end, t_tx_start, t_tx_end,
              provenance: Provenance,
              withdrawn_at: Option<DateTime<Utc>>,
            }
```

#### 11. `belief_revisions` — append-only revision audit

```rust
table_name: "belief_revisions"
key:        (BeliefId, RevisionIndex)
value:      BeliefRevision {
              timestamp: DateTime<Utc>,
              previous_confidence: f32,
              new_confidence: f32,
              triggering_evidence: Option<EpisodeId>,
              reason: String,
              llm_used: bool,                  // was an LLM consulted?
              llm_model: Option<String>,
            }
```

#### 12. `sensory_buffer` — floor 1 ring buffer

```rust
table_name: "sensory_buffer"
key:        SensoryId (u64)
value:      SensoryFrame {
              modality: Modality,
              data: SensoryData,
              received_at: DateTime<Utc>,
              surprise_score: f32,
              promoted_to: Option<EpisodeId>,
            }
```

Ring behavior: when capacity reached, drop oldest unless promoted. Capacity default 1000 entries or 60s, whichever smaller.

#### 13. `learning_events` — floor 7 audit log

```rust
table_name: "learning_events"
key:        LearningEventId (u64, auto-increment)
value:      LearningEvent (the closed enum from cognitive-primitives.md)
```

Append-only. Never updated, never deleted. The audit log of every state change.

#### 14. `tombstones` — non-destructive removal tracking

```rust
table_name: "tombstones"
key:        AuditId (u64)
value:      Tombstone {
              target: UnlearnTarget,
              reason: String,
              cascade_summary: UnlearnReport,
              created_at: DateTime<Utc>,
              expires_at: DateTime<Utc>,       // default + 30 days
              recoverable: bool,
            }
```

### Tables added in v2.1 (2 tables)

#### 15. `self_vector_history` — floor 7 self-vector snapshots

```rust
table_name: "self_vector_history"
key:        SelfVectorSnapshotId (u64, auto-increment)
value:      SelfVectorSnapshot {
              taken_at: DateTime<Utc>,
              signature_offset: u64,           // self-vector HV stored in signatures.dat
              drift_from_previous: u32,        // hamming distance
              trigger: SelfVectorTrigger,      // Consolidation, Unlearn, Manual
              consolidation_id: Option<ConsolidationId>,
              unlearn_audit_id: Option<AuditId>,
            }
```

Lets the agent ask `self_vector_at(time)` to replay the self-vector at any historical point. Also enables drift analysis ("how much has my self-model changed in the last month?").

#### 16. `encoder_versions` — v2.1 encoder pinning

```rust
table_name: "encoder_versions"
key:        EncoderRole  // "vjepa2", "wav2vec_bert", "llama_text", "gliner"
value:      EncoderConfig {
              version: String,
              weight_sha: String,
              projection_seed: u64,
              registered_at: DateTime<Utc>,
              huggingface_url: String,
            }
```

If a binary opens a database with mismatched encoder hashes, agidb errors out with a clear migration message. No silent encoder swaps.

## The signature file

```
signatures.dat layout:
┌────────────────────────────────┐
│  Header (32 bytes)             │
│  - magic: "AGIDB-SIG\0\0\0\0\0\0\0" (16 bytes)
│  - format_version: u32 (4)
│  - signature_size_bits: u32 (4) — 8192 for v2
│  - count: u64 (8)
├────────────────────────────────┤
│  Slot 0: 1024 bytes (HV)       │
├────────────────────────────────┤
│  Slot 1: 1024 bytes (HV)       │
├────────────────────────────────┤
│  ...                           │
└────────────────────────────────┘
```

mmap'd with `memmap2`. Read-only mapping for layer 1 retrieval (POPCOUNT scan). Append-only writes via fsync.

**Why mmap not redb-stored:** signatures are fixed-size, never updated in-place, frequently bulk-scanned. mmap is the right tool. POPCOUNT over 100k signatures runs at memory bandwidth speed.

**Grow strategy:** when capacity reached, the file grows by 2× (subject to a 1GB minimum increment). mmap remapped after grow.

**Slot reuse:** when episodes are tombstoned, their slots are marked invalid (header bit). Compaction (v0.3+) rewrites the file dropping invalid slots.

## Bi-temporal model

Every fact carries four timestamps:
- `t_valid_start, t_valid_end` — the time interval during which the fact is true in the world
- `t_tx_start, t_tx_end` — the time interval during which agidb has known the fact

Queries can specify both:
```rust
db.recall(Query::cue("...").valid_as_of(date1).transaction_as_of(date2)).await
```

Updates produce new rows; old rows are marked `superseded_by`. Nothing is silently overwritten.

This is how legal and financial systems track changing facts (Snodgrass 1995, Date/Darwen/Lorentzos 2002). agidb mirrors that pattern for cognitive memory.

### Why bi-temporal matters

Two different questions:
- "Was Sarah a vegetarian in March?" — valid-time query: get the fact valid at March 15.
- "When did I learn Sarah became vegetarian?" — transaction-time query: get the row whose `t_tx_start` is earliest.

These are different queries with different answers, and both matter for an agent that needs to reason about its own evolving knowledge.

### Implementation

Each table that stores facts has the four timestamps as columns. Indexes:
- `episodes_by_valid_time` — secondary index keyed by `t_valid_start`
- `episodes_by_tx_time` — secondary index keyed by `t_tx_start`
- `episodes_active` — filtered view of `tombstoned_at IS NULL AND superseded_by IS NULL`

Updates use redb transactions: insert new row + update old row's `t_valid_end` and `t_tx_end` atomically.

## Tombstones and the 30-day window

Tombstoning is article XVI's non-destructive removal. The data is marked invalid but kept for 30 days for recovery, then compacted away.

**Tombstone behavior:**
- Tombstoned rows excluded from recall (filtered in layer 1's bi-temporal filter).
- Signatures in mmap marked invalid but bytes preserved until compaction.
- Within 30 days, `restore_within_window(audit_id)` can undo.
- After 30 days, compaction physically removes tombstoned rows.
- The `LearningEvent::Unlearned` record is permanent regardless of compaction.

**Self-vector subtraction (v2):** when unlearn runs, the self-vector is updated as:
```
self_vector ← self_vector - α * bundle(tombstoned_signatures)
```
And the corrected snapshot is appended to `self_vector_history`. See [cognitive-primitives.md](./cognitive-primitives.md) section on Unlearn.

## Storage operations

### Write — observe()

```rust
async fn observe(&self, text: String, ctx: ObserveContext) -> Result<EpisodeId> {
    let triples = self.extractor.extract(&text).await?;
    let episode_id = self.next_episode_id();
    let signature = encode_episode_signature(&triples);
    let gist_signature = encode_gist_signature(&text);
    let sig_offset = self.signatures.append(&signature)?;
    let gist_offset = self.signatures.append(&gist_signature)?;
    let txn = self.redb.begin_write()?;
    {
        let mut episodes = txn.open_table(EPISODES_TABLE)?;
        episodes.insert(episode_id, EpisodeRow { ... })?;
        let mut inverted = txn.open_multimap_table(INVERTED_INDEX)?;
        for bit_idx in signature.set_bits() {
            inverted.insert(bit_idx as u16, episode_id)?;
        }
        let mut concept_eps = txn.open_multimap_table(CONCEPT_EPISODES)?;
        for triple in &triples {
            concept_eps.insert(triple.subject, episode_id)?;
        }
        let mut events = txn.open_table(LEARNING_EVENTS)?;
        events.insert(self.next_event_id(), LearningEvent::EpisodeStored { ... })?;
    }
    txn.commit()?;
    self.signatures.fsync()?;
    Ok(episode_id)
}
```

### Write — observe_multimodal() (v2.1)

```rust
async fn observe_multimodal(
    &self,
    video: Option<VideoClip>,
    audio: Option<AudioClip>,
    text: Option<String>,
    ctx: ObserveContext,
) -> Result<EpisodeId> {
    let video_sig = video.as_ref().map(|v| self.vjepa.encode_and_project(v)).transpose()?;
    let audio_sig = audio.as_ref().map(|a| self.wav2vec.encode_and_project(a)).transpose()?;
    let text_latent_sig = text.as_ref().map(|t| self.llama.encode_and_project(t)).transpose()?;
    let triples = if let Some(t) = &text {
        self.extractor.extract(t).await?
    } else { vec![] };
    let active_goal = self.active_goals().await?.first().map(|g| g.id);
    let belief_ids = self.active_belief_ids().await?;
    let episode_sig = encode_multimodal_episode(
        &triples, video_sig, audio_sig, text_latent_sig,
        active_goal, &belief_ids, TimeBucket::now(),
    );
    let surprise = self.compute_brain_calibrated_surprise(&episode_sig).await?;
    if surprise < self.theta_brain && !ctx.force_promote { return Ok(EpisodeId::ZERO); }
    let episode_id = self.next_episode_id();
    let sig_offset = self.signatures.append(&episode_sig)?;
    let video_offset = video_sig.as_ref().map(|s| self.signatures.append(s)).transpose()?;
    let audio_offset = audio_sig.as_ref().map(|s| self.signatures.append(s)).transpose()?;
    let text_offset = text_latent_sig.as_ref().map(|s| self.signatures.append(s)).transpose()?;
    let modality_offsets = ModalitySignatureOffsets { video: video_offset, audio: audio_offset, text: text_offset };
    let modalities: Vec<Modality> = [video.is_some().then_some(Modality::Video { path: video_path }), audio.is_some().then_some(Modality::Audio { path: audio_path, duration_ms: 30000 }), text.as_ref().map(|_| Modality::Text)].into_iter().flatten().collect();
    let txn = self.redb.begin_write()?;
    {
        let mut episodes = txn.open_table(EPISODES_TABLE)?;
        episodes.insert(episode_id, EpisodeRow { ..., modalities, modality_signature_offsets: Some(modality_offsets), ... })?;
        let mut events = txn.open_table(LEARNING_EVENTS)?;
        events.insert(self.next_event_id(), LearningEvent::MultimodalEpisodeStored { id: episode_id, modalities, at: Utc::now() })?;
    }
    txn.commit()?;
    self.signatures.fsync()?;
    Ok(episode_id)
}
```

### Read — recall()

See [layer-1-recall.md](./layer-1-recall.md) for the read path. Layer 3's role: serve signature lookups (via mmap), serve metadata lookups (via redb), apply bi-temporal filters.

### Write — unlearn()

```rust
async fn unlearn(&self, target: UnlearnTarget, reason: String) -> Result<UnlearnReport> {
    let cascade = self.compute_cascade(&target).await?;
    let txn = self.redb.begin_write()?;
    let mut self_vector = self.self_vector().await?;
    let mut tombstoned_signatures = vec![];
    let tombstone_time = Utc::now();
    {
        let mut episodes = txn.open_table(EPISODES_TABLE)?;
        for ep_id in &cascade.episodes {
            if let Some(mut row) = episodes.get(*ep_id)? {
                row.value().tombstoned_at = Some(tombstone_time);
                episodes.insert(*ep_id, row)?;
                tombstoned_signatures.push(self.signatures.read(row.value().signature_offset)?);
            }
        }
        // similar for beliefs, atoms, procedures...
    }
    let alpha = 0.05;
    let bundle_tomb = bundle(&tombstoned_signatures);
    self_vector = self_vector.subtract(&bundle_tomb, alpha);
    let drift = self_vector.hamming(&self.current_self_vector);
    let self_vec_offset = self.signatures.append(&self_vector)?;
    let snapshot_id = self.next_self_vector_snapshot_id();
    let audit_id = self.next_audit_id();
    {
        let mut snapshots = txn.open_table(SELF_VECTOR_HISTORY)?;
        snapshots.insert(snapshot_id, SelfVectorSnapshot { taken_at: tombstone_time, signature_offset: self_vec_offset, drift_from_previous: drift, trigger: SelfVectorTrigger::Unlearn, consolidation_id: None, unlearn_audit_id: Some(audit_id) })?;
        let mut tombstones = txn.open_table(TOMBSTONES)?;
        tombstones.insert(audit_id, Tombstone { target: target.clone(), reason: reason.clone(), cascade_summary: cascade.report.clone(), created_at: tombstone_time, expires_at: tombstone_time + Duration::days(30), recoverable: true })?;
        let mut events = txn.open_table(LEARNING_EVENTS)?;
        events.insert(self.next_event_id(), LearningEvent::Unlearned { target, cascade_size: cascade.total_size(), self_vector_drift: drift, audit_id, at: tombstone_time })?;
    }
    txn.commit()?;
    self.signatures.fsync()?;
    Ok(cascade.report)
}
```

The self-vector subtraction step (v2.1: lines computing `bundle_tomb`, `self_vector.subtract`, snapshot append) is what makes unlearn real not just hiding.

### Compaction (v0.3+)

Periodically (default monthly, or manual `db.compact()`):
1. Scan tombstones table for entries with `expires_at < now`.
2. Permanently remove those rows from their source tables.
3. Rewrite `signatures.dat` to drop invalidated slots.
4. Rebuild indexes if needed.

The `LearningEvent::Unlearned` records survive compaction. Article XVI guarantees the audit trail is permanent.

## ACID guarantees

- **Atomicity:** redb transactions are atomic. Multi-table writes commit or abort together.
- **Consistency:** schema invariants enforced by typed wrappers around redb tables. Foreign-key-like constraints (concept_ids must exist) verified at write time.
- **Isolation:** redb uses MVCC. Readers don't block writers; writers serialize.
- **Durability:** signatures fsync on append. redb commits fsync the redb file. Crash mid-write → recover to last successful commit.

## Performance characteristics

| Operation | Target | Notes |
|---|---|---|
| Episode insert (v2.0) | ~5ms | dominated by GLiNER extraction; storage is ~1ms |
| Episode insert (v2.1) | ~2s CPU / ~500ms GPU | dominated by V-JEPA 2; storage is still ~1ms |
| Episode lookup by ID | < 1ms | redb point query |
| Signature load (mmap) | < 0.1ms | mmap'd, OS page cache |
| Inverted index intersection (10 bits) | ~5ms | roaring bitmap AND |
| Consolidation pass (10k episodes) | ~5s | mostly POPCOUNT clustering |
| Tombstone unlearn (100-episode cascade) | ~100ms | cascade compute + write + self-vector update |
| Self-vector update (consolidation) | ~5ms | bundle + redb write |
| Full POPCOUNT scan (100k signatures) | ~5ms portable / ~1.5ms AVX-512 | layer 1 tier C/D scan |
| Full POPCOUNT scan (1M signatures) | ~50ms portable / ~15ms AVX-512 | at this scale switch to LSH (v0.3+) |
| `fsync` after batch insert | ~10ms | OS-dependent |

## Migration paths

### sochdb v1 → agidb v2.0

The v1 schema is a subset of v2.0. Opening a sochdb v1 file with agidb v2.0:
1. Verify magic + format_version.
2. New tables (`goals`, `beliefs`, `belief_revisions`, `sensory_buffer`, `learning_events`, `tombstones`) created empty on first write.
3. Existing tables unchanged.
4. Bump `manifest.toml` schema version to "agidb-2.0".

No data migration required. Read-write compatible.

### agidb v2.0 → agidb v2.1

The v2.0 schema is a subset of v2.1. Opening a v2.0 file with agidb v2.1:
1. Verify magic + format_version.
2. New tables (`self_vector_history`, `encoder_versions`) created empty.
3. Existing `episodes` rows lack `modalities` and `modality_signature_offsets` columns (Option<> → None).
4. Encoder downloads triggered on first multimodal write.
5. Bump `manifest.toml` schema version to "agidb-2.1".

### Cross-version reading

agidb v2.1 binary can read v2.0 and v1 files. Forward compatibility (older binary opening newer file) errors out with clear message. Migration tool: `agidb migrate --from old.agidb --to new.agidb`.

## Test coverage

| Test | What it verifies |
|---|---|
| Episode roundtrip | observe → recall returns identical episode |
| Bi-temporal supersession | superseded episodes excluded from default recall, included with as_of |
| Tombstone behavior | tombstoned excluded from recall; LearningEvent emitted |
| Tombstone expiry | after 30 days, compaction removes data; LearningEvent survives |
| Cascade correctness | unlearn cascade matches dependency graph |
| Self-vector subtraction | unlearn produces correct self-vector drift |
| Crash recovery | kill mid-write → recover to last commit; no torn writes |
| Inverted index correctness | scan-by-bit returns same episode set as full POPCOUNT scan |
| Concurrent reads + writes | MVCC behavior, no read-write blocking |
| 100k-episode load test | p95 retrieval under 50ms |
| Sochdb v1 → agidb v2.0 read | v1 files open and read correctly |
| Agidb v2.0 → v2.1 read | v2.0 files open and read correctly |
| Encoder version mismatch | clear error, suggest migration |

Phase 2 (sochdb v1 inherited) covers tests 1-8. Phase 11 (unlearn) extends tests 4-6. Phase 14 (v2.1) adds tests 11-13.

## Why this stack

| Choice | Alternative | Why |
|---|---|---|
| redb for metadata | sqlite, rocksdb, sled | pure Rust, ACID, MVCC, no FFI |
| mmap signatures | redb-stored | fixed-size, bulk POPCOUNT, OS page cache |
| Append-only learning_events | mutable log | constitution article XVI: audit must be permanent |
| 4-timestamp bi-temporal | flat timestamps | enables "as of" queries, supersession audit |
| 30-day tombstone window | immediate hard delete | recovery, compliance, trust |
| **Self-vector snapshots (v2.1)** | log-only | enables historical self-vector queries, drift analysis |
| **Per-modality signature offsets (v2.1)** | only bundled episode | enables factoring stored episodes by modality |
| **Encoder versioning (v2.1)** | implicit | prevents silent encoder swaps, ensures BAMS reproducibility |
| Roaring bitmaps inverted index | hash-set | sparse-set arithmetic for fast intersections |
| Crash safety via fsync + redb commits | no durability | data loss unacceptable |
| Single binary, embedded | client-server | sqlite-like deployment, no infra |

## What this layer doesn't do

- **Extract entities, encode video, project to HDC.** Layer 2's job.
- **Run retrieval cascades, goal-biasing, attention tracing.** Layer 1's job.
- **Run consolidation.** Consolidation worker (separate module).
- **Run any ML inference.** Pure I/O + indexing. ML inference lives in layer 2's encoder wrappers.

## Dependency graph

```
redb (phase 2, done) ──┐
mmap signatures (phase 2, done) ──┤
bi-temporal model (phase 2, done) ┤
                                  ├──> layer 3 v2.0 complete
8 tables from sochdb (phase 2, done) ──┘

cognitive primitive tables (phase 9 + 10 + 11) ──> 14-table v2.0 schema
                                                    │
                                                    ▼
                                          v2.0 launch (month 9)

self_vector_history table (phase 10 enhanced + phase 14) ──┐
encoder_versions table (phase 14) ──────────────────────────┤
modality columns on episodes (phase 14) ────────────────────┤
                                                              ▼
                                          16-table v2.1 schema
                                                              │
                                                              ▼
                                          v2.1 launch (month 12)
```

Layer 3 is the most stable layer. The schema evolves carefully — each new table is additive, every change is versioned, migrations are explicit. The plumbing has to be reliable because every other layer depends on it.
