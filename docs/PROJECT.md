# agidb — Project Guide (v2)

> The single-document reference for agidb v2: what it is, why it exists,
> what's built today (inheriting from sochdb v1), the full architecture,
> the v2.0 substrate plan (9 months), the v2.1 brain-alignment milestone
> (12 months), and the complete 5-year roadmap toward AGI-grade
> infrastructure. If you read one file, read this one.

**Last updated:** 2026-05-20 · **Status:** v2.0 pre-alpha · **Inherits from:** sochdb v1 (phases 0, 1, 2, 4, 6 complete)

---

## Table of contents

1. [What agidb is](#1-what-agidb-is)
2. [Why agidb exists](#2-why-agidb-exists)
3. [The pivot from sochdb v1 to agidb v2](#3-the-pivot-from-sochdb-v1-to-agidb-v2)
4. [Where we are right now](#4-where-we-are-right-now)
5. [Architecture](#5-architecture)
6. [The data model](#6-the-data-model)
7. [The build roadmap](#7-the-build-roadmap)
8. [Brain-alignment in v2.1+](#8-brain-alignment-in-v21)
9. [Feature catalogue](#9-feature-catalogue)
10. [Tech stack](#10-tech-stack)
11. [The decision gate](#11-the-decision-gate)
12. [The 5-year trajectory](#12-the-5-year-trajectory)
13. [How to navigate the project](#13-how-to-navigate-the-project)

---

## 1. What agidb is

agidb is **the cognitive substrate for autonomous AI agents** — content-addressable hyperdimensional memory, first-class goals and beliefs, bi-temporal supersession, sleep-like consolidation, and a non-destructive unlearn primitive. One Rust binary, one API, no query language. The database AGI systems will run on top of.

It is a **new category of database**. Not a vector database, not a graph database, not a key-value store. It is built for a consumer no existing database was designed for: an autonomous AI agent that needs to *remember, reason, revise, and forget* over months and years.

```rust
let db = Agidb::open("./memory.agidb").await?;

db.observe("Sarah recommended Bawri in Bandra last weekend").await?;
db.assert_belief(Belief::new("Sarah likes thai food").with_confidence(0.8)).await?;
db.set_goal(Goal::new("find a thai place for the team dinner")).await?;

let result = db.recall("what thai place did sarah mention?").await?;
// → Bawri, confidence 0.94, with provenance back to the original observation,
//   goal-biased because "find a thai place" is currently active.
```

No SQL. No Cypher. No embedding API calls. No separate vector DB. No rerank step. One function.

### The wedge

agidb integrates binding and recall **without an external index** — storage and retrieval share the same representation. And on top of that substrate, it adds the four primitives no other database has: first-class goals, revisable beliefs, sensory buffering, and non-destructive unlearn. These five things together are the wedge.

**v2.1 adds a sixth differentiator: brain-aligned multimodal sensory encoding.** Same encoder stack as Meta FAIR's TRIBE v2 (V-JEPA 2 video, Wav2Vec-BERT audio, Llama-3.2-3B text). Surprise threshold calibrated against neural surprise signals from 720-subject fMRI ground truth. This makes agidb the first agent memory substrate with publishable cortical alignment.

### Who it's for

- **Developers building autonomous AI agents** currently fighting with mem0, Letta, Zep, Cognee, LangMem, or a hand-rolled vector-DB-plus-graph stack — who need not just memory but typed cognition, with full provenance and audit.
- **Teams building toward AGI** — frontier-adjacent startups and research groups who need a substrate where goals, beliefs, and self-model are first-class, not bolted on.
- **Developers in regulated industries** — healthcare, legal, finance — who need auditable memory, non-destructive updates, and a non-destructive unlearn API to handle right-to-be-forgotten.
- **Developers building local-first / offline-first AI** — coding agents, desktop assistants, on-device personal AIs — who can't depend on cloud services.
- **Cognitive science researchers** (v2.1+) — who want to benchmark agent memory architectures against human cortical ground truth via BAMS.

### Who it's not for

A general-purpose database, a full-text search engine, a pure vector-similarity store, a hosted-only service, a multimodal-document store, a distributed database, or a knowledge-graph editor with a UI. Use the right tool for those, then put agidb on top.

---

## 2. Why agidb exists

Every existing database was designed for a different consumer: postgres for accountants, mongodb for app developers, neo4j for analysts, pinecone/qdrant for retrieval pipelines, mem0/letta/zep for chat-style RAG memory. **None were designed for an autonomous agent that needs to remember, reason, revise, and forget over time with full provenance and audit.**

The standard agent-memory pattern today is a six-step pipeline: embed every conversation → store vectors → embed the query → similarity search → rerank with another LLM call → stuff chunks into the prompt. This pipeline has six structural problems:

| Problem | Why it happens |
|---|---|
| **Latency** | Every recall is multiple network calls; p95 is often 1–3s, sometimes 60s |
| **Cost** | Every recall is embedding API calls + context-window tokens |
| **No temporal grounding** | Vector DBs don't know what was true *when* |
| **No provenance** | Vector retrieval returns chunks with weak attribution |
| **No graceful degradation** | Below the similarity threshold, queries return nothing |
| **No consolidation** | The store grows without bound; no "sleep", no compaction |

And on top of those, four problems specific to autonomous agents:

| Problem | Why it happens |
|---|---|
| **No first-class goals** | Goals get stored as text. State machines, parent-child structure, success criteria all live in agent code, not the database |
| **No first-class beliefs** | Beliefs get stored as facts. Revision history, confidence updates, contradiction tracking all happen in agent code, badly |
| **No introspection surface** | The agent has no way to ask "what did I learn?" because learning events aren't recorded |
| **No clean unlearn** | Removing a memory cascades through dependent semantic atoms, beliefs, and procedures — nobody handles this correctly |

These aren't bugs — they're properties of the wrong primitive. agidb is the right primitive.

### The three foundations

1. **Content-addressable storage** — memories are high-dimensional binary signatures; retrieval is bit-overlap counting, not query parsing. (How the hippocampus works.)
2. **Bi-temporal supersession** — new facts don't overwrite old ones; they supersede, and you can query the DB "as of" any historical date. (How legal/financial systems — and human memory — track changing facts.)
3. **Sleep-like consolidation** — a surprise-gated background worker compacts repeated episodic patterns into semantic concepts, decays unused memory, flags contradictions. (The McClelland-McNaughton-O'Reilly complementary-learning-systems model, extended with surprise gradients.)

### The four extensions that make it AGI-grade

4. **Goals as first-class typed shapes** — state machines with parent-child structure, success criteria, deadlines. First database to ship this.
5. **Beliefs with explicit revision** — confidence-tracked claims with audit trails of every revision. The agent can prove what it believed, when, and why.
6. **Sensory buffer with surprise gating** — a short-lived ring buffer of recent input, promoted to episodic memory only when surprise exceeds threshold.
7. **Non-destructive unlearn API** — cascading removal of facts with full audit trail. Right-to-be-forgotten as a first-class operation.

### The brain-aligned extension (v2.1)

8. **Multimodal sensory encoding via TRIBE-aligned encoders.** V-JEPA 2 for video, Wav2Vec-BERT for audio, Llama-3.2-3B for text. Each modality projected to 8192-bit HDC signatures via thresholded random projection (Charikar 2002). Episode signatures fused via VSA role-filler binding — factorable, unlike attention-based fusion.
9. **Brain-calibrated surprise gating.** Surprise threshold empirically fit against neural surprise predicted by TRIBE v2 on 720-subject fMRI. Replaces the magic threshold from v2.0 with measurement-grounded calibration.
10. **BAMS — the brain-aligned memory similarity benchmark.** Six-cortical-network RSA evaluation against TRIBE v2 ground truth. First brain-alignment benchmark for agent memory. Paper to ICLR 2026 MemAgents.

### Why now

1. **Agent memory became a category** — Mem0 ($24M Series A, October 2025), Letta ($10M seed, September 2024), Cognee ($7.5M seed February 2026), Zep — all real funding, no AGI substrate yet.
2. **HDC/VSA research matured for production** — Torchhd (2023), karunaratne et al. nature electronics 2020, PathHD, HPE Hippocampus papers showing binary-signature retrieval beating vector DBs by 31× latency and 14× token cost.
3. **The embedded-DB category got serious in Rust** — duckdb, lancedb, redb, surrealdb, tigerbeetle all matured.
4. **Frontier labs are not building externalizable substrates.** Anthropic's September 2025 memory tool is a file-directory CRUD API. OpenAI's April 2025 ChatGPT memory upgrade is a product feature. Google's Gemini Personal Context is a product feature. The open vendor-neutral substrate wedge is empirically unoccupied.
5. **NEW: Brain-encoding foundation models matured (March 2026)** — Meta TRIBE v2 released open weights for a foundation model predicting fMRI BOLD across 720 subjects from V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B. This makes brain-aligned evaluation tractable for the first time. agidb v2.1 is built on the same encoder stack to inherit alignment.

---

## 3. The pivot from sochdb v1 to agidb v2

agidb v2 is the v2 successor to sochdb v1. **every line of sochdb's code carries forward.** the HDC math is the same. the storage layout is the same. bi-temporal supersession is the same. agidb is sochdb extended with the cognitive primitives an AGI substrate requires.

If you used sochdb v1, agidb v2 is a strict superset. No migrations required for the substrate features you already use; new tables and types are additive.

### What carries forward unchanged

- **The HDC kernel.** bind, bundle, hamming, AVX-512/NEON/portable POPCOUNT paths. perfect as-is.
- **The redb + mmap storage layer.** the on-disk format works for everything we'd add. just more tables.
- **Bi-temporal supersession.** central to agidb, already shipped in v1.
- **Episode encoding and signature math.** the binding for episodic memory works identically for semantic atoms and beliefs.
- **The four-tier recall cascade.** tier A/B/C/D works for any memory type.
- **Consolidation clustering and contradiction detection.** the math is correct for all the floors that need it.
- **The constitution and design discipline.** test-first, red-green rhythm, no-LLM-in-read-path, never-return-empty — all preserved.

### What extends naturally (adds rather than changes)

- **Floor 6: goals and beliefs as first-class types.** New `Goal` and `Belief` types in `agidb-core::types`. New redb tables. New API methods.
- **Floor 1: sensory buffer with surprise gating.** Lightweight ring buffer in redb. New API surface for raw observations before episodic promotion.
- **Floor 7: self-model and learning log.** Extension of the existing `consolidation_log` into a general `learning_events` log. New introspection methods.
- **Unlearn API.** Cascading non-destructive removal with audit log. Brand new but uses existing supersession primitives.
- **Neurosymbolic interface.** Explicit translation between HDC signatures and structured triples/beliefs. Already implicit; makes the seam first-class.

### What gets added in v2.1 (brain-alignment)

- **Multimodal sensory encoders.** New `agidb-sensory` crate. V-JEPA 2, Wav2Vec-BERT, Llama-3.2-3B as feature extractors. ONNX/Candle backends.
- **HDC projection layer.** Charikar 2002 thresholded random projection from dense latents (1024d, 2048d) to 8192-bit signatures. Deterministic, seed-fixed.
- **Brain-calibrated surprise threshold.** Calibration tooling against TRIBE v2 fMRI predictions.
- **BAMS benchmark suite.** New `agidb-bams` crate. RSA evaluation against TRIBE-derived cortical ground truth.

### What gets reconsidered

| current sochdb stance | agidb v2 update |
|---|---|
| sensory memory is out of scope | sensory buffer is in scope (lightweight, v0.1) |
| 6-month v0.1 ships memory db | 9-month v2.0 substrate + 12-month v2.1 brain-alignment milestone |
| no LLM in any path | no LLM in read path; LLMs may participate at write time for belief revision and consolidation |
| success criteria = mem0/zep/letta benchmarks | + cognitive benchmarks (goal consistency, belief revision, unlearn cascade) + BAMS at v2.1 |
| name = sochdb | name = agidb |
| domain = sochdb.dev (collision) | domain = agidb.ai |
| text-only sensory | text + video + audio sensory in v2.1, brain-aligned |

### What does not change

The wedge. Content-addressable HDC retrieval, bi-temporal supersession, embedded Rust binary, no LLM in read path, never return empty, full provenance. These are still the differentiators against mem0/zep/letta/cognee. The AGI pivot adds cognitive primitives *on top of* this foundation — it doesn't replace the foundation. Brain-alignment in v2.1 is *additive* on top of the cognitive primitives — it doesn't replace them either.

---

## 4. Where we are right now

agidb v2 inherits sochdb v1's five completed phases. v2.0 needs five new phases for the AGI substrate (phases 9-13). v2.1 adds three more phases for brain-alignment (phases 14-16). Total: 8 phases of substrate work (carrying forward from sochdb) + 5 new phases of cognitive primitives + 3 new phases of brain-alignment = **16 phases**.

### Phase completion (carried forward from sochdb v1)

| Phase | Scope | Status |
|---|---|---|
| **0 — Setup** | Workspace, CI, docs, constitution, ADRs | ✅ **Complete (from sochdb v1)** |
| **1 — HDC kernel** | `HV` type, bind, bundle, hamming | ✅ **Complete (from sochdb v1)** |
| **2 — Storage** | redb metadata, mmap'd signatures, bi-temporal | ✅ **Complete (from sochdb v1)** |
| **3 — Extraction** | GLiNER ONNX entity/relation extraction | ⬜ Not started |
| **4 — Binding + recall** | Episode encoding, tier A/B/C/D | ✅ **Complete (from sochdb v1)** |
| **5 — MCP + Python** | MCP server, pyo3 bindings | ⬜ Not started |
| **6 — Consolidation** | Clustering, semantic atoms, contradictions | ✅ **Complete (from sochdb v1)** |
| **7 — Decision gate** | Benchmark vs Mem0/Zep/Letta | ⬜ Not started |
| **8 — Hardening + launch** | Whitepaper, design partners, launch | ⬜ Not started |

### New phases for the AGI pivot (v2.0)

| Phase | Scope | Status |
|---|---|---|
| **9 — Cognitive primitives** | `Goal` + `Belief` types, state machines, revision audit | ⬜ Not started |
| **10 — Sensory + self-model** | Sensory buffer, surprise gating, learning log | ⬜ Not started |
| **11 — Unlearn API** | Non-destructive cascading unlearn with audit trail | ⬜ Not started |
| **12 — Neurosymbolic interface** | Explicit signature ↔ triple/belief translation layer | ⬜ Not started |
| **13 — Cognitive benchmarks** | Goal consistency, belief revision, unlearn cascade tests | ⬜ Not started |

### New phases for the brain-alignment milestone (v2.1)

| Phase | Scope | Status |
|---|---|---|
| **14 — Multimodal sensory** | V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B encoders, HDC projection | ⬜ Not started |
| **15 — Brain-calibrated surprise** | Calibration tooling against TRIBE v2 fMRI ground truth | ⬜ Not started |
| **16 — BAMS benchmark** | Six-cortical-network RSA suite, baselines, ICLR 2026 paper | ⬜ Not started |

### What's built in `agidb-core` today (inherited from sochdb v1)

The engine crate is real, tested code. Other crates are scaffolded stubs.

| Module | What it does | Phase | Source |
|---|---|---|---|
| `hdc.rs` | 8192-bit hypervector: `bind`, `bundle`, `hamming`, `from_name`, `similarity`, `active_dims` | 1 | sochdb v1 |
| `types.rs` | Domain types: `Episode`, `Triple`, `Concept`, `SemanticAtom`, `Procedure`, `Provenance`, `TimeRange`, `Query`, `Recall`, `RecallMatch`, `SemanticMatch`, `Tier` | 2, 4, 6 | sochdb v1 |
| `error.rs` | `AgidbError` typed error model + `Result<T>` alias | 2 | sochdb v1 (renamed) |
| `signatures.rs` | `SignatureFile` — mmap'd `signatures.dat`, 32-byte header, grow-on-append | 2 | sochdb v1 |
| `store.rs` | `Store` — redb metadata, 8 tables, bi-temporal `observe`/`supersede`, `recall_exact`, jsonl export/import | 2 | sochdb v1 |
| `episode.rs` | Episode encoding, role HVs, `bind_triple`, `encode_episode_signature`, `encode_gist_signature` | 4 | sochdb v1 |
| `recall.rs` | `Store::recall` — the four-tier cascade (A → C → D today; B activates with phase 3) | 4 | sochdb v1 |
| `consolidate.rs` | `Store::consolidate` — clustering, semantic-atom creation, contradiction detection, audit log | 6 | sochdb v1 |

### What gets added in `agidb-core` for v2.0

| Module | What it does | Phase |
|---|---|---|
| `goal.rs` | `Goal`, `GoalState`, `GoalPatch`, `GoalTree`, state-machine validation | 9 |
| `belief.rs` | `Belief`, `BeliefRevision`, `Evidence`, `RevisionReport`, revision math | 9 |
| `sensory.rs` | `SensoryFrame`, ring buffer, surprise scoring | 10 |
| `learning_log.rs` | `LearningEvent` enum, append-only log, introspection queries | 10 |
| `unlearn.rs` | `UnlearnTarget`, cascading removal, tombstones, audit | 11 |
| `neurosymbolic.rs` | bidirectional translation, weighted hybrid queries | 12 |

### What gets added for v2.1

| Crate / Module | What it does | Phase |
|---|---|---|
| `agidb-sensory::vjepa.rs` | V-JEPA 2 ONNX runtime, 64-frame video → 1024d latent | 14 |
| `agidb-sensory::wav2vec.rs` | Wav2Vec-BERT 60s audio → 1024d latent | 14 |
| `agidb-sensory::llama.rs` | Llama-3.2-3B text → 2048d latent | 14 |
| `agidb-sensory::project.rs` | Charikar 2002 thresholded random projection → 8192-bit HV | 14 |
| `agidb-sensory::multimodal.rs` | Role-filler binding for video+audio+text → episode HV | 14 |
| `agidb-core::surprise.rs` | Brain-calibrated surprise threshold + calibration API | 15 |
| `agidb-bams` | BAMS benchmark crate, six-network RSA harness | 16 |

### Test status (inherited)

**44 tests passing, 1 ignored** (`cargo test -p agidb-core`):
- 3 — `types` unit tests
- 13 — `hdc_properties` — HDC algebra invariants
- 9 — `storage_properties` — round-trips, supersession (1 ignored: alias resolution, phase 3)
- 11 — `recall_properties` — tier A/C/D, fall-through, `as_of`
- 8 — `consolidate_properties` — clustering, atom creation, contradictions

Plus a criterion benchmark (`hdc_scan`): 8192-bit hamming scan over 100k signatures — ~5.4ms on a Zen 4 laptop (portable path; M2 NEON path is the official target).

### What you can do with the engine today

`observe` a pre-extracted episode (text + triples) → it's signed, indexed, and bi-temporally stamped. `recall` a cue → tier-A exact concept lookup, tier-C gist similarity, tier-D nearest-neighbor fallback (recall never returns empty). `consolidate` → repeated episodes cluster into semantic atoms, contradictions get superseded. `supersede`, `export_jsonl`, `import_jsonl` all work.

### What's not built yet

- **Raw-text extraction** (phase 3) — blocks tier B and alias resolution
- **MCP server + Python bindings** (phase 5)
- **Cognitive primitives** (phase 9) — goals, beliefs
- **Sensory buffer + self-model** (phase 10)
- **Unlearn API** (phase 11)
- **Neurosymbolic interface** (phase 12)
- **Benchmark harnesses** (phases 7 and 13)
- **Multimodal sensory encoders** (phase 14) — V-JEPA 2, Wav2Vec-BERT
- **Brain-calibrated surprise** (phase 15)
- **BAMS suite** (phase 16)
- **Decay + compaction** in consolidation
- **Background consolidation scheduler**

---

## 5. Architecture

agidb is built in **three engineering layers** that implement **seven cognitive floors**. The user only ever interacts with the floors via the public API; the engineering layers are invisible.

### The three engineering layers

```
┌──────────────────────────────────────────────────────────────┐
│  LAYER 1 — RECALL                                            │
│  The mind-like layer. HDC signatures, binding, bundling,     │
│  hamming-distance retrieval, tiered confidence, goal-biased  │
│  weighting. This is what the user experiences.               │
└──────────────────────────────────────────────────────────────┘
                              ▲
┌──────────────────────────────────────────────────────────────┐
│  LAYER 2 — EXTRACTION                                        │
│  The scaffolding. v2.0: GLiNER ONNX entity/relation          │
│  extraction, belief extraction, time-anchor parsing.         │
│  v2.1: V-JEPA 2 (video), Wav2Vec-BERT (audio),               │
│  Llama-3.2-3B (text). All project to 8192-bit HV.            │
└──────────────────────────────────────────────────────────────┘
                              ▲
┌──────────────────────────────────────────────────────────────┐
│  LAYER 3 — STORAGE                                           │
│  The plumbing. redb for metadata + bi-temporal indexes.      │
│  mmap'd flat files for signatures. Append-only logs for the  │
│  self-model audit trail. Crash-safe, ACID, Rust.             │
└──────────────────────────────────────────────────────────────┘
```

### The seven cognitive floors

The biological mapping. These are *what* the user experiences and *what* agidb stores. The three engineering layers above are *how* it's built.

| Floor | What it is | agidb implementation |
|---|---|---|
| **1. Sensory** | Raw signal buffer, sub-second residence, surprise-gated | `SensoryFrame` ring buffer in redb, promoted to episodic on surprise > threshold. v2.1: multimodal (video + audio + text) via V-JEPA 2, Wav2Vec-BERT, Llama-3.2-3B |
| **2. Working** | Active context, ~7 items, session-scoped | Session ID + recency boost on top of episodic retrieval |
| **3. Episodic** | Events with time/place/people | `Episode` with bi-temporal stamps and HDC signature |
| **4. Semantic** | Facts decoupled from when learned | `SemanticAtom` produced by consolidation worker |
| **5. Procedural** | Workflows, skills, with execution traces | `Procedure` with success counts and `ExecutionTrace` log |
| **6. Goals + Beliefs** | What the agent wants and thinks is true | `Goal` (state machine) + `Belief` (revisable with audit) |
| **7. Self-model** | Audit log of every learning event | `LearningEvent` append-only log + `attention_trace` + self-vector EMA |

### Why three layers and not one

A biological brain stores memories as activation patterns in cortical neurons — extraction is implicit in perception, and synaptic weights *are* the storage. We don't have 86 billion neurons on a laptop, so we simulate brain-like *behaviour* with conventional parts: layer 1 simulates retrieval-by-overlap, layer 2 prepares input so signatures are robust to phrasing, layer 3 persists reliably with crash safety. The user gets brain-like behavior; the engineering does the work.

### Why seven floors and not three

The three engineering layers tell you *how* agidb is built. The seven cognitive floors tell you *what* it stores and what shapes the API takes. Three is a system-design abstraction; seven is a cognitive-science abstraction grounded in 50 years of memory research (Tulving, Squire, Baddeley, McClelland). Both are required to understand the system fully.

### The write path — `observe()` (v2.0)

```
USER  db.observe("Sarah recommended Bawri in Bandra last weekend")
  │
  ▼  FLOOR 1 — SENSORY BUFFER (lightweight)
  │  1. record raw text in sensory ring buffer with timestamp
  │  2. compute surprise score against current beliefs
  │  3. if surprise > threshold OR explicit observe() call: promote
  │
  ▼  LAYER 2 — EXTRACTION
  │  1. GLiNER ONNX (~150ms CPU) extracts entities + relations
  │  2. resolve "last weekend" → 2026-05-09 (valid time)
  │  3. attach confidence scores; canonicalize entity names
  │  4. optionally extract beliefs (high-confidence claims)
  │
  ▼  LAYER 1 — BINDING
  │  1. look up / assign 8192-bit HVs per concept
  │  2. bind triples into role-filler patterns:
  │     triple = (SUBJ⊗Sarah) ⊕ (PRED⊗recommended) ⊕ (OBJ⊗Bawri)
  │  3. bundle triples into one episode signature
  │  4. also compute a raw-text gist signature
  │
  ▼  LAYER 3 — STORAGE
  │  1. append 1KB signature to signatures.dat
  │  2. write episode row to redb (text, offset, triples,
  │     bi-temporal stamps, provenance, confidence)
  │  3. update inverted index + concept index
  │  4. emit LearningEvent::EpisodeStored to floor 7 log
  │  5. fsync, return EpisodeId
  │
  ▼
USER gets EpisodeId
```

### The write path — `observe_multimodal()` (v2.1)

```
USER  db.observe_multimodal(video_clip, audio_clip, "sarah said bawri")
  │
  ▼  FLOOR 1 — SENSORY BUFFER (multimodal)
  │  1. record raw modalities in sensory ring buffer
  │  2. for each modality, run frozen encoder:
  │     - video → V-JEPA 2 → 1024d
  │     - audio → Wav2Vec-BERT → 1024d
  │     - text  → Llama-3.2-3B → 2048d
  │  3. project each latent to 8192-bit HV via thresholded random projection
  │  4. compute brain-aligned surprise: hamming distance from
  │     predicted-next-signature, threshold calibrated to TRIBE v2
  │  5. if surprise > θ_brain: promote to episodic
  │
  ▼  LAYER 1 — MULTIMODAL BINDING
  │  episode = ROLE_video ⊕ sig_video
  │         XOR ROLE_audio ⊕ sig_audio
  │         XOR ROLE_text  ⊕ sig_text
  │         XOR ROLE_goal  ⊕ sig_goal
  │         XOR ROLE_belief ⊕ sig_belief
  │         XOR ROLE_time  ⊕ sig_time
  │
  ▼  LAYER 3 — STORAGE (same as v2.0)
  │
  ▼
USER gets EpisodeId
```

### The read path — `recall()`

The four-tier cascade. Recall falls through tiers until it finds matches or hits `tier_floor`. **It never returns the empty set.** With agidb v2, retrieval is also *goal-biased*: active goals up-weight relevant matches.

```
USER  db.recall("what thai place did sarah mention?")
  │
  ▼  LAYER 2 — PARTIAL EXTRACTION (if natural-language cue)
  │  Extract partial triple shape from the cue
  │
  ▼  LAYER 1 — TIERED RETRIEVAL
  │
  ▼  TIER A — EXACT     canonical entity match via concept index
  │                     confidence 1.0
  │
  ▼  TIER B — SIMILARITY  HDC structured-signature similarity,
  │                       POPCOUNT over inverted-index intersection
  │                       confidence band [0.6, 0.95]
  │
  ▼  TIER C — GIST     raw-text gist signature similarity
  │                    confidence band [0.3, 0.6]
  │
  ▼  TIER D — NEAREST  best-effort nearest neighbours,
  │                    low_confidence flag, confidence ≤ 0.3
  │
  ▼  GOAL-BIAS REWEIGHTING
  │  If active goals exist, up-weight matches semantically
  │  related to those goals (HDC similarity to goal signatures).
  │
  ▼  LAYER 3 — HYDRATION
  │  fetch rows, apply bi-temporal `as_of` filter,
  │  apply supersession filter, attach provenance + beliefs
  │
  ▼  FLOOR 7 — ATTENTION TRACE
  │  emit LearningEvent::AttentionTraced with which signatures
  │  were activated and rejected
  │
  ▼
USER gets Recall { matches, semantic_atoms, beliefs, tier_used, elapsed_ms, attention_trace }
```

Target: **p95 under 50ms** on a laptop with 100k episodes. No network calls. No API keys. No LLM in the read path.

### The consolidation loop — `consolidate()`

Runs in the background (default: every 5 minutes when idle) or on demand. The analog of sleep, extended with surprise-gating and learning-log emission.

```
1. SCAN         scan recent episodic signatures (last 7 days)
2. SURPRISE     compute surprise score for each episode against
                current beliefs and semantic atoms
3. CLUSTER      cluster by hamming distance (similarity ≥ 0.95)
                clusters of N ≥ 3 are consolidation candidates
4. ATOMS        bundle each cluster into a SemanticAtom with
                evidence_count = N and links back to episodes
5. BELIEFS      promote high-confidence semantic atoms to beliefs
                (evidence ≥ 5, no contradictions)
6. CONTRADICT   same (subject, predicate), overlapping valid time,
                different object → older fact superseded
                emit BeliefRevision for affected beliefs
7. DECAY        unreferenced atoms (90 days) decay by factor λ
8. COMPACT      rewrite signatures.dat to drop archived entries
9. AUDIT        emit LearningEvents for every consolidation action
10. SELF-VECTOR (v2)  update self-model EMA: self_vector ← (1-α) self_vector + α bundle(consolidated_atoms)
```

### The unlearn loop — `unlearn()`

Non-destructive cascading removal. Critical for compliance (GDPR Article 17), security (poisoned memories), and trust.

```
USER  db.unlearn(UnlearnTarget::Concept("Sarah"), "user requested forget")
  │
  ▼  IDENTIFY CASCADE
  │  Find all episodes, beliefs, semantic atoms, procedures
  │  referencing the target. Compute the dependency graph.
  │
  ▼  TOMBSTONE (non-destructive)
  │  Mark each affected row with t_tombstoned = now.
  │  Signatures invalidated in mmap (compacted later).
  │  Concept HV marked withdrawn.
  │
  ▼  CASCADE
  │  Beliefs whose evidence drops below threshold → confidence
  │  reduced or belief withdrawn. Affected semantic atoms
  │  recomputed without removed evidence.
  │
  ▼  SELF-VECTOR SUBTRACTION (v2)
  │  self_vector ← (self_vector - α · bundle(tombstoned_signatures))
  │  Otherwise the self-model still "remembers" the unlearned
  │  thing as centroid contamination. Real unlearn means
  │  subtracting from the self-model too.
  │
  ▼  AUDIT
  │  Emit LearningEvent::Unlearned with target_ref, cascade_size,
  │  reason, and the full dependency graph that was tombstoned.
  │
  ▼
USER gets UnlearnReport { episodes, beliefs, atoms, procedures, audit_id }
```

### Key design choices

| Choice | Alternative | Why |
|---|---|---|
| HDC signatures as the primary representation | dense embeddings | deterministic, no model dependency, 8× smaller, POPCOUNT-fast, encoder-free |
| Goals + beliefs as first-class types | text-stored in episodes | enables typed retrieval, state-machine semantics, revision audit |
| GLiNER for extraction | LLM-based extraction | local, no API key, no hallucination at write time |
| **V-JEPA 2 for video sensory (v2.1)** | dense pixel models | latent-space prediction philosophy, 1.2B params, SOTA Something-Something v2, used by TRIBE v2 (brain-alignment free) |
| **Wav2Vec-BERT for audio sensory (v2.1)** | whisper | TRIBE v2 uses Wav2Vec-BERT, alignment via shared encoder |
| **Charikar 2002 random projection (v2.1)** | learned quantization | deterministic, training-free, JL distance preservation guarantee |
| **VSA role-filler binding for multimodal (v2.1)** | attention fusion | factorable — can recover individual modality components from stored episode |
| LLM allowed at write time (belief revision) | no LLM ever | belief revision needs semantic judgment beyond pure math |
| redb for metadata | sqlite, rocksdb, sled | pure Rust, ACID, MVCC, no FFI |
| mmap'd flat files for signatures | signatures in redb | fixed-size, POPCOUNT-scanned in bulk |
| bi-temporal supersession | overwrite on update | preserves history, enables "as of", auditable |
| non-destructive unlearn with audit | hard DELETE | compliance, trust, recoverability |
| tiered recall with explicit confidence | binary hit/miss | graceful degradation; agent always gets something |
| goal-biased retrieval | uniform retrieval | matches cognitive function — agents attend to what they want |
| **brain-calibrated surprise gating (v2.1)** | hand-tuned threshold | grounded in 720-subject fMRI data, defensible in papers |
| **Self-vector EMA in self-model (v2)** | learning log only | inspired by V-JEPA 2 target encoder + TRIBE per-subject layer |
| single binary, embedded | client-server | runs offline, no infra, sqlite-like deployment |
| Rust top to bottom | Python + Rust extensions | sub-50ms p95, no GC, fits embedded-DB ecosystem |
| MCP server as a first-class interface | REST API | reaches agents directly, no glue code |
| Self-model audit log | no introspection | enables "what did I learn?" — the agent can reason about itself |

---

## 6. The data model

The types stored by layer 3 and exposed through the public API. Defined in `crates/agidb-core/src/types.rs` and the new modules.

### Carried forward from sochdb v1

- **`HV`** — a binary hypervector, 8192 bits / 1024 bytes, 64-byte aligned
- **`Episode`** — a stored observation with bi-temporal stamps and HDC signature
- **`Triple`** — a `(subject, predicate, object)` tuple with confidence and source-episode back-reference
- **`Concept`** — a canonical entity with deterministic HV, canonical name, aliases, type
- **`SemanticAtom`** — a consolidated fact bundled from multiple episodes with `evidence_count` and source-episode provenance
- **`Procedure`** — procedural memory: typed shape for a workflow/skill
- **`Provenance`** — write attribution: source, session_id, trace_id, metadata
- **`TimeRange`** — closed-open valid-time interval `[start, end)`
- **`Query` / `Recall` / `RecallMatch` / `SemanticMatch`** — retrieval request and result types
- **`Tier`** — `Exact` / `Similarity` / `Gist` / `NearestNeighbor`

### Added in agidb v2.0

- **`Goal`** — id, parent_id, description, state (`Active`/`Paused`/`Completed`/`Abandoned`), success criteria, deadlines, HDC signature, provenance
- **`Belief`** — claim, subject, confidence, evidence (episodes), contradictions, revision_log, bi-temporal stamps, HDC signature
- **`BeliefRevision`** — timestamp, previous_confidence, new_confidence, triggering_evidence, reason
- **`SensoryFrame`** — raw text or multimodal blob ref, modality, received_at, surprise_score, optional promotion link
- **`Modality`** — `Text` / `Image { path }` / `Audio { path }` / `Video { path }` / `Multimodal { components }`
- **`LearningEvent`** — enum of every introspectable event the system records
- **`AttentionTrace`** — per-recall record of which signatures activated and why
- **`UnlearnTarget`** — `Episode` / `Belief` / `Concept` / `BySource` / `BySession`
- **`UnlearnReport`** — cascade summary with audit log reference
- **`SelfVector`** — slowly drifting 8192-bit hypervector representing the agent's current self-model centroid

### Added in agidb v2.1

- **`MultimodalEncoding`** — `{ video: Option<HV>, audio: Option<HV>, text: Option<HV> }` — modality-tagged signatures
- **`EncoderConfig`** — versioned config for V-JEPA 2 / Wav2Vec-BERT / Llama-3.2-3B (model hashes, weights paths, projection matrices)
- **`BrainCalibration`** — surprise threshold, fitted parameters, calibration dataset reference, TRIBE v2 version
- **`BamsScore`** — six-cortical-network RSA score breakdown

### Updated for v2

The `Recall` struct now includes goal-bias metadata and belief context:

```rust
pub struct Recall {
    pub matches: Vec<RecallMatch>,
    pub semantic_atoms: Vec<SemanticMatch>,
    pub beliefs: Vec<BeliefMatch>,           // new in v2
    pub active_goals: Vec<GoalId>,           // new in v2 — which goals biased this
    pub tier_used: Tier,
    pub elapsed_ms: u32,
    pub attention_trace: Option<AttentionTrace>,  // new in v2
}
```

### The on-disk layout (v2.1)

An agidb v2.1 store is a directory containing:

```
memory.agidb/
├── meta.redb              redb database — 16 tables in v2.1:
│                            Inherited from sochdb v1:
│                              episodes, concepts, concept_by_name,
│                              concept_episodes (multimap),
│                              inverted_index, semantic_atoms,
│                              consolidation_log, manifest
│                            New in agidb v2.0:
│                              goals, beliefs, belief_revisions,
│                              sensory_buffer, learning_events,
│                              tombstones
│                            New in agidb v2.1:
│                              self_vector_history,
│                              encoder_versions
├── signatures.dat         mmap'd flat file — 1024-byte HV slots
├── manifest.toml          format version, hyperparams, schema version,
│                          encoder config hashes
├── audit.log              append-only signed audit log (optional, v0.3+)
└── encoders/              v2.1: ONNX weights, projection matrices
    ├── vjepa2-gigantic-256.onnx
    ├── wav2vec-bert-2.0.onnx
    ├── llama-3.2-3b.onnx
    └── projections.bin    seeded random projection matrices
```

Bi-temporal columns live on every fact. The on-disk format is versioned via `format_version` in the manifest; migrations are explicit, never silent. **agidb v2 can read sochdb v1 files directly** (the v1 schema is a subset of v2). **agidb v2.1 can read v2.0 files** (v2.0 stores have no multimodal signatures, those tables are simply empty).

---

## 7. The build roadmap

The 12-month plan: agidb v2.0 substrate at month 9, agidb v2.1 brain-alignment + BAMS at month 12. Decision gate at week 12 binding.

### Guiding principles

- **Inherit, don't rewrite.** Every line of sochdb v1's working code carries forward.
- **Ship small.** v2.0 is the smallest credible AGI substrate. v2.1 adds brain-alignment on top.
- **Benchmark honestly.** Every claim is reproducible; publish raw logs. Include cognitive benchmarks. Include BAMS.
- **Defer hosting.** Stay embedded; cloud is v0.5+ territory.
- **Decision gate at week 12.** If the numbers don't justify the bet, reposition.

### Phases inherited from sochdb v1 (carry forward)

Phases 0, 1, 2, 4, 6 already complete. Phases 3, 5 remain on the critical path.

### Phase 3 — Extraction (sochdb v1 carryover) ⬜

Vendor / port the GLiNER ONNX loading + inference code from ctxgraph. The `Extraction` pipeline: entities, relations, time anchors, confidence propagation, alias resolution, predicate canonicalization. **New for v2:** belief extraction.

**Exit criterion:** `observe()` extracts triples from 20 sample observations with >85% F1 against a human-labelled gold set. Belief extraction extracts >70% of explicit beliefs correctly. Unlocks tier B and alias resolution.

### Phase 5 — MCP + Python bindings (sochdb v1 carryover) ⬜

`agidb-mcp` MCP server, `agidb-py` pyo3 bindings with async support, Python wheels.

**Exit criterion:** Claude Desktop uses agidb as a memory tool via MCP; `pip install agidb` works.

### Phase 7 — Decision gate ⬜

Run benchmarks against Mem0, Zep/Graphiti, Letta, MemMachine on **LongMemEval-S + LoCoMo + BEAM** using a shared harness, publishing the full six-metric stack plus cognitive benchmark results.

### New phases for the AGI pivot (v2.0)

### Phase 9 — Cognitive primitives ⬜

Implement `Goal` and `Belief` as first-class types. New redb tables. Belief revision math. Goal state machine validation.

**Exit criterion:** can `set_goal`, `revise_goal`, `assert_belief`, `revise_belief`, `what_do_i_believe`, retrieve goals and beliefs through unified recall. 100-step goal mutation test passes. Belief revision audit log captures every change.

**Timeline:** weeks 13-18.

### Phase 10 — Sensory + self-model ⬜

Implement `SensoryFrame` ring buffer with surprise gating. Extend `consolidation_log` into `learning_events` log. Add `attention_trace` to recall path. Implement `what_did_i_learn` introspection API. **Add `self_vector` EMA in the self-model floor.**

**Exit criterion:** sensory buffer ingests 1000 frames/sec; surprise gating promotes only ~5% to episodic. Learning log captures every state change. Attention traces survive recall round-trips. Self-vector drifts with consolidation, subtracts on unlearn.

**Timeline:** weeks 19-22.

### Phase 11 — Unlearn API ⬜

Implement `unlearn(target, reason)` with cascading removal. Tombstone model. Cascade through dependent beliefs, semantic atoms, procedures. **Self-vector subtraction.** Audit log retention permanent.

**Exit criterion:** unlearning a concept removes all references within 100ms. Audit log permanent. Self-vector verifiably no longer contains the unlearned concept.

**Timeline:** weeks 23-25.

### Phase 12 — Neurosymbolic interface ⬜

Expose the implicit signature ↔ triple translation as a first-class API. Implement `neurosymbolic_query` with weighted hybrid. Add `signature_to_triples` and `triples_to_signature`.

**Exit criterion:** hybrid queries with 50/50 weights return appropriately blended results.

**Timeline:** weeks 26-27.

### Phase 13 — Cognitive benchmarks ⬜

Build `cognitive-bench` suite. Goal consistency, belief revision, unlearn cascade, multi-floor retrieval tests.

**Exit criterion:** all four cognitive benchmarks pass with documented thresholds.

**Timeline:** weeks 28-30.

### Phase 8 — Hardening + launch (v2.0 substrate ships) ⬜

Expand the harness, fuzz suite, 30-day soak test, arxiv whitepaper, 3 design-partner deployments, launch blog post, demo video, crates.io + PyPI + MCP-registry publication.

**Public v2.0 launch at week 36 (month 9).**

### New phases for v2.1 brain-alignment

### Phase 14 — Multimodal sensory encoders ⬜

New crate `agidb-sensory`. Wrap V-JEPA 2 (Gigantic-256), Wav2Vec-BERT 2.0, Llama-3.2-3B via ONNX or Candle. Implement Charikar 2002 thresholded random projection from each encoder's latent dim to 8192-bit HV. Extend `observe_multimodal()` API.

**Exit criterion:** end-to-end pipeline: 30s video+audio clip → V-JEPA 2 + Wav2Vec-BERT inference → projected to 8192-bit episode HV → stored in redb. P50 latency ≤ 2s per 30s clip on a laptop.

**Timeline:** weeks 37-42 (6 weeks).

### Phase 15 — Brain-calibrated surprise gating ⬜

Build calibration tooling. Download TRIBE v2 weights from HuggingFace. Run TRIBE v2 on paired stimulus dataset. Compute neural surprise as deviation of associative-cortex BOLD from sliding-window baseline. Fit agidb surprise threshold θ_brain to maximize correlation with TRIBE-predicted neural surprise indicator.

**Exit criterion:** calibrated threshold ships in v2.1. Documentation includes reproducible calibration recipe. Comparison plot: pre/post calibration vs neural surprise.

**Timeline:** weeks 43-46 (4 weeks).

### Phase 16 — BAMS benchmark + paper ⬜

Build `agidb-bams` crate. Six-cortical-network RSA harness. Compute TRIBE-predicted RDMs over held-out movies. Compute agidb signature RDMs. RSA. Run baselines: raw V-JEPA 2 latents, mem0, letta, zep/graphiti, hippoRAG.

**Exit criterion:** BAMS score reported for all baselines and agidb. Paper submitted to ICLR 2026 MemAgents workshop or CCN 2026 (whichever has the earlier deadline). Public repo with reproduction kit.

**Timeline:** weeks 47-52 (6 weeks).

### Beyond v2.1

See [agi-trajectory.md](./product/agi-trajectory.md) for the 5-year roadmap.

### Explicitly out of scope (any version)

Distributed/sharded agidb, a query language, a UI, replacing vector DBs for document RAG, replacing graphs for general graph workloads, a fine-tuning service, building AGI itself.

---

## 8. Brain-alignment in v2.1+

**Why it's separate from v2.0:** the substrate has to land first. Brain-alignment is additive, requires GPU-class compute for encoder inference, and depends on cognitive primitives that ship in v2.0 phases 9-13.

**The integration in one paragraph:** v2.1 ships `agidb-sensory`, a Rust crate that wraps Meta's V-JEPA 2 (video, 1.2B params), Wav2Vec-BERT (audio), and Llama-3.2-3B (text). Each encoder outputs a dense latent that gets projected to an 8192-bit signature via Charikar 2002 thresholded random projection (training-free, JL distance preservation guarantee). Multimodal episodes are bound via VSA role-filler binding so individual modality signatures remain factorable. Sensory surprise is computed against the predicted-next-signature, with threshold empirically calibrated against TRIBE v2's predicted fMRI BOLD across 720 subjects.

**Why this is defensible:** because agidb uses the same encoder stack as TRIBE v2, its internal representations can be compared directly against human cortical activations on matched stimuli. This is the brain-alignment benchmark (BAMS) and the first paper-sized contribution from the project.

**See [brain-alignment.md](./architecture/brain-alignment.md) for the full technical detail.**
**See [bams-benchmark.md](./architecture/bams-benchmark.md) for the benchmark protocol and paper plan.**

---

## 9. Feature catalogue

### Storage & durability (inherited from sochdb v1)

| Feature | Status |
|---|---|
| Embedded single-binary store (no server) | ✅ phase 2 |
| redb ACID metadata, 14-table schema (v2.0), 16-table (v2.1) | ✅ phase 2 (8 tables); ⬜ v2.0 (6 more); ⬜ v2.1 (2 more) |
| mmap'd flat-file signatures, grow-on-append | ✅ phase 2 |
| Bi-temporal columns (valid time + transaction time) | ✅ phase 2 |
| Bi-temporal supersession (non-destructive updates) | ✅ phase 2 |
| jsonl export / import | ✅ phase 2 |
| Crash-safety (kill mid-write → recover) | ⬜ phase 2 follow-up |
| Format versioning + explicit migrations | ✅ phase 2 (scaffolded) |
| Sochdb v1 → agidb v2 read compatibility | ✅ implicit (v2 = v1 superset) |
| v2.0 → v2.1 read compatibility | ✅ implicit (v2.1 = v2.0 superset) |
| Encryption at rest | ⬜ v0.3 |
| WAL streaming | ⬜ v0.3 |
| Signed audit log | ⬜ v0.3 |

### Retrieval (mostly inherited, extended in v2)

| Feature | Status |
|---|---|
| HDC kernel — bind / bundle / hamming | ✅ phase 1 |
| AVX-512 + NEON + portable POPCOUNT paths | ✅ phase 1 |
| Episode signature encoding | ✅ phase 4 |
| Gist signatures (raw-text fallback) | ✅ phase 4 |
| Tier A — exact concept match | ✅ phase 4 |
| Tier B — structured HDC similarity | ⬜ phase 3 (needs extraction) |
| Tier C — gist similarity | ✅ phase 4 |
| Tier D — nearest-neighbour fallback | ✅ phase 4 |
| Never-return-empty guarantee | ✅ phase 4 |
| Bi-temporal `as_of` filtering | ✅ phase 4 |
| `tier_floor` / `k` / `min_confidence` controls | ✅ phase 4 |
| Goal-biased retrieval reweighting | ⬜ phase 9 |
| Working-memory session boost + recency | ⬜ phase 10 |
| Belief context in recall results | ⬜ phase 9 |
| Attention trace per recall | ⬜ phase 10 |
| Confidence calibration (ECE ≤ 0.05) | ⬜ phase 4 follow-up |
| LSH for >1M episodes | ⬜ v0.3 |
| **Multimodal episode retrieval (v2.1)** | ⬜ phase 14 |

### Extraction — layer 2

| Feature | Status |
|---|---|
| GLiNER ONNX entity/relation extraction | ⬜ phase 3 |
| Time-anchor parsing | ⬜ phase 3 |
| Alias resolution + concept canonicalization | ⬜ phase 3 |
| Predicate canonicalization | ⬜ phase 3 |
| Belief extraction (claims with confidence) | ⬜ phase 9 |
| Learned predicate similarity | ⬜ v0.3 |
| **V-JEPA 2 video encoder (v2.1)** | ⬜ phase 14 |
| **Wav2Vec-BERT audio encoder (v2.1)** | ⬜ phase 14 |
| **Llama-3.2-3B text encoder (v2.1)** | ⬜ phase 14 |
| **HDC random projection (v2.1)** | ⬜ phase 14 |
| **VSA multimodal binding (v2.1)** | ⬜ phase 14 |

### Consolidation (mostly inherited)

| Feature | Status |
|---|---|
| Clustering by hamming distance | ✅ phase 6 |
| Semantic-atom creation (evidence ≥ 3) | ✅ phase 6 |
| Contradiction detection + supersession | ✅ phase 6 |
| Consolidation audit log | ✅ phase 6 |
| Synchronous `consolidate()` API | ✅ phase 6 |
| Surprise-gated promotion (sensory → episodic) | ⬜ phase 10 |
| Belief promotion from semantic atoms | ⬜ phase 9 |
| Decay of unreferenced atoms | ⬜ phase 6 follow-up |
| Storage compaction | ⬜ phase 6 follow-up |
| Background scheduler (tokio task) | ⬜ phase 6 follow-up |
| **Self-vector EMA update (v2)** | ⬜ phase 10 |

### Cognitive primitives — new in agidb v2

| Feature | Status |
|---|---|
| `Goal` type with state machine | ⬜ phase 9 |
| `set_goal` / `revise_goal` / `active_goals` / `goal_tree` API | ⬜ phase 9 |
| `Belief` type with confidence + evidence + revision log | ⬜ phase 9 |
| `assert_belief` / `revise_belief` / `what_do_i_believe` API | ⬜ phase 9 |
| Belief revision math | ⬜ phase 9 |
| `SensoryFrame` ring buffer | ⬜ phase 10 |
| Surprise scoring against current beliefs | ⬜ phase 10 |
| `observe_sensory` / `working_state` API | ⬜ phase 10 |
| `LearningEvent` enum + append-only log | ⬜ phase 10 |
| `what_did_i_learn` introspection API | ⬜ phase 10 |
| `attention_trace` recording | ⬜ phase 10 |
| `self_vector` EMA in self-model | ⬜ phase 10 |
| `unlearn` API with cascading non-destructive removal | ⬜ phase 11 |
| **Self-vector subtraction on unlearn** | ⬜ phase 11 |
| `UnlearnReport` with audit | ⬜ phase 11 |
| 30-day tombstone recovery window | ⬜ phase 11 |
| Permanent audit log | ⬜ phase 11 |
| Neurosymbolic translation layer | ⬜ phase 12 |
| `neurosymbolic_query` weighted hybrid | ⬜ phase 12 |
| `signature_to_triples` / `triples_to_signature` | ⬜ phase 12 |

### Brain-alignment — new in agidb v2.1

| Feature | Status |
|---|---|
| `agidb-sensory` crate | ⬜ phase 14 |
| V-JEPA 2 ONNX/Candle integration | ⬜ phase 14 |
| Wav2Vec-BERT ONNX/Candle integration | ⬜ phase 14 |
| Llama-3.2-3B integration | ⬜ phase 14 |
| Charikar 2002 random projection to 8192-bit | ⬜ phase 14 |
| `observe_multimodal()` API | ⬜ phase 14 |
| Brain-calibrated surprise threshold | ⬜ phase 15 |
| Calibration tooling (TRIBE v2 → θ_brain) | ⬜ phase 15 |
| `agidb-bams` crate | ⬜ phase 16 |
| Six-cortical-network RSA harness | ⬜ phase 16 |
| BAMS reproduction kit | ⬜ phase 16 |
| ICLR 2026 MemAgents paper | ⬜ phase 16 |

### Interfaces & distribution

| Feature | Status |
|---|---|
| `agidb-core` Rust API | ✅ in progress |
| `agidb` umbrella crate | ✅ scaffold |
| CLI (`agidb` binary) | ⬜ phase 5 |
| MCP server | ⬜ phase 5 |
| Python bindings (pyo3) + wheels | ⬜ phase 5 |
| `cargo add agidb` / `pip install agidb` | ⬜ phase 8 |

---

## 10. Tech stack

**Rust top to bottom in `agidb-core`** (constitution article VIII). ONNX runtime via the `ort` crate is the only permitted FFI for v2.0. v2.1 adds Candle as an additional ML inference backend option.

### Workspace — 11 crates (v2.1 expanded from v2.0's 9)

| Crate | Role | Status |
|---|---|---|
| `agidb` | umbrella crate, re-exports the public API | scaffold |
| `agidb-core` | the engine: HDC, redb, mmap, recall, consolidation, goals, beliefs, sensory, self-model, unlearn | **in progress** |
| `agidb-extract` | GLiNER ONNX wrapper, triple + belief extraction | stub (phase 3) |
| `agidb-ns` | neurosymbolic translation layer | stub (phase 12) |
| `agidb-skills` | procedural execution traces, skill runtime | stub (phase 9+) |
| `agidb-cli` | the `agidb` binary | stub (phase 5) |
| `agidb-mcp` | MCP server | stub (phase 5) |
| `agidb-py` | pyo3 Python bindings | stub (phase 5) |
| `agidb-bench` | benchmark harness vs Mem0/Zep/Letta + cognitive benchmarks | stub (phase 7, 13) |
| `agidb-sensory` | **NEW v2.1** — V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B encoders, HDC projection | stub (phase 14) |
| `agidb-bams` | **NEW v2.1** — brain-aligned memory similarity benchmark harness | stub (phase 16) |

### Key dependencies

| Crate | Purpose |
|---|---|
| `tokio` | async runtime (ADR-0002) |
| `redb` | embedded ACID KV store — pure Rust, MVCC |
| `memmap2` | safe Rust mmap |
| `roaring` | roaring bitmaps for the inverted index |
| `ort` | ONNX runtime for GLiNER (phase 3), V-JEPA 2 + Wav2Vec-BERT (phase 14) |
| `candle-core` | **NEW v2.1** — alternative ML inference backend, pure Rust |
| `candle-transformers` | **NEW v2.1** — transformer ops for Llama-3.2-3B |
| `chrono` | bi-temporal timestamps |
| `serde` + `bincode` | serialization for redb values |
| `serde_json` | jsonl export/import |
| `anyhow` + `thiserror` | error handling |
| `tracing` | structured logging |
| `proptest` + `criterion` | property tests + benchmarks |

MSRV: Rust 1.89. License: Apache-2.0.

### Performance targets (v2.0, on an M2 / i7-12700H benchmark laptop)

| Metric | Target |
|---|---|
| `recall` p50 / p95 / p99 | ≤ 20ms / ≤ 50ms / ≤ 100ms |
| `observe` p50 / p95 | ≤ 100ms / ≤ 200ms |
| 8192-bit hamming scan over 100k signatures | ≤ 5ms |
| `consolidate` (10k episodes) | ≤ 5s |
| `set_goal` / `revise_goal` / `assert_belief` | ≤ 5ms |
| `unlearn` cascade (1000-episode concept) | ≤ 100ms |
| `what_did_i_learn` (last 7 days) | ≤ 50ms |
| Binary size | ≤ 80 MB |
| Memory footprint (1M episodes loaded) | ≤ 250 MB |

### Additional performance targets (v2.1)

| Metric | Target |
|---|---|
| `observe_multimodal` (30s video+audio clip) p50 | ≤ 2s on laptop CPU (V-JEPA 2 is the bottleneck) |
| `observe_multimodal` p50 with GPU | ≤ 500ms (M2 ANE / RTX 4090) |
| V-JEPA 2 inference (64 frames, 256x256) p50 | ≤ 1.5s CPU / ≤ 200ms GPU |
| Wav2Vec-BERT inference (60s audio) p50 | ≤ 400ms CPU / ≤ 80ms GPU |
| Llama-3.2-3B encoder (1024 token window) p50 | ≤ 200ms CPU / ≤ 30ms GPU |
| Random projection 1024d → 8192-bit | ≤ 1ms |
| Multimodal binding (3 modalities → 1 episode HV) | ≤ 200µs |
| Brain-calibrated surprise score | ≤ 500µs |
| BAMS single-movie RSA evaluation | ≤ 30s |
| Binary size (with ONNX weights bundled) | ≤ 4GB |
| Binary size (weights downloaded on first use) | ≤ 100 MB |

---

## 11. The decision gate

**Phase 7, week 12.** Same binding gate as sochdb v1. The entire 9-month bet collapses to one week of benchmarks. The project commits, repositions, or retreats.

### The benchmark suite

| Benchmark | What it tests |
|---|---|
| **LongMemEval-S** | long-context episodic recall accuracy |
| **LoCoMo** | long-conversation memory across 10+ sessions |
| **BEAM** | scale to millions of tokens; contradiction resolution |
| **Cognitive (new in v2)** | goal consistency, belief revision, unlearn cascade, multi-floor retrieval |

Every run publishes **all six metrics plus the cognitive benchmark results** — never a single number: BLEU + F1 + LLM-judge + token cost + p95 latency + noisy-cue degradation + cognitive primitive correctness. Raw logs + the harness commit hash ship with every claim.

### The thresholds

**Commit** (proceed to launch + v2.1 + fundraise) — all of:
- agidb ≥ Zep/Graphiti accuracy on LongMemEval-S (within 1pp F1 *and* LLM-judge)
- ≥ 3× lower p95 retrieval latency than Mem0
- ≥ 3× lower token cost than Mem0 (target < 2,500 tokens/query)
- wins the noisy-cue degradation test
- passes all four cognitive benchmarks (goal, belief, unlearn, multi-floor)
- holds across all three standard benchmarks (no cherry-picking)

**Reposition** (ship smaller, no fundraise) — within 3pp of Mem0 F1 *and* ≥ 10× memory savings, even if cognitive benchmarks are partial. Reposition as "agidb-lite: embedded cognitive memory for edge agents." Skip v2.1.

**Retreat** (fold back into ctxgraph) — more than 10pp behind dense baselines and the gap doesn't close with reranking. Reposition as "ctxgraph: temporal graph memory for agents."

### What success looks like at 9 months (v2.0)

- Match/beat Zep/Graphiti on LongMemEval-S (≥ 64 accuracy)
- ≥ 3× lower retrieval latency than Mem0 (p95 < 50ms)
- ≥ 3× lower token cost than Mem0 (< 2,500 tokens/query)
- All four cognitive benchmarks pass
- 1M+ episodes on a laptop with sub-100ms p99 recall
- `cargo add agidb` + `pip install agidb` both work
- An MCP server any MCP-compatible agent can use
- 1000+ GitHub stars, 5+ design-partner deployments
- arxiv whitepaper posted

### What success looks like at 12 months (v2.1)

- v2.0 success criteria all hold
- Multimodal `observe_multimodal()` API ships
- Brain-calibrated surprise threshold released with reproducible calibration recipe
- BAMS benchmark harness published with baselines (mem0, letta, zep, hippoRAG, raw V-JEPA latents)
- ICLR 2026 MemAgents workshop paper accepted (or CCN 2026, whichever lands first)
- agidb wins BAMS in associative-cortex networks (DMN, dorsal attention, frontoparietal)
- 5000+ GitHub stars
- 10+ design-partner deployments
- Seed round closed

If these are met, agidb is a category-leading substrate. If not, the brain-alignment work folds back into a v2.2 retry or gets abandoned in favor of pure-substrate evolution.

---

## 12. The 5-year trajectory

The full roadmap from agidb v2.0 (substrate, 2026) to v2.5 (AGI-grade, 2031). Detailed in [agi-trajectory.md](./product/agi-trajectory.md). Summary:

| Version | Year | What it adds |
|---|---|---|
| **v2.0** | 2026 (m9) | Substrate — episodic, semantic, procedural, working, sensory, goals, beliefs, self-model, unlearn, neurosymbolic interface |
| **v2.1** | 2026 (m12) | Brain-alignment — multimodal sensory via V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B, brain-calibrated surprise, BAMS benchmark + paper |
| **v2.2** | 2027 | Cognitive engine v0.1 — pattern completion via Hopfield, belief revision with formal AGM semantics, analogical retrieval via HDC binding |
| **v2.3** | 2028 | Causal layer — causal claim storage with intervention semantics, world model fragments, on-line learning state |
| **v2.4** | 2029-2030 | Production-grade — full enterprise tier, distributed mode, formal safety guarantees on self-modification, BCI input experimental (Brain-JEPA, signal-JEPA) |
| **v2.5** | 2031 | AGI-grade — the substrate for true autonomous systems; closed-loop self-modification, causal reasoning over learned beliefs, the cognitive engine fully realized |

This is a 5-year commitment. The 12-month v2.1 launch is the first major milestone, not the whole project.

---

## 13. How to navigate the project

### Docs

| Path | What |
|---|---|
| [`README.md`](../README.md) | the user-facing pitch |
| [`docs/PROJECT.md`](./PROJECT.md) | **this file** — the master reference |
| [`docs/overview.md`](./product/overview.md) | product overview, audiences, comparisons |
| [`docs/architecture.md`](./architecture/architecture.md) | architecture, 3 layers + 7 floors |
| [`docs/biological-mapping.md`](./product/biological-mapping.md) | the seven floors of cognitive memory |
| [`docs/layer-1-recall.md`](./architecture/layer-1-recall.md) | HDC math, retrieval, goal-biased recall |
| [`docs/layer-2-extraction.md`](./architecture/layer-2-extraction.md) | GLiNER + V-JEPA 2 + Wav2Vec-BERT, triples, belief extraction |
| [`docs/layer-3-storage.md`](./architecture/layer-3-storage.md) | redb schema, mmap, on-disk layout |
| [`docs/cognitive-primitives.md`](./architecture/cognitive-primitives.md) | goals, beliefs, sensory, self-model, unlearn |
| [`docs/neurosymbolic.md`](./architecture/neurosymbolic.md) | signature ↔ symbolic interface |
| [`docs/brain-alignment.md`](./architecture/brain-alignment.md) | **new** — V-JEPA 2 + TRIBE v2 integration, brain-calibrated surprise |
| [`docs/bams-benchmark.md`](./architecture/bams-benchmark.md) | **new** — the brain-aligned memory similarity benchmark, paper plan |
| [`docs/tech-spec.md`](./spec/tech-spec.md) | full Rust API, traits, performance targets |
| [`docs/agi-trajectory.md`](./product/agi-trajectory.md) | the 5-year roadmap v2.0 → v2.5 |
| [`docs/roadmap.md`](./product/roadmap.md) | week-by-week phase plan, 1-52 (extended for v2.1) |
| [`docs/CONSTITUTION.md`](./spec/constitution.md) | the immutable principles |

### The constitution in one breath

18 immutable principles (14 inherited from sochdb v1, 3 new for v2.0, 1 new for v2.1). See [CONSTITUTION.md](./spec/constitution.md). Anything that contradicts the constitution needs an ADR.

### Build & test

```bash
cargo build --workspace                          # all 11 crates
cargo test  -p agidb-core                        # 44+ tests
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p agidb-core --bench hdc_scan       # the 100k hamming scan
cargo bench -p agidb-bench --bench cognitive     # v2.0 cognitive benchmarks
cargo bench -p agidb-bams  --bench bams_full     # v2.1 BAMS suite
```

### The immediate next steps

1. **Rename + push to GitHub** — create the `agidb/agidb` org/repo. transfer the sochdb v1 commits. configure the remote. push the rebranded codebase.
2. **Buy agidb.ai + agidb.dev + agidb.io + agidb.co** — lock the namespace today.
3. **Reserve `agidb` on crates.io, npm, PyPI** — lock the package names.
4. **Phase 3 — extraction** — vendor GLiNER ONNX from ctxgraph; unlocks raw-text `observe`, tier B, and alias resolution.
5. **Phase 5 — MCP + Python** — make the engine consumable from Claude Desktop and `pip install`.
6. **Phase 9 — cognitive primitives** — implement `Goal` and `Belief` as first-class types. **This is the v2.0 wedge.**
7. **Phase 7 — the benchmark harness** — build `agidb-bench`, run the decision gate at week 12.
8. **Phase 14-16 — v2.1 brain-alignment** — only after v2.0 decision gate passes. ship V-JEPA 2 sensory + BAMS + paper by month 12.

---

*agidb v2 is pre-alpha. The one-liner: the cognitive substrate for autonomous AI agents — content-addressable hyperdimensional memory, first-class goals and beliefs, bi-temporal supersession, sleep-like consolidation, and a non-destructive unlearn primitive. One Rust binary, one API, no query language. The database AGI systems will run on top of. v2.1 extends with brain-aligned multimodal sensory and the BAMS benchmark.*
