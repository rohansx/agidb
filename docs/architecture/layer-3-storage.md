# layer 3 — storage (the plumbing layer)

this is the boring layer that makes everything else durable. it stores signatures, metadata, indexes, and bi-temporal timestamps on disk in a crash-safe, ACID-compliant way. the user never interacts with this layer directly.

the goal of this doc is to explain *what's actually stored, where, and why* — so you understand the engineering tradeoffs and can extend sochdb without breaking persistence guarantees.

## the storage stack

sochdb uses two storage primitives:

```
┌─────────────────────────────────────────────────────────────┐
│  redb                                                       │
│  (pure-rust embedded ACID key-value store)                  │
│                                                             │
│  stores: episodes, concepts, triples, indexes,              │
│          bi-temporal metadata, consolidation log            │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  signatures.dat (mmap'd flat file)                          │
│                                                             │
│  stores: 8192-bit binary signatures, one after another,     │
│          referenced by byte offset                          │
└─────────────────────────────────────────────────────────────┘
```

a sochdb database on disk is one directory:

```
./memory.soch/
├── metadata.redb         # all structured metadata, ACID-managed
├── signatures.dat        # mmap'd binary signature blobs
├── atoms.redb            # concept atom registry (Sarah_hv, etc.)
├── consolidation.log     # append-only audit trail
└── manifest.toml         # version, hyperparams (dim=8192, etc.)
```

## why redb (not sqlite, not rocksdb, not sled)

we evaluated four embedded stores. the comparison:

| store | language | ACID | concurrency | rust-native | maturity |
|---|---|---|---|---|---|
| **redb** | rust | yes | MVCC | yes | stable 2.x |
| sqlite | C | yes | locks | no (FFI) | extremely mature |
| rocksdb | C++ | yes | snapshots | no (FFI) | extremely mature |
| sled | rust | yes (beta) | optimistic | yes | beta, slowing |

we chose **redb** because:

1. **pure rust.** no FFI, no C build dependencies. clean cross-compilation for macOS/Linux/Windows/musl.
2. **MVCC and ACID.** multiple readers + single writer with snapshot isolation. matches the consolidation worker's needs.
3. **embedded-first.** designed for single-process, single-file usage. no server.
4. **active and stable.** redb hit 2.0 in 2024 and has steady releases.
5. **simple API surface.** typed table definitions, transactions, range queries. nothing exotic.

sqlite would also work and is more mature. the rust-native story wins because the rest of sochdb is rust, the build story stays clean, and we don't need full SQL. if redb ever becomes a limitation, swapping to sqlite (via rusqlite) is a contained 2-3 day project. the choice is not load-bearing.

## what lives in redb

redb organizes data into typed tables. each table is `Table<Key, Value>`. sochdb defines these:

### `episodes`

the core write log. one row per `observe()` call.

```rust
// Table<EpisodeId, EpisodeRow>
struct EpisodeRow {
    text:             String,            // raw observation text
    signature_offset: u64,                // byte offset in signatures.dat
    triples:          Vec<TripleId>,      // references to triples table
    t_valid_start:    DateTime<Utc>,
    t_valid_end:      Option<DateTime<Utc>>,
    t_tx_start:       DateTime<Utc>,      // when sochdb received it
    t_tx_end:         Option<DateTime<Utc>>,
    superseded_by:    Option<EpisodeId>,  // for state evolution
    confidence:       f32,
    provenance:       Provenance,         // who/what/where
    extractor:        ExtractorTag,       // GLiNER, LLM(model_name), manual
}
```

### `triples`

individual subject-predicate-object facts extracted from episodes. one episode can produce many triples.

```rust
// Table<TripleId, TripleRow>
struct TripleRow {
    episode_id:  EpisodeId,
    subject:     ConceptId,
    predicate:   PredicateId,
    object:      ConceptId,
    confidence:  f32,
    t_valid_start: DateTime<Utc>,
    t_valid_end:   Option<DateTime<Utc>>,
    superseded_by: Option<TripleId>,
}
```

triples are stored separately from episodes for two reasons: (a) bi-temporal supersession works at triple granularity, not episode granularity (one episode might contain three facts, only one of which gets superseded later), and (b) queries that scan by predicate or by subject can index over triples without touching full episodes.

### `concepts`

the canonical entity registry. one row per known entity.

```rust
// Table<ConceptId, ConceptRow>
struct ConceptRow {
    canonical_name: String,
    aliases:        Vec<String>,
    entity_type:    String,
    atom_offset:    u64,                  // byte offset in atoms.redb
    semantic_atom_offset: Option<u64>,    // if consolidated
    evidence_count: u32,                  // how many episodes mention this
    first_seen:     DateTime<Utc>,
    last_seen:      DateTime<Utc>,
}
```

### `predicates`

canonical predicates. predicates are stored separately from concepts because the synonym table is hot — "said", "told me", "mentioned" all map to one canonical predicate atom.

```rust
// Table<PredicateId, PredicateRow>
struct PredicateRow {
    canonical_form: String,
    synonyms:       Vec<String>,
    atom_offset:    u64,
}
```

### `concept_index`

a reverse index from canonical name to all episodes mentioning that concept. used by tier-A retrieval.

```rust
// Table<String, RoaringBitmap<u64>>
// canonical_name → set of EpisodeIds
```

we use roaring bitmaps because episode_ids are u64 and the bitmap intersection is fast.

### `inverted_index`

the inverted index from active signature dimension to episode_ids whose signatures have that bit set. this is the first-pass filter for tier-B retrieval.

```rust
// Table<u32, RoaringBitmap<u64>>
// active_dim_index (0..8192) → set of EpisodeIds
```

`croaring-rs` handles the bitmap operations. lookup is `O(k)` where k is the number of active bits in the query signature (~50), and the intersection over k posting lists narrows the candidate set from millions to typically a few hundred.

### `consolidation_log`

append-only audit trail of consolidation operations. for debugging, observability, and the "show me what changed this week" API.

```rust
// Table<LogEntryId, ConsolidationLogEntry>
struct ConsolidationLogEntry {
    timestamp:   DateTime<Utc>,
    operation:   ConsolidationOp,
    episode_ids: Vec<EpisodeId>,
    details:     String,
}

enum ConsolidationOp {
    SemanticAtomCreated { concept: ConceptId, evidence: u32 },
    ContradictionDetected { older: TripleId, newer: TripleId },
    Decayed { atom: ConceptId, new_confidence: f32 },
    Archived { episode: EpisodeId, reason: String },
}
```

### `semantic_atoms`

consolidated knowledge — the output of the consolidation worker. semantic atoms represent "things sochdb has seen enough times to consider general knowledge."

```rust
// Table<SemanticAtomId, SemanticAtomRow>
struct SemanticAtomRow {
    concept:        ConceptId,
    signature:      Vec<u8>,              // bundled signature
    evidence:       Vec<EpisodeId>,       // source episodes
    evidence_count: u32,
    confidence:     f32,
    last_referenced: DateTime<Utc>,
}
```

## what lives in `signatures.dat`

the mmap'd flat file holds raw 8192-bit signatures. each signature is exactly 1024 bytes. the file is append-only during normal operation; the consolidation worker may rewrite it during compaction.

layout:

```
[signature 0: 1024 bytes][signature 1: 1024 bytes]...[signature N: 1024 bytes]
```

referenced by byte offset: episode_id → episode_row → signature_offset → mmap[offset..offset+1024].

**why mmap, not redb?**

redb is a B-tree. it's optimized for keyed lookups, not bulk sequential scans. our hot read path needs to POPCOUNT-scan thousands of signatures in milliseconds — that's a streaming workload over a contiguous byte array. mmap gives us that directly:

```rust
let stored_sig: &[u8; 1024] = &mmap[offset..offset+1024];
let hamming: u32 = popcount_xor(query_sig, stored_sig);
```

no deserialization, no allocator, no copy. the kernel handles caching via the page cache. AVX-512 POPCOUNT processes 512 bits per instruction; 1024 bytes is 16 instructions. for 100k candidates that's 1.6M instructions, well under 1ms on a modern CPU.

mmap also gives us **zero-cost durability**. the OS flushes pages to disk on its own schedule, and we can `msync` explicitly when we want a hard fsync barrier. for v0.1 we sync after every observe; in v0.2 we'll batch.

**what about updates?** signatures are immutable once written. supersession happens at the metadata layer (`superseded_by` in the episode row). the signature itself is never modified — it's only ever appended.

**what about deletion?** we don't delete signatures. they're archived: marked in metadata as archived, and removed from the active inverted index. periodic compaction (during consolidation) rewrites signatures.dat to drop archived entries and updates offsets. this is the same pattern as PostgreSQL's VACUUM or LSM-tree compaction.

## the bi-temporal model

every fact in sochdb has **two** time axes:

- **valid time** (`t_valid_start`, `t_valid_end`) — when the fact was true *in the world*
- **transaction time** (`t_tx_start`, `t_tx_end`) — when the fact was known *to sochdb*

these are independent. you can record on May 14 (transaction time) a fact that was true from January (valid time). you can also record on May 14 a fact that's true *now* and going forward (valid_start = May 14, valid_end = None).

### why two axes

consider: you tell your agent in January that you live in Mumbai. you tell it in May that you moved to Berlin in April. now:

- **valid time** for "lives in Mumbai" = January 1 to March 31 (you used to)
- **valid time** for "lives in Berlin" = April 1 onwards (current)
- **transaction time** for "lives in Mumbai" = January (when learned)
- **transaction time** for "lives in Berlin" = May (when learned)

if your agent asks "where does Rohan live?" in May, sochdb returns Berlin (current valid time). if your agent asks "where did Rohan live in February?", sochdb returns Mumbai. if you ask "what did sochdb believe in February?", sochdb returns Mumbai (because that's all it knew at the time). three different correct answers, one storage model.

### supersession instead of overwriting

when a new fact contradicts an old one with overlapping valid time, sochdb does **not** delete the old fact. it:

1. sets `older.t_valid_end = newer.t_valid_start - 1ms`
2. sets `older.superseded_by = newer.id`
3. inserts the new fact with its own row

both rows remain queryable. queries default to "current truth" (filter `where t_valid_end is null or t_valid_end > now`), but historical queries with `as_of: DateTime` bypass that filter.

this is exactly Graphiti's "non-destructive update" pattern, applied at the triple level.

## indexes and why they exist

sochdb maintains four indexes:

| index | maps | used by | typical size |
|---|---|---|---|
| `concept_index` | canonical_name → episode_ids | tier-A retrieval | O(unique entities) |
| `inverted_index` | active_dim (0..8192) → episode_ids | tier-B first-pass filter | O(8192 × avg posting len) |
| `temporal_index` | (t_valid_start, t_valid_end) → episode_ids | bi-temporal range queries | O(episodes) |
| `predicate_index` | predicate_id → triple_ids | "what does sochdb know about X relation" | O(unique predicates) |

all four live in redb tables. all four are updated transactionally on `observe()` — the write succeeds only if all index updates commit together.

## crash safety

redb is ACID with single-writer MVCC. an `observe()` call is a single transaction that includes:

1. write `EpisodeRow` to `episodes` table
2. write `TripleRow`s to `triples` table
3. append signature bytes to signatures.dat + msync
4. update `concept_index`, `inverted_index`, `temporal_index`
5. (if supersession) update `superseded_by` on older triples
6. commit transaction

if the process crashes between steps 3 and 6, redb's WAL rolls the transaction back on next open. the signature bytes in signatures.dat are unreferenced and get GC'd during the next compaction. **no observed episode is ever partially persisted.**

we treat signatures.dat as "additional state outside redb" and reconcile on startup: scan the file, find any offset not referenced by any `EpisodeRow`, mark for GC.

## storage characteristics

| quantity | value | notes |
|---|---|---|
| signature size | 1 KB | 8192 bits |
| typical episode row | ~500 bytes | text varies |
| typical triple row | ~100 bytes | |
| 1M episodes | ~1 GB | dominated by signatures |
| 10M episodes | ~10 GB | linear; mmap handles it |
| open cost (cold) | ~50ms | redb header + mmap initialization |
| open cost (warm) | <5ms | page cache populated |

these are well within "embedded database" expectations. a sochdb file is the same shape as a sqlite file — single directory, opens in milliseconds, ships with the app.

## what's deferred to v0.2+

- **compression of signatures.** binary signatures are highly compressible (random-looking but with patterns); zstd would shrink the file 30-50%. defer until 100k-episode customers need it.
- **encryption at rest.** sochdb databases will often contain personal data. AES-GCM encryption of signatures.dat and redb file. v0.2.
- **replication / backup hooks.** event log of all `observe()` calls so external systems can mirror. v0.2.
- **distributed mode.** out of scope. sochdb is embedded.
- **DWM compression** (the HPE Hippocampus structure). targets 10M+ episode scale. v1.0+.

## next reads

- [LAYER_1_RECALL.md](./layer-1-recall.md) — how the data in this layer gets retrieved
- [LAYER_2_EXTRACTION.md](./layer-2-extraction.md) — how the data in this layer was created
- [TECH_SPEC.md](../spec/tech-spec.md) — the rust types and API surface
