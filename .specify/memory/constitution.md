# agidb — Constitution

> The immutable principles governing every decision in agidb. Inherited from
> sochdb v1's 14 articles, extended with 3 new articles for the v2.0 AGI
> substrate pivot, plus 1 new article for the v2.1 brain-alignment milestone.
> Anything that contradicts the constitution requires an ADR.

**Status:** ratified · **Version:** 2.1 · **Last amended:** 2026-05-20

## Preamble

agidb is the cognitive substrate for autonomous AI agents. The choices that make it different from other databases — content-addressable HDC retrieval, bi-temporal supersession, no LLM in the read path, first-class cognitive primitives, non-destructive unlearn, brain-aligned multimodal sensory — are not feature decisions. They are architectural commitments that compound over time. Once violated, they cannot be unviolated.

These 18 articles are the principles that govern every design and engineering decision. Changes to the constitution require an ADR and a documented justification. The constitution survives leadership changes, funding rounds, and pressure to compromise.

---

## Article I — The one-liner is non-negotiable

agidb is **the cognitive substrate for autonomous AI agents — content-addressable hyperdimensional memory, first-class goals and beliefs, bi-temporal supersession, sleep-like consolidation, and a non-destructive unlearn primitive. One Rust binary, one API, no query language.**

Marketing may rephrase. Documentation may elaborate. The one-liner does not change. If a proposed feature does not fit the one-liner, it does not belong in agidb.

---

## Article II — No external index for retrieval

Storage and retrieval share the **same representation**. agidb does not ship a separate vector index for retrieval, a separate graph index for relations, a separate keyword index for text. The HDC signature is the index. The inverted-bit map is the index. Memories are retrieved by bit-overlap counting over the same signatures that store them.

This is the architectural wedge. Without this, agidb is just another vector DB with extra steps.

---

## Article III — Embedded-first forever

agidb is a single Rust binary that runs locally. No required server, no required cloud, no required API keys for the core path. The embedded form is canonical.

A hosted tier may exist for enterprise customers in v0.4+, but it must be strictly additive — the OSS embedded engine stays free, complete, and self-hostable. Anything that requires hosting to function is out of scope.

---

## Article IV — No LLM in the read path

`recall()` is deterministic math over stored signatures. No LLM call, no embedding API, no network round-trip. Read latency is a function of CPU and disk only.

**Amendment for v2 (new):** LLMs may participate at write time for belief revision and consolidation when semantic judgment is required beyond pure HDC math. Belief revision benefits from an LLM's ability to assess whether new evidence genuinely contradicts an old belief. Semantic atom creation may use an LLM to summarize clusters. **The read path remains LLM-free.**

**Clarification for v2.1:** sensory encoders (V-JEPA 2, Wav2Vec-BERT, Llama-3.2-3B) at write time are *not* LLM calls in the constitutional sense. They are frozen feature extractors run locally via ONNX or Candle. No network calls. No tokens billed. The encoders are part of layer 2 (extraction), not part of inference/reasoning. They are deterministic feature engineering on raw modalities.

---

## Article V — Bi-temporal supersession, never overwrite

Every fact has four timestamps: `t_valid_start`, `t_valid_end`, `t_tx_start`, `t_tx_end`. Updates produce new rows; old rows are marked `superseded_by`. Nothing is silently overwritten. Queries can ask "what was true at time T?" (valid-time) and "what did I know at time T?" (transaction-time) — they are different questions with different answers.

This is how legal and financial systems track changing facts. It is how human memory tracks contradicting information. agidb mirrors both.

---

## Article VI — Never return the empty set

`recall()` never returns `[]`. It returns a result with a `tier_used` field indicating how good the match was: `Exact` / `Similarity` / `Gist` / `NearestNeighbor`. If nothing matches above the floor, return the best available with `low_confidence: true`.

Agents must always get *something* and the confidence to know how much to trust it. Returning nothing forces the agent into error-handling paths that don't reflect how cognition actually works.

---

## Article VII — Full provenance, always

Every claim agidb makes traces back to the verbatim observation that produced it. Every belief has a `revision_log`. Every semantic atom has `source_episodes`. Every recall result includes a `provenance` field with source, session_id, trace_id.

No opaque embeddings. No untraceable facts. No "the model said so." If agidb cannot show its work, the work doesn't ship.

---

## Article VIII — Rust top to bottom

`agidb-core` is pure Rust. The only permitted FFI is ONNX runtime via the `ort` crate (for GLiNER in v2.0, V-JEPA 2 + Wav2Vec-BERT in v2.1) and Candle for pure-Rust ML inference where available. No Python bridges, no C++ engines, no JNI, no node-native modules in the core path. Python bindings exist via pyo3 for the user-facing API; the engine is Rust.

This is what gives agidb its sub-50ms p95 latency, its single-binary deployment, and its lack of GC pauses. Negotiable language choices invite negotiable performance.

---

## Article IX — No query language

agidb has no SQL, no Cypher, no GraphQL, no DSL. The public API is a small set of Rust functions exposed through bindings: `observe`, `observe_multimodal`, `recall`, `set_goal`, `assert_belief`, `unlearn`, `consolidate`, `what_about`, `between`, `what_did_i_learn`, `attention_trace`.

Query languages assume the user knows the schema. Agents don't. Agents say what they want; agidb figures out how to retrieve it.

---

## Article X — Benchmark honestly

Every performance or accuracy claim must be reproducible. Every benchmark run publishes raw logs, the harness commit hash, and the full six-metric stack (BLEU + F1 + LLM-judge + token cost + p95 latency + noisy-cue degradation) **plus** the cognitive benchmark results (goal consistency, belief revision, unlearn cascade, multi-floor retrieval).

**Extension for v2.1:** BAMS benchmark scores are published alongside the standard suite. RSA scores across all six functional cortical networks (DMN, visual, auditory, language, dorsal attention, frontoparietal) are reported. No cherry-picking individual networks. Calibration data and TRIBE v2 version pinned in every BAMS report.

No cherry-picking. No single-number claims. No "trust us, it's fast." If a comparison cannot be reproduced from the published harness, it does not appear in marketing.

---

## Article XI — Small, composable API surface

The public API of agidb is small enough to fit on a single screen. Every method is composable with every other method. New methods require a constitutional justification (this article); they are added rarely.

Featurism is a slow death. agidb is the database for autonomous agents, not the database that does everything for everyone.

---

## Article XII — Sacred non-goals

agidb is **not** and will never be:

- a general-purpose database (use postgres)
- a full-text search engine (use elasticsearch)
- a pure vector-similarity store (use pinecone)
- a hosted-only service (the OSS engine is always complete)
- a multimodal-document store (use lancedb)
- a distributed sharded database (use cockroach + a vector DB)
- a knowledge-graph editor with a UI (use neo4j browser)
- a fine-tuning service (use replicate or runpod)
- a brain-decoding service (TRIBE v2 weights are CC BY-NC, agidb uses TRIBE for *evaluation* only, never as a product feature decoding individual users' brains)
- AGI itself (agidb is the substrate AGI runs on)

Pressure to expand into any of these will be persistent. The answer is no. Pick a different tool for those problems.

---

## Article XIII — The binding decision gate

At phase 7 (week 12 of the v2.0 build), agidb runs the benchmark harness against Mem0, Zep/Graphiti, and Letta. The result is one of three binding outcomes:

- **Commit** — the numbers justify launch + v2.1 + fundraise. Proceed to phase 8 + v2.1 phases 14-16.
- **Reposition** — the numbers don't beat the incumbents but the embedded/cost angle still has a market. Ship as "agidb-lite," skip the seed round, skip v2.1 brain-alignment work, iterate.
- **Retreat** — the numbers don't justify the bet. Fold the learnings back into ctxgraph (sochdb's predecessor), retire the agidb name.

The decision gate is binding. No "let's give it another quarter." No moving the thresholds. The harness is committed by week 8; the thresholds are committed by week 10; the decision is made by week 13.

**v2.1 dependency rule:** v2.1 brain-alignment work (phases 14-16) is gated on the week-12 decision gate outcome being "Commit." If "Reposition" or "Retreat," v2.1 is canceled. No partial-credit ship of brain-alignment without substrate credibility first.

---

## Article XIV — Respect existing code

agidb v2 inherits sochdb v1's working code: phases 0, 1, 2, 4, 6 complete, 44 tests passing, ~13 commits. Every line of that code carries forward. The HDC kernel, the redb storage layer, the bi-temporal model, the episode encoder, the four-tier recall, the consolidation worker — all preserved.

When v2 adds capabilities, it adds them on top of v1's substrate, not by rewriting. The discipline of red-green test-first development from sochdb continues unchanged in agidb. Refactors that touch v1 code require ADRs.

---

## Article XV — Cognitive primitives are first-class types (new in v2.0)

Goals, beliefs, sensory frames, learning events, and unlearn targets are first-class Rust types with their own storage, retrieval, and audit semantics. They are not text fields inside episodes. They are not JSON blobs. They have:

- typed shapes in `types.rs` or dedicated modules
- redb tables with explicit schemas
- API methods with documented contracts
- property tests covering invariants

If a future contributor proposes "let's just store goals as JSON text," the answer is no. The whole point of the v2 pivot is that goals and beliefs are typed substrate primitives.

---

## Article XVI — Unlearn is non-destructive at audit (new in v2.0)

The `unlearn()` API removes facts from active retrieval and cascades through dependencies. Tombstoned data is recoverable for 30 days. After that, the data itself may be compacted away — but **the audit log entry recording the unlearn is permanent**.

Even after full compaction, agidb can answer: "was anything unlearned in the last year, and why?" The right-to-be-forgotten removes the data; it does not remove the *fact that data was removed*. This is what makes unlearn trustworthy.

**Extension for v2:** unlearn also subtracts from the self-vector (floor 7). Tombstoning the rows is not enough — the self-model EMA still contains a contribution from the unlearned episodes. Real unlearn means recomputing the self-vector with the tombstoned signatures subtracted. Otherwise the agent still "remembers" the unlearned concept as a centroid contamination in its self-model.

---

## Article XVII — Beliefs are revisable, never overwritten (new in v2.0)

When new evidence arrives that affects a belief's confidence, the belief is **revised**, not mutated. A `BeliefRevision` entry is appended to the revision_log capturing:

- `timestamp`
- `previous_confidence` and `new_confidence`
- `triggering_evidence` (the EpisodeId that caused the revision)
- `reason` (a brief human-readable explanation)

This means: the agent can always answer "what did I believe last week, and what changed my mind?" Belief revision is a special case of article V (bi-temporal supersession), specialized for the belief floor. Belief revision math may use an LLM at write time (per article IV amendment), but the revision log itself is structured data, not free text.

---

## Article XVIII — Brain-alignment is empirical and additive (new in v2.1)

v2.1 ships multimodal sensory encoding via V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B and the BAMS benchmark. The constitutional commitments are:

1. **Brain-alignment is empirical, not metaphorical.** We do not claim agidb "thinks like a brain." We claim agidb's internal representations align with TRIBE-predicted cortical activations on matched stimuli, measurable via RSA across six functional networks. The claim is falsifiable via BAMS evaluation.

2. **Brain-alignment is additive, not constitutive.** The substrate works without it (v2.0 ships text-only). v2.1 is the brain-aligned expansion. If brain-alignment turns out to be unhelpful empirically, v2.2+ can deprioritize it without breaking the substrate.

3. **TRIBE v2 is used for evaluation, not as a product feature.** We do not decode user brains. We do not ship TRIBE weights inside agidb. We use TRIBE v2 as published, frozen, for one purpose: scoring BAMS. CC BY-NC of TRIBE v2 weights means agidb's BAMS harness is research-licensed; the core substrate remains Apache-2.0.

4. **The encoder stack matches TRIBE v2 for alignment.** V-JEPA 2 Gigantic-256 for video, Wav2Vec-BERT 2.0 for audio, Llama-3.2-3B for text. Using the same encoders is what makes BAMS evaluation meaningful. Swapping encoders (e.g. to whisper for audio) requires re-running BAMS calibration and is an ADR-level decision.

5. **HDC projection is training-free.** Charikar 2002 thresholded random projection from encoder latents to 8192-bit signatures. Seed-fixed. Deterministic. No learned quantization in v2.1. Learned quantization may be considered in v2.2+ only if BAMS plateaus with the random projection.

6. **VSA binding remains factorable.** Multimodal fusion is via XOR role-filler binding, not attention. This is what lets agidb factor a stored episode signature back into its component modality signatures — something attention-based fusion cannot do. Factorability is the structural advantage over TRIBE's attention fusion and over mem0/letta/zep dense embeddings.

7. **BAMS scores ship with every release.** Once v2.1 ships, every subsequent release reports BAMS scores against the previous version. A regression in BAMS is a release-blocker unless explicitly justified by an ADR.

---

## Enforcement

Articles I-XII and XV-XVIII govern the public-facing product and its commitments. Articles XIII and XIV govern internal discipline.

When a proposed change conflicts with an article:
1. Write an ADR explaining the conflict
2. Justify why the change is necessary
3. Propose an amendment to the article OR demonstrate the change does not actually conflict
4. Get the amendment approved or the change rejected

Constitutional changes are not made by code review. They are made by ADR.

## Amendments since sochdb v1

- **Article IV amended** for v2.0: LLMs allowed at write time for belief revision and consolidation; read path stays LLM-free.
- **Article IV clarified** for v2.1: V-JEPA 2 / Wav2Vec-BERT / Llama-3.2-3B as frozen feature extractors are not LLM calls in the constitutional sense.
- **Article XII extended** for v2.1: brain-decoding service added to sacred non-goals.
- **Article XIII extended** for v2.1: v2.1 work gated on week-12 decision gate "Commit" outcome.
- **Article XV added** for v2.0: cognitive primitives as first-class typed shapes.
- **Article XVI added** for v2.0: non-destructive unlearn with permanent audit. **Extended for v2** with self-vector subtraction requirement.
- **Article XVII added** for v2.0: belief revision with explicit revision log.
- **Article XVIII added** for v2.1: brain-alignment is empirical and additive.

Articles I, II, III, V, VI, VII, VIII, IX, X, XI, XIV are inherited from sochdb v1, with article X extended for BAMS reporting.

---

*The constitution is the longest-lived document in agidb. Code rots; tests get refactored; benchmarks become obsolete. The principles persist.*
