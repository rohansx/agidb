# sochdb — architecture

this document explains how sochdb is built, end to end. it covers the three layers, the write path, the read path, and the consolidation loop. for the deep technical detail of each layer, see the layer-specific docs.

## the three-layer model

sochdb is built in three distinct layers. each layer has one job. the layers are stacked: layer 1 sits on top of layer 2 which sits on top of layer 3.

```
┌──────────────────────────────────────────────────────────────┐
│  LAYER 1 — RECALL                                            │
│  the mind-like layer. HDC signatures, binding, bundling,     │
│  hamming-distance retrieval, tiered confidence.              │
│  this is what the user experiences.                          │
└──────────────────────────────────────────────────────────────┘
                              ▲
                              │
┌──────────────────────────────────────────────────────────────┐
│  LAYER 2 — EXTRACTION                                        │
│  the scaffolding layer. GLiNER ONNX entity/relation          │
│  extraction. turns natural language into structured triples  │
│  that can be bound into robust signatures.                   │
└──────────────────────────────────────────────────────────────┘
                              ▲
                              │
┌──────────────────────────────────────────────────────────────┐
│  LAYER 3 — STORAGE                                           │
│  the plumbing layer. redb for metadata + bi-temporal         │
│  indexes. mmap'd flat files for signatures. crash-safe,      │
│  ACID, pure rust.                                            │
└──────────────────────────────────────────────────────────────┘
```

**the user only ever interacts with layer 1.** they call `observe()` and `recall()`. layers 2 and 3 are invisible engineering underneath.

### the biological mapping

cognitive psychology recognizes five memory systems. the three engineering layers above are *how* sochdb is built; the five biological tiers below are *what* sochdb stores. they are orthogonal concerns.

| biological tier | what it is | sochdb implementation | layer(s) involved |
|---|---|---|---|
| sensory memory | raw signal, <1s | upstream of sochdb (out of scope) | none |
| working memory | active context, ~7 items | session-scoped recall with recency boost | layer 1 (retrieval) |
| episodic memory | events with time/place/people | `Episode` with bi-temporal stamps and HDC signature | layers 1, 2, 3 |
| semantic memory | decoupled facts | `SemanticAtom` produced by consolidation | layers 1, 3 |
| procedural memory | workflows and skills | `Procedure` (typed episode shape) | layers 1, 2, 3 |

four of five tiers are in scope for v0.1. sensory memory belongs to the input pipeline upstream of any database. for the full mapping rationale, see [BIOLOGICAL_MAPPING.md](../product/biological-mapping.md).

### why three layers and not one

a real biological brain stores memories as activation patterns in cortical neurons. it doesn't have an "extraction layer" — the cortex does extraction implicitly as part of perception. it doesn't have a "storage layer" — synaptic weights *are* the storage.

we don't have 86 billion neurons on a laptop. so we have to simulate the brain-like *behavior* using conventional computer parts. that simulation requires three explicit layers:

- layer 1 simulates the brain-like behavior (signatures, retrieval-by-overlap)
- layer 2 prepares the input so that signatures are robust to phrasing
- layer 3 reliably persists the signatures to disk

the user gets brain-like behavior. the engineering underneath does the work.

## the write path

what happens when you call `db.observe(text)`:

```
USER
  │
  │  db.observe("Sarah recommended a thai place called Bawri
  │              in Bandra last weekend")
  ▼
┌─────────────────────────────────────────────────────────┐
│ LAYER 2: EXTRACTION                                     │
│                                                         │
│ 1. GLiNER ONNX model (local, ~150ms on CPU) extracts:  │
│    entities: [Sarah/Person, Bawri/Restaurant,           │
│               Bandra/Location, last weekend/Time]       │
│    relations: [(Sarah, recommended, Bawri),             │
│                (Bawri, located_in, Bandra),             │
│                (Bawri, type, thai_restaurant)]          │
│                                                         │
│ 2. resolve "last weekend" → 2026-05-09 (valid time)     │
│ 3. attach confidence scores from GLiNER                 │
│ 4. canonicalize entity names against existing concepts  │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────┐
│ LAYER 1: BINDING                                        │
│                                                         │
│ 1. look up or assign 8192-bit hypervectors for each     │
│    concept: Sarah_hv, Bawri_hv, Bandra_hv,              │
│    recommended_hv, located_in_hv, ...                   │
│                                                         │
│ 2. bind triples into role-filler patterns:              │
│    triple1 = (SUBJ⊗Sarah) ⊕ (PRED⊗recommended)         │
│              ⊕ (OBJ⊗Bawri)                              │
│    triple2 = (SUBJ⊗Bawri) ⊕ (PRED⊗located_in)          │
│              ⊕ (OBJ⊗Bandra)                             │
│                                                         │
│ 3. bundle triples into one episode signature:           │
│    episode_signature = triple1 ⊕ triple2 ⊕ ...          │
│                       ⊕ (TIME⊗2026-05-09_hv)            │
│                                                         │
│ 4. also compute a raw-text gist signature               │
│    (for fallback retrieval)                             │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────┐
│ LAYER 3: STORAGE                                        │
│                                                         │
│ 1. append signature bytes (1 KB each) to                │
│    signatures.dat at offset N                           │
│ 2. write episode row to redb:                           │
│    episode_id → { text, signature_offset, triples,      │
│                   t_valid_start, t_valid_end,           │
│                   t_tx_start, provenance, confidence }  │
│ 3. update inverted index: each active dim in signature  │
│    → roaring bitmap of episode_ids                      │
│ 4. update concept index: each entity name → list of     │
│    related episode_ids                                  │
│ 5. fsync. return episode_id.                            │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
                        USER gets EpisodeId
```

**total time: ~200ms.** dominated by GLiNER inference. all subsequent steps are microseconds.

## the read path

what happens when you call `db.recall(query)`:

```
USER
  │
  │  db.recall("what thai place did sarah mention?")
  ▼
┌─────────────────────────────────────────────────────────┐
│ LAYER 2: PARTIAL EXTRACTION                             │
│                                                         │
│ 1. GLiNER extracts partial triple shape:                │
│    [(Sarah, mentioned, ?), (?, type, thai_restaurant)]  │
│ 2. mark unknowns with placeholder roles                 │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────┐
│ LAYER 1: PARTIAL SIGNATURE + RETRIEVAL                  │
│                                                         │
│ 1. compute partial query signature using the same       │
│    binding math (with unbound slots for "?")            │
│                                                         │
│ 2. TIER A (exact): look up by canonical entity name     │
│    in the concept index. if hit with high confidence,   │
│    return.                                              │
│                                                         │
│ 3. TIER B (similarity): POPCOUNT hamming distance       │
│    between query signature and stored signatures,       │
│    filtered by inverted-index intersection (only        │
│    consider episodes that share ≥ N active dims).       │
│    return top-K with hamming-derived confidence.        │
│                                                         │
│ 4. TIER C (gist): if tier B confidence < threshold,     │
│    fall back to raw-text gist signature similarity.     │
│    return top-K with lower confidence.                  │
│                                                         │
│ 5. TIER D (nearest neighbors): if all above fail,       │
│    return nearest neighbors with explicit               │
│    low_confidence=true flag.                            │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────┐
│ LAYER 3: HYDRATION                                      │
│                                                         │
│ 1. for each matched episode_id, fetch row from redb:    │
│    text, triples, timestamps, provenance.               │
│ 2. apply bi-temporal filter (drop episodes where        │
│    t_valid_end < query.as_of unless historical mode).   │
│ 3. apply supersession filter (mark superseded facts).   │
│ 4. return Recall { matches, tier_used, confidence }.    │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
                       USER gets Recall
                       (matches with confidence + provenance)
```

**total time: target p95 under 50ms** on a laptop with 100k episodes. dominated by GLiNER on the query (~50ms) plus the POPCOUNT scan (~5ms for 100k signatures with AVX-512). no network calls. no API keys. no LLM.

for very fast recall, GLiNER on the query can be bypassed in favor of a lighter sparse encoder — this is a tunable.

## the consolidation loop

what happens in the background, periodically (default: every 5 minutes when idle, or on `db.consolidate()`):

```
BACKGROUND WORKER (tokio task)
  │
  ▼
┌─────────────────────────────────────────────────────────┐
│ STEP 1: CLUSTER                                         │
│                                                         │
│ scan recent episodic signatures (last 7 days).          │
│ cluster by hamming distance threshold.                  │
│ clusters with N ≥ 3 episodes are candidates for         │
│ consolidation.                                          │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────┐
│ STEP 2: SEMANTIC ATOM CREATION                          │
│                                                         │
│ for each cluster:                                       │
│ - bundle member signatures into a semantic atom         │
│ - record evidence_count = N                             │
│ - record last_seen = max(t_tx_start)                    │
│ - link semantic atom to source episode_ids              │
│ - write to semantic_atoms table in redb                 │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────┐
│ STEP 3: CONTRADICTION DETECTION                         │
│                                                         │
│ for each new episode in the last sweep:                 │
│ - find existing facts with same (subject, predicate)    │
│ - if overlapping t_valid window and different object:   │
│   - newer fact: t_valid_start = now                     │
│   - older fact: t_valid_end = now - 1ms                 │
│                  superseded_by = newer.id               │
│ - emit a consolidation log entry                        │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────┐
│ STEP 4: DECAY                                           │
│                                                         │
│ for unreferenced semantic atoms (no recall hits in      │
│ 90 days): decay confidence by factor λ.                 │
│ when confidence falls below floor: archive to           │
│ cold storage (separate file), remove from hot index.    │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────┐
│ STEP 5: COMPACT                                         │
│                                                         │
│ rewrite signatures.dat in-place to remove archived      │
│ entries. update offsets in redb episode rows.           │
│ run at low priority; pauses if main thread is busy.     │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
                      return ConsolidationReport
```

consolidation is the analog of sleep in biological memory — what the brain does when you're not actively using it. it's also the answer to "what stops the db from growing forever" and "how does old, redundant memory get compressed into general knowledge."

## the data flow at a glance

```
                    OBSERVE                          RECALL
                       │                                │
                       ▼                                ▼
                ┌──────────────┐                ┌──────────────┐
                │  LAYER 2     │                │  LAYER 2     │
                │  extraction  │                │  partial     │
                │  (GLiNER)    │                │  extraction  │
                └──────┬───────┘                └──────┬───────┘
                       │                                │
                       ▼                                ▼
                ┌──────────────┐                ┌──────────────┐
                │  LAYER 1     │                │  LAYER 1     │
                │  binding +   │                │  signature + │
                │  bundling    │                │  tiered      │
                │              │                │  retrieval   │
                └──────┬───────┘                └──────┬───────┘
                       │                                │
                       ▼                                ▼
                ┌──────────────┐                ┌──────────────┐
                │  LAYER 3     │ ◀─────────────▶│  LAYER 3     │
                │  redb +      │                │  hydration + │
                │  mmap +      │                │  bi-temporal │
                │  index       │                │  filter      │
                │  update      │                │              │
                └──────────────┘                └──────────────┘
                       ▲
                       │
                ┌──────┴───────┐
                │  BACKGROUND  │
                │ consolidate  │
                └──────────────┘
```

## key design choices and why

| choice | alternative | why we chose it |
|---|---|---|
| HDC signatures as the primary representation | dense embeddings (BGE, OpenAI) | deterministic math, no model dependency, 8x smaller, POPCOUNT-fast retrieval, encoder-free |
| GLiNER for extraction | LLM-based extraction (Mem0, Cognee) | local, no API key, no hallucination at write time |
| redb for metadata | sqlite, rocksdb, sled | pure rust, ACID, MVCC, no FFI, embedded-first |
| mmap'd flat files for signatures | store signatures in redb | fixed-size, POPCOUNT-scanned in bulk, redb B-tree would be overhead |
| bi-temporal supersession | overwrite on update | preserves history, enables "as of" queries, auditable |
| tiered recall with explicit confidence | binary hit/miss | graceful degradation, agent always gets something to work with |
| single binary, embedded | client-server architecture | runs offline, no infra, sqlite-like deployment story |
| rust top to bottom | python + rust extensions | sub-50ms p95, no GC, fits the embedded-db ecosystem |
| MCP server as first-class interface | rest api | reaches agents directly, no glue code |

## what's deliberately *not* in the architecture

these are out of scope for v0.1:

- **distributed mode.** single-node only. when an agent needs distributed memory, that's a v2 problem.
- **multi-modal storage.** text first. images, audio, video deferred.
- **fine-tuning hooks.** sochdb stores facts, not gradients. fine-tuning is downstream.
- **a query language.** the whole point is no query language.
- **a UI.** sochdb is a library, not an app. consumers build UIs on top.
- **transactional guarantees beyond ACID per operation.** sochdb is not a system of record for orders or payments.

## next steps in this doc set

- [LAYER_1_RECALL.md](./layer-1-recall.md) — the math of HDC signatures and retrieval
- [LAYER_2_EXTRACTION.md](./layer-2-extraction.md) — how GLiNER turns text into triples
- [LAYER_3_STORAGE.md](./layer-3-storage.md) — redb schema and on-disk layout
- [TECH_SPEC.md](../spec/tech-spec.md) — the rust API, types, performance targets
