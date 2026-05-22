# agidb domain glossary

> The vocabulary every doc, comment, identifier, test name, and PR title should use. If a term you need isn't here, either add it (one-line PR) or you're inventing language the project doesn't use — reconsider.
>
> This file is consumed by the Matt Pocock skills (`improve-codebase-architecture`, `diagnose`, `tdd`, `grill-with-docs`) per [`docs/agents/domain.md`](./docs/agents/domain.md). Drift between the glossary and the code is a signal for `/grill-with-docs`.
>
> **Scope:** this glossary covers **agidb v2** — the cognitive-substrate vocabulary. Terms marked _(v2.1)_ ship in the brain-alignment milestone and are gated on the week-12 decision gate.

## Core nouns — the memory model

**HV (Hypervector)** — A fixed-size binary vector of [`D = 8192`] bits / [`D_BYTES = 1024`] bytes, cache-line-aligned. The unit of representation in layer 1. Two HVs derived from unrelated names are uncorrelated in expectation (hamming ≈ D/2).

**Episode** — A single observation as stored by agidb. Carries the raw text, an HDC signature offset (into `signatures.dat`), an extracted list of Triples, bi-temporal stamps (`t_valid_start`, `t_valid_end`, `t_tx_start`), a `superseded_by` link if applicable, a Provenance record, and a confidence score. Identified by an `EpisodeId`.

**Triple** — A `(subject, predicate, object)` tuple extracted from natural-language input by layer 2. Each triple carries its own confidence score and a back-reference to its source Episode.

**Concept** — A canonical entity (e.g. "Sarah", "Bawri") with a deterministic HV (via `HV::from_name`), a canonical name, a list of aliases, and an entity type. Identified by a `ConceptId`. Concepts are how agidb canonicalizes mentions across episodes — "Sarah said X" and "sarah_kelly mentioned X" land on the same `ConceptId`.

**SemanticAtom** — A consolidated fact produced by the consolidation loop. Bundles repeated episodic patterns (N ≥ 3) into one durable statement with `evidence_count`, a list of source `EpisodeId`s for provenance, a confidence score, and a `last_referenced` timestamp. Identified by an `AtomId`.

**Procedure** — Procedural memory. A typed Episode shape describing a workflow or skill: name, description, trigger, preconditions, ordered steps (each with optional tool + args), postconditions. Answers "how do I do X?" the way Episodes answer "when did I do X?" and SemanticAtoms answer "what is X?"

**Provenance** — Attribution for a write. Records `source` (`"user"`, `"agent"`, `"tool:gmail"`, …), optional `session_id`, optional `trace_id`, and freeform metadata. Every Episode, SemanticAtom, Goal, and Belief traces back to a Provenance record.

## Cognitive primitives (v2)

The five typed shapes that make agidb a cognitive substrate rather than a memory database. Each is a Rust type, has its own redb table, and is property-tested. See [`docs/architecture/cognitive-primitives.md`](./docs/architecture/cognitive-primitives.md).

**Goal** — Floor 6. What the agent wants. Carries `description`, a `GoalState`, an optional `parent_id` (goals form a tree), `success_criteria`, an optional `deadline`, and an HDC `signature`. First-class so retrieval can be goal-biased and state transitions are auditable. Identified by a `GoalId`. Constitution article XV.

**GoalState** — A goal's lifecycle: `Active` → `Paused` → `Active`, or `Active`/`Paused` → `Completed` / `Abandoned`. `Completed` and `Abandoned` are terminal. Every transition emits `LearningEvent::GoalStateChanged`.

**Belief** — Floor 6. What the agent thinks is true. A graded, revisable claim: `claim` text, `(subject, predicate, object)`, a `confidence` in `[0.0, 1.0]`, supporting `evidence` and `contradictions` (both lists of `EpisodeId`), an append-only `revision_log`, an HDC `signature`, and bi-temporal stamps. Identified by a `BeliefId`. Differs from a fact: facts are atomic and get superseded; beliefs are graded and get *revised* with reasons. Constitution article XVII.

**BeliefRevision** — One entry in a Belief's append-only `revision_log`: timestamp, `previous_confidence`, `new_confidence`, `triggering_evidence`, and a `reason`. Replaying the log reconstructs the current confidence. New evidence drives a Bayesian-style update; an LLM may be consulted *at write time* to judge contradiction (constitution article IV amendment) — the read path stays LLM-free.

**SensoryFrame** — Floor 1. Raw input before promotion to episodic memory. Carries a `Modality`, the `data` (inline text or a blob ref), `received_at`, a `surprise_score`, and `promoted_to` (set once promoted). Lives in the sensory ring buffer. Identified by a `SensoryId`.

**Surprise** — `surprise(frame) = 1 - similarity(frame_signature, bundle(recent_beliefs))`. The gate on promotion: a frame promotes to episodic memory only if its surprise exceeds the threshold (v2.0 default `0.4`; v2.1 brain-calibrated `θ_brain`). The agent's attentional filter — how it decides what's worth remembering.

**SelfVector** — Floor 7. A slowly-drifting 8192-bit HV representing "what kind of agent am I right now." EMA-updated each consolidation epoch: `self_vector ← (1-α)·self_vector + α·bundle(consolidated_atoms)` with `α ≈ 0.05`. Unlearn *subtracts* from it (see Unlearn) so forgotten data leaves no centroid contamination.

**LearningEvent** — Floor 7. One entry in the append-only `learning_events` audit log — every introspectable state change (episode stored, goal state changed, belief asserted/revised/withdrawn, sensory frame promoted, semantic atom formed, contradiction detected, unlearned, attention traced, self-vector updated, consolidation run). A **closed enum**: new variants require an ADR.

**AttentionTrace** — Floor 7. The record of which signatures a `recall()` considered and why — each candidate's `similarity`, `goal_bias`, `recency_boost`, `final_confidence`, and whether it was retained or rejected. Emitted when `Query.trace_attention` is set. Lets the agent answer "what was I attending to during recall X?"

**Unlearn** — A cross-floor, cascading, **non-destructive** removal operation. Computes the dependency cascade for a target, tombstones the affected rows, cascades through dependent beliefs / semantic atoms / procedures, subtracts the removed signatures from the SelfVector, and emits a permanent `LearningEvent::Unlearned`. The difference between *hiding* data and *forgetting* it. Constitution articles XII and XVI.

**UnlearnTarget** — What to forget: an `Episode`, `Belief`, `Concept`, `BySource` (e.g. a GDPR request), `BySession` (a whole conversation), or `Pattern` (anything matching criteria).

**Tombstone** — A non-destructive removal marker. An unlearned row gets `t_tombstoned = now` instead of being deleted; it is recoverable within the 30-day window via `restore_within_window`, and physically compacted out after expiry — but the `LearningEvent::Unlearned` audit record is **permanent**, surviving compaction.

## Time

**Bi-temporal** — Every fact has two time axes:
- `t_valid_start` / `t_valid_end` — *valid time*: when this fact was true in the world
- `t_tx_start` — *transaction time*: when agidb learned it

Queries can be issued "as of" any historical date along either axis. This is how agidb answers "what did we believe about X on date Y?" without losing earlier facts.

**Supersession** — How agidb handles contradictions. When a new fact disagrees with an old one with the same `(subject, predicate)` and overlapping valid time, the old fact gets `t_valid_end = now - 1ms` and `superseded_by = <new id>`; the new fact gets `t_valid_start = now`. The old fact is preserved, not overwritten — see [`.specify/memory/constitution.md`](./.specify/memory/constitution.md) article V.

**Decay** — Background process that reduces a SemanticAtom's confidence over time when it isn't referenced. Atoms whose confidence falls below the floor get archived (cold storage), not deleted.

**Consolidation** — The McClelland-McNaughton-O'Reilly–style background loop that clusters recent episodic signatures, creates SemanticAtoms from clusters with N ≥ 3 evidence episodes, detects contradictions, promotes high-confidence atoms toward beliefs, updates the SelfVector, decays unreferenced atoms, and compacts storage. The analog of biological sleep. See [`docs/architecture/architecture.md`](./docs/architecture/architecture.md).

## Retrieval

**Recall** — The public read operation. Takes a Query (cue text + optional `as_of` + optional session + `min_confidence` + `tier_floor` + `k` + `trace_attention` + `goal_bias_weight`), returns a Recall (episodic matches + SemanticAtom matches + relevant Beliefs + active Goals + `tier_used` + `elapsed_ms` + optional AttentionTrace). Per [constitution article VI](./.specify/memory/constitution.md), `recall()` never returns the empty set. No LLM in the read path (article IV).

**Tier** — The four bands of `recall()` results:
- **Tier A — Exact**: canonical entity match via the concept index
- **Tier B — Similarity**: HDC signature similarity (POPCOUNT over inverted-index intersection)
- **Tier C — Gist**: raw-text gist signature similarity (fallback when B falls below threshold)
- **Tier D — NearestNeighbor**: best-effort match with explicit `low_confidence = true`

`recall()` falls through tiers in order; `tier_floor` caps how deep it can fall.

**Confidence** — A `f32` in `[0.0, 1.0]` attached to every Triple, Episode, SemanticAtom, Belief, and RecallMatch. Calibrated per tier so the score is comparable across tiers (ECE ≤ 0.05 target). Never silently inferred — every fact's confidence has a documented origin.

**Working memory** — Session-scoped recall behavior. A `session_id` plus `session_boost` + `recency_tau` lets recall favor in-session and recent matches without polluting the long-term store. Floor 2; ~7 items in the published cognitive literature.

**Goal-biased retrieval** — A reweighting pass in `recall()`: each active Goal's HDC signature up-weights related matches by `goal_bias_weight · similarity(episode_sig, goal_sig)`. Attention as a cognitive function — the agent attends to what it wants.

## Operations on HVs (the kernel)

**bind** — XOR-binding. `a.bind(&b)` produces an HV that encodes "a in role b" and is uncorrelated to both operands. Self-inverse (`a.bind(&b).bind(&b) == a`) and commutative. Used to bind a role HV with a filler HV (e.g. `SUBJ ⊗ Sarah`).

**bundle** — Per-bit majority bundling. `bundle([a, b, c])` produces an HV where each bit is 1 iff strictly more than half of the inputs have that bit set. Members of a small bundle are significantly more similar to the bundle than chance (≥ 0.6 for N ≤ 7). Used to combine triples into an Episode signature.

**hamming** — Bit-count of `a XOR b`. The primary similarity metric — `similarity = 1 - hamming/D`. Dispatched at runtime to AVX-512 `vpopcntdq` on x86_64 when present, NEON `vcntq_u8` on aarch64, or a portable `u64::count_ones` fallback.

**subtract** — v2 self-vector operation. Removes a bundle's contribution from an HV (`self_vec ← self_vec - α·bundle(tombstoned)`). Used by Unlearn so a forgotten concept stops biasing the SelfVector.

**active_dims** — Iterator over the indices of bits set in an HV. Drives the inverted-index update path in layer 3.

## Three engineering layers (how it's built)

**Layer 1 — Recall** — The mind-like layer. HDC signatures, binding, bundling, hamming-distance retrieval, tiered confidence, goal-biased reweighting. The only layer the user touches.

**Layer 2 — Extraction** — The scaffolding. GLiNER ONNX turns natural language into Triples + time anchors + belief candidates so signatures encode meaning, not phrasing. _(v2.1)_ adds the multimodal encoders.

**Layer 3 — Storage** — The plumbing. redb for metadata + bi-temporal indexes; mmap'd flat files for HV signatures; append-only logs for the self-model audit trail. ACID, crash-safe, pure-Rust, embedded.

## Seven cognitive floors (what it stores)

The biological framing — orthogonal to the engineering layers. Each floor is a typed shape with its own retrieval semantics. See [`docs/product/biological-mapping.md`](./docs/product/biological-mapping.md).

- **Floor 1 — Sensory** — Raw signal in a surprise-gated ring buffer. Promotes the novel to episodic, drops the rest. Stored as SensoryFrames. _(v2.1)_ multimodal: video + audio + text.
- **Floor 2 — Working** — Active, session-scoped context (~7 items). Not a separate table — a session + recency boost over episodic recall.
- **Floor 3 — Episodic** — Autobiographical events with time / place / people. Stored as Episodes.
- **Floor 4 — Semantic** — Decoupled general knowledge. Stored as SemanticAtoms produced by consolidation.
- **Floor 5 — Procedural** — Workflows and skills with execution traces. Stored as Procedures.
- **Floor 6 — Goals + Beliefs** — What the agent wants and what it thinks is true. Stored as Goals and Beliefs.
- **Floor 7 — Self-model** — The agent's audit log of its own development (LearningEvents) plus a slowly-drifting SelfVector.

## v2.1 — brain-aligned multimodal

Ships in the v2.1 milestone (phases 14–16), gated on a "Commit" outcome at the week-12 decision gate. See [`docs/architecture/brain-alignment.md`](./docs/architecture/brain-alignment.md).

**Modality** — The kind of sensory input: `Text`, `Image`, `Audio`, `Video`, or `Multimodal`.

**Multimodal binding** — VSA role-filler XOR binding of per-modality signatures into one episode signature. Factorable: each modality component is recoverable by XORing the bound episode with its `ROLE_*` hypervector.

**Charikar 2002 projection** — Thresholded random projection mapping a dense encoder latent (V-JEPA 2 / Wav2Vec-BERT / Llama-3.2-3B) to an 8192-bit HV. Deterministic, training-free, Johnson-Lindenstrauss distance-preserving.

**Brain-calibrated surprise** — The surprise threshold `θ_brain`, empirically fit against TRIBE v2's predicted neural surprise on associative cortex (TPJ, dlPFC, DMN). Replaces the hand-tuned `0.4` default.

**BAMS** — Brain-Aligned Memory Similarity benchmark. Representational-similarity analysis between agidb's signatures and TRIBE-derived cortical activations on matched stimuli. See [`docs/architecture/bams-benchmark.md`](./docs/architecture/bams-benchmark.md).

## Out-of-scope terms

If you see these in a draft, redirect the writer — they describe categories agidb is explicitly **not** ([constitution article XII](./.specify/memory/constitution.md)):

- **OLTP / transactional store** — agidb is not a system of record for orders or payments
- **Full-text search** — use tantivy or elastic
- **Pure similarity search over fixed embeddings** — use lancedb / qdrant
- **Document RAG** — agidb stores observations, not documents
- **Query language / DSL** — agidb has functions; no SQL, no Cypher
- **Distributed / sharded** — single-node, embedded-first; no plan to shard
- **Brain-decoding service** — agidb is brain-*aligned* (it benchmarks its representations against neural data); it does **not** decode, reconstruct, or infer neural signals. Constitution article XII, v2.1 extension.

> **Note:** *multimodal* (video / audio) was an out-of-scope term in sochdb v1. In agidb v2 it is in scope — the v2.1 milestone, phase 14. *Sensory memory* was likewise "upstream, out of scope" in v1; in v2 it is floor 1.
