# agidb domain glossary

> The vocabulary every doc, comment, identifier, test name, and PR title should use. If a term you need isn't here, either add it (one-line PR) or you're inventing language the project doesn't use — reconsider.
>
> This file is consumed by the Matt Pocock skills (`improve-codebase-architecture`, `diagnose`, `tdd`, `grill-with-docs`) per [`docs/agents/domain.md`](./docs/agents/domain.md). Drift between the glossary and the code is a signal for `/grill-with-docs`.

## Core nouns

**HV (Hypervector)** — A fixed-size binary vector of [`D = 8192`] bits / [`D_BYTES = 1024`] bytes, cache-line-aligned. The unit of representation in layer 1. Two HVs derived from unrelated names are uncorrelated in expectation (hamming ≈ D/2).

**Episode** — A single observation as stored by agidb. Carries the raw text, an HDC signature offset (into `signatures.dat`), an extracted list of Triples, bi-temporal stamps (`t_valid_start`, `t_valid_end`, `t_tx_start`), a `superseded_by` link if applicable, a Provenance record, and a confidence score. Identified by an `EpisodeId`.

**Triple** — A `(subject, predicate, object)` tuple extracted from natural-language input by layer 2. Each triple carries its own confidence score and a back-reference to its source Episode.

**Concept** — A canonical entity (e.g. "Sarah", "Bawri") with a deterministic HV (via `HV::from_name`), a canonical name, a list of aliases, and an entity type. Identified by a `ConceptId`. Concepts are how agidb canonicalizes mentions across episodes — "Sarah said X" and "sarah_kelly mentioned X" land on the same `ConceptId`.

**SemanticAtom** — A consolidated fact produced by the consolidation loop. Bundles repeated episodic patterns (N ≥ 3) into one durable statement with `evidence_count`, a list of source `EpisodeId`s for provenance, a confidence score, and a `last_referenced` timestamp. Identified by a `SemanticAtomId`.

**Procedure** — Procedural memory. A typed Episode shape describing a workflow or skill: name, description, trigger, preconditions, ordered steps (each with optional tool + args), postconditions. Answers "how do I do X?" the way Episodes answer "when did I do X?" and SemanticAtoms answer "what is X?"

**Provenance** — Attribution for a write. Records `source` (`"user"`, `"agent"`, `"tool:gmail"`, …), optional `session_id`, optional `trace_id`, and freeform metadata. Every Episode and SemanticAtom traces back to a Provenance record.

## Time

**Bi-temporal** — Every fact has two time axes:
- `t_valid_start` / `t_valid_end` — *valid time*: when this fact was true in the world
- `t_tx_start` — *transaction time*: when agidb learned it

Queries can be issued "as of" any historical date along either axis. This is how agidb answers "what did we believe about X on date Y?" without losing earlier facts.

**Supersession** — How agidb handles contradictions. When a new fact disagrees with an old one with the same `(subject, predicate)` and overlapping valid time, the old fact gets `t_valid_end = now - 1ms` and `superseded_by = <new id>`; the new fact gets `t_valid_start = now`. The old fact is preserved, not overwritten — see [`docs/spec/constitution.md`](./docs/spec/constitution.md) article V.

**Decay** — Background process that reduces a SemanticAtom's confidence over time when it isn't referenced. Atoms whose confidence falls below the floor get archived (cold storage), not deleted.

**Consolidation** — The McClelland-McNaughton-O'Reilly–style background loop that clusters recent episodic signatures, creates SemanticAtoms from clusters with N ≥ 3 evidence episodes, detects contradictions, decays unreferenced atoms, and compacts storage. The analog of biological sleep. See [`docs/architecture/architecture.md` § the consolidation loop](./docs/architecture/architecture.md).

## Retrieval

**Recall** — The public read operation. Takes a Query (cue text + optional `as_of` + optional session + min_confidence + tier_floor + k), returns a Recall (list of episodic matches + list of SemanticAtom matches + `tier_used` + `elapsed_ms`). Per [constitution article VI](./.specify/memory/constitution.md), `recall()` never returns the empty set.

**Tier** — The four bands of `recall()` results:
- **Tier A — Exact**: canonical entity match via the concept index
- **Tier B — Similarity**: HDC signature similarity (POPCOUNT over inverted-index intersection)
- **Tier C — Gist**: raw-text gist signature similarity (fallback when B falls below threshold)
- **Tier D — NearestNeighbor**: best-effort match with explicit `low_confidence=true`

`recall()` falls through tiers in order; `tier_floor` caps how deep it can fall.

**Confidence** — A `f32` in `[0.0, 1.0]` attached to every Triple, Episode, SemanticAtom, and RecallMatch. Calibrated per tier so that the score is comparable across tiers (ECE ≤ 0.05 target). Never silently inferred — every fact's confidence has a documented origin.

**Working memory** — Session-scoped recall behavior. A `session_id` plus `session_boost` + `recency_tau` lets recall favor in-session and recent matches without polluting the long-term store. The "active context" tier of biological memory; ~7 items in the published cognitive literature.

## Operations on HVs (the kernel)

**bind** — XOR-binding. `a.bind(&b)` produces an HV that encodes "a in role b" and is uncorrelated to both operands. Self-inverse (`a.bind(&b).bind(&b) == a`) and commutative. Used to bind a role HV with a filler HV (e.g. `SUBJ ⊗ Sarah`).

**bundle** — Per-bit majority bundling. `bundle([a, b, c])` produces an HV where each bit is 1 iff strictly more than half of the inputs have that bit set. Members of a small bundle are significantly more similar to the bundle than chance (≥ 0.6 for N ≤ 7). Used to combine triples into an Episode signature.

**hamming** — Bit-count of `a XOR b`. The primary similarity metric — `similarity = 1 - hamming/D`. Dispatched at runtime to AVX-512 `vpopcntdq` on x86_64 when present, NEON `vcntq_u8` on aarch64, or a portable `u64::count_ones` fallback.

**active_dims** — Iterator over the indices of bits set in an HV. Drives the inverted-index update path in layer 3.

## Three layers (engineering, not biological)

**Layer 1 — Recall** — The mind-like layer. HDC signatures, binding, bundling, hamming-distance retrieval, tiered confidence. The only layer the user touches.

**Layer 2 — Extraction** — The scaffolding. GLiNER ONNX turns natural language into Triples + time anchors so signatures encode meaning, not phrasing. ("Sarah recommended Bawri" and "Bawri was recommended by Sarah" produce the same signature.)

**Layer 3 — Storage** — The plumbing. redb for metadata + bi-temporal indexes; mmap'd flat files for HV signatures. ACID, crash-safe, pure-Rust, embedded.

## Five biological tiers (orthogonal to layers)

**Sensory memory** — Raw signal, <1s. Upstream of agidb. Out of scope.

**Working memory** — Active context, ~7 items. Modeled as session-scoped recall with `session_boost` + recency weighting.

**Episodic memory** — Events with time / place / people. Stored as Episodes.

**Semantic memory** — Decoupled facts. Stored as SemanticAtoms produced by consolidation.

**Procedural memory** — Workflows and skills. Stored as Procedures.

## Out-of-scope terms

If you see these in a draft, redirect the writer — they describe categories agidb is explicitly **not** ([constitution article XII](./.specify/memory/constitution.md)):

- **OLTP / transactional store** — agidb is not a system of record for orders or payments
- **Full-text search** — use tantivy or elastic
- **Pure similarity search over fixed embeddings** — use lancedb / qdrant
- **Document RAG** — agidb stores observations, not documents
- **Query language / DSL** — agidb has functions; no SQL, no Cypher
- **Distributed / sharded** — single-node only in v0.x
- **Multimodal** — text first; images / audio deferred to v0.3+
