# agidb — Roadmap

> The week-by-week phase plan from where we are today (sochdb v1 phases 0-2-4-6
> complete, rebranded to agidb v2) through v2.0 launch at month 9 and v2.1
> brain-alignment ship at month 12. Sixteen phases total. Decision gate
> binding at week 12.

**Status:** weeks counted from agidb v2 kickoff (rebrand from sochdb v1). Phases 0, 1, 2, 4, 6 already complete from sochdb. Remaining critical path: phases 3, 5, 9-13 for v2.0; phases 14-16 for v2.1.

## The 16 phases at a glance

| # | Phase | Weeks | Status | Version |
|---|---|---|---|---|
| 0 | Setup | — | ✅ done (sochdb v1) | inherited |
| 1 | HDC kernel | — | ✅ done (sochdb v1) | inherited |
| 2 | Storage | — | ✅ done (sochdb v1) | inherited |
| 3 | Extraction (GLiNER) | 1-4 | ⬜ | v2.0 critical |
| 4 | Binding + recall | — | ✅ done (sochdb v1) | inherited |
| 5 | MCP + Python | 5-8 | ⬜ | v2.0 critical |
| 6 | Consolidation | — | ✅ done (sochdb v1) | inherited |
| 7 | Decision gate | 11-13 | ⬜ | **binding** |
| 8 | Hardening + launch | 31-36 | ⬜ | v2.0 ship |
| 9 | Cognitive primitives (goals + beliefs) | 13-18 | ⬜ | v2.0 wedge |
| 10 | Sensory + self-model | 19-22 | ⬜ | v2.0 |
| 11 | Unlearn API | 23-25 | ⬜ | v2.0 |
| 12 | Neurosymbolic interface | 26-27 | ⬜ | v2.0 |
| 13 | Cognitive benchmarks | 28-30 | ⬜ | v2.0 |
| 14 | Multimodal sensory (V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B) | 37-42 | ⬜ | v2.1 (gated) |
| 15 | Brain-calibrated surprise | 43-46 | ⬜ | v2.1 (gated) |
| 16 | BAMS benchmark + ICLR paper | 47-52 | ⬜ | v2.1 (gated) |

## Phase ordering rationale

The ordering reflects three engineering constraints and one strategic constraint:

1. **Phase 3 first** — extraction unlocks tier B recall and alias resolution. Without it, the recall cascade is missing its most important tier. Also unlocks belief extraction, which phase 9 needs.
2. **Phase 5 second** — MCP + Python bindings make the engine consumable. Demos and design partners need this before we can run the decision gate.
3. **Phase 7 at week 12** — the binding decision gate happens *after* MCP/Python (so we can run real benchmarks against Mem0/Letta/Zep) but *before* the cognitive primitives. If the substrate doesn't beat incumbents on the standard agent-memory benchmarks, the cognitive-primitive bet doesn't get to run.
4. **Phases 9-13 after decision gate** — only build the cognitive primitives if the substrate wins the gate. Otherwise reposition or retreat.
5. **v2.1 phases 14-16 only on "Commit"** — constitutionally gated. No brain-alignment work if v2.0 substrate doesn't earn its credibility first.

## Pre-week-0 — Rebrand and namespace lock

Before the week-counter starts: rename sochdb → agidb across the codebase, push to GitHub, secure namespaces.

**Tasks:**
- ☐ Rename workspace crates: `sochdb-core` → `agidb-core`, `sochdb-cli` → `agidb-cli`, etc.
- ☐ Update `Cargo.toml` package names, dependency references, README path links.
- ☐ Update doc references from "sochdb" to "agidb" (~50 places across docs/).
- ☐ Rename storage error type: `SochError` → `AgidbError`.
- ☐ Update the manifest format string from "sochdb-v0.1" to "agidb-v2.0".
- ☐ `cargo build --workspace && cargo test --workspace` — all 44 tests still pass.
- ☐ Buy `agidb.ai`, `agidb.dev`, `agidb.io`, `agidb.co`.
- ☐ Create `github.com/agidb` organization, transfer existing sochdb commits.
- ☐ Reserve `agidb` crate name on crates.io (publish empty 0.0.1 placeholder).
- ☐ Reserve `agidb` package on PyPI (placeholder).
- ☐ Reserve `agidb` on npm (placeholder, even if no JS pkg planned, for namespace hygiene).
- ☐ Send formal prior-inventions email to Naman at Utkrusht.ai (this is the legal hygiene step you mentioned).

**Exit criterion:** the codebase compiles under the new name, all 44 tests pass, the GitHub org exists, the four domains are locked, the crates.io/PyPI/npm placeholders are claimed. Estimated effort: 1-2 weekends.

This is *not* counted as a week of the build. It's prerequisite hygiene.

---

## Weeks 1-4 — Phase 3: Extraction (GLiNER)

**Goal:** raw text in, structured triples + canonical entities + parsed time anchors + belief candidates out.

### Week 1

- ☐ Vendor GLiNER ONNX model + tokenizer code from ctxgraph repo. Compile under `agidb-extract` crate.
- ☐ Wire `ort` (ONNX runtime) into the workspace. Verify CPU-only inference path works.
- ☐ Add `agidb-extract::gliner::GLiNERExtractor` with `extract(text, entity_types) -> Vec<Entity>` API.
- ☐ Write unit tests: 10 hand-labeled observations, check that entities + spans extracted correctly.

### Week 2

- ☐ Build `agidb-extract::relations` — given entities + sentence context, extract `(subj, pred, obj)` triples.
- ☐ Add predicate-canonicalization trie ("recommended", "suggested", "told me about" → `recommends`).
- ☐ Build `agidb-extract::time` — parse "last weekend", "two months ago", ISO dates, etc., into `TimeRange`. Use `chrono_english` for casual phrasings.
- ☐ Build `agidb-extract::alias` — fuzzy match new mentions to existing canonical concepts (exact match + Levenshtein ≤ 3 for typos).

### Week 3

- ☐ Wire extraction into `Agidb::observe(text)` — replace today's "pre-extracted triples only" path with full pipeline.
- ☐ Property tests: 50 synthetic observations with known triples; check F1 > 0.85.
- ☐ Build gold-set evaluation: 100 hand-labeled observations from realistic agent-conversation data; record F1, precision, recall.
- ☐ Activate tier B in the recall cascade (now that triples exist with proper canonicalization).
- ☐ Activate alias resolution in tier A.

### Week 4

- ☐ Build belief extractor: detect "X said Y", "X believes Y", "X claimed Y" patterns; emit `Belief` candidates with confidence priors (0.5-0.8 depending on predicate).
- ☐ Integration tests for full observe pipeline: text in → episode stored, triples in redb, signature in mmap, belief candidates queued.
- ☐ Benchmark: 100 observations/sec on a laptop CPU end-to-end.
- ☐ Documentation update: `layer-2-extraction.md` reflects shipped behavior, not aspirational.

**Exit criterion:** `cargo test -p agidb-extract` passes ≥30 new tests. F1 > 0.85 on the 100-sample gold set. Tier B activates correctly in `recall()`. **Phase 3 complete.**

---

## Weeks 5-8 — Phase 5: MCP + Python

**Goal:** make agidb consumable from outside the Rust workspace. MCP server + Python wheels.

### Week 5

- ☐ Build `agidb-mcp` crate. MCP server skeleton over stdio + JSON-RPC.
- ☐ Expose MCP tools: `observe`, `recall`, `consolidate`, `between`. (Goals/beliefs added later, post-phase-9.)
- ☐ Tool schemas: JSON-Schema input/output for each, with examples.
- ☐ Smoke-test against Claude Desktop: register `agidb` as an MCP server, observe + recall via Claude Desktop chat.

### Week 6

- ☐ Build `agidb-py` crate. pyo3 bindings, async via pyo3-asyncio.
- ☐ Expose: `Agidb.open`, `observe`, `recall`, `consolidate`, `set_goal` (stub for now), `assert_belief` (stub for now).
- ☐ Build maturin pipeline. Local `pip install -e .` works.
- ☐ Type stubs: `agidb.pyi` for IDE support.

### Week 7

- ☐ CI: build wheels for macOS (arm64 + x86), Linux (x86 + arm64), Windows (x86).
- ☐ Test wheels in fresh venvs across all platforms; verify imports and basic ops.
- ☐ Quickstart Python notebook: 50 LOC end-to-end demo (observe a conversation, recall, consolidate).
- ☐ MCP server: configurable port + transport (stdio default + optional WebSocket for non-Anthropic clients).

### Week 8

- ☐ Documentation: Python API reference, MCP tool reference.
- ☐ Example agents: 3 small example agents (research-summarizer, journal, todo-helper) using agidb-py.
- ☐ Performance sanity-check across all bindings: end-to-end recall p95 < 100ms even through Python/MCP layer.

**Exit criterion:** `pip install agidb` works from a fresh venv. Claude Desktop can use agidb as a memory tool. 3 example agents run. **Phase 5 complete.**

---

## Weeks 9-10 — Benchmark harness build (phase 7 prep)

**Goal:** build the harness *before* the cognitive primitives, so the decision gate at week 12 has working benchmarks ready to run.

### Week 9

- ☐ Build `agidb-bench` crate.
- ☐ Implement LongMemEval-S harness: load dataset, run agent loop with agidb backend, score with the official LongMemEval grading prompt.
- ☐ Implement equivalent harness for Mem0 baseline (call Mem0's Python SDK from agidb-bench via subprocess).
- ☐ Six-metric output: BLEU, F1, LLM-judge, token cost, p95 latency, noisy-cue degradation.

### Week 10

- ☐ LoCoMo harness — 10+ session conversations, memory consistency scoring.
- ☐ BEAM harness — millions-of-tokens scale, contradiction resolution.
- ☐ Baselines: Mem0, Letta, Zep/Graphiti (each via their respective Python SDK; subprocess invocation).
- ☐ Reproducibility kit: docker-compose for harness + all baselines, fixed seeds, committed dataset SHAs.
- ☐ Commit harness code by EOW10 (constitution article XIII: "harness committed by week 8" — we're slightly behind but inside the 13-week window).

**Exit criterion:** `agidb-bench run --suite all --systems agidb,mem0,letta,zep` produces a JSON report with the six metrics across the three benchmarks. Reproducible from a docker container.

---

## Weeks 11-13 — Phase 7: Decision gate (binding)

**Goal:** run the benchmarks, publish results, make the binding commit/reposition/retreat decision.

### Week 11

- ☐ Commit thresholds (constitution article XIII: "thresholds committed by week 10" — we're a week behind but inside the 13-week window). Write them down publicly so they can't be quietly moved later.
- ☐ Run full benchmark suite. Three movies' worth of compute.
- ☐ Sanity-check results against published numbers from Mem0/Letta/Zep papers; investigate anything off by > 5%.

### Week 12 — the actual gate

- ☐ Final benchmark run. Raw logs preserved.
- ☐ Compare results against the three thresholds:
  - **Commit:** agidb wins/ties on accuracy, beats on latency 3×+, beats on token cost 3×+, wins noisy-cue degradation. → proceed to phases 9-13 + v2.1.
  - **Reposition:** agidb within 3pp of Mem0 F1 AND ≥10× memory savings. → ship as "agidb-lite", skip v2.1, no fundraise.
  - **Retreat:** more than 10pp behind on accuracy, no closing path. → fold back into ctxgraph.

### Week 13

- ☐ Decision communicated to (a) self, (b) Naman/Utkrusht context (informational), (c) any prospective design partners.
- ☐ If Commit: phase 9 starts week 13. (Phase 9 takes 6 weeks → ends week 18.)
- ☐ If Reposition: pivot the messaging, defer phases 9-13, focus on phase 8 hardening as "agidb-lite", skip v2.1.
- ☐ If Retreat: write a public post-mortem, transfer code back into ctxgraph repo, retire the agidb name.

**Exit criterion (assuming Commit):** decision made and publicly logged. Phase 9 begins. **Phase 7 complete.**

The rest of this roadmap assumes Commit. If Reposition or Retreat, see ROADMAP_REPOSITION.md or ROADMAP_RETREAT.md (TBD docs that get written if those branches activate).

---

## Weeks 13-18 — Phase 9: Cognitive primitives (the wedge)

**Goal:** `Goal` and `Belief` as first-class typed shapes with state machines, revision audit, HDC signatures. The thing no other agent memory system has.

### Week 13

- ☐ Add `agidb-core::goal` module. Types: `Goal`, `GoalState`, `GoalPatch`, `GoalTree`, `SuccessCriterion`. State-machine transition validator.
- ☐ Add `agidb-core::belief` module. Types: `Belief`, `BeliefRevision`, `Evidence`, `RevisionReport`.
- ☐ Two new redb tables: `goals`, `beliefs`. Migration code: open v2.0 db without these tables → create them empty.
- ☐ Property tests: goal state machine invariants (Completed/Abandoned are terminal; pause/resume preserves history).

### Week 14

- ☐ Implement `Agidb::set_goal`, `revise_goal`, `complete_goal`, `abandon_goal`, `active_goals`, `goal_tree`, `get_goal`.
- ☐ Goal HDC signature derivation: bind description tokens with parent context.
- ☐ Add `belief_revisions` redb table (third v2.0 table this phase).
- ☐ Implement `Agidb::assert_belief`, `revise_belief`, `what_do_i_believe`, `belief_history`, `withdraw_belief`.

### Week 15

- ☐ Belief revision math: Bayesian-style confidence update on new evidence. Append `BeliefRevision` to log on every change.
- ☐ LLM-assisted revision (constitution article IV amendment): when evidence is ambiguous, call an LLM at write time to judge contradiction. Structured prompt → structured `RevisionDecision`. Document which LLMs are supported (Claude, GPT, local Llama via Ollama).
- ☐ Withdraw belief on confidence drop below 0.5 (configurable).
- ☐ 100-step goal-mutation property test: random walk through goal state machines never violates invariants.

### Week 16

- ☐ Wire goal-biased retrieval into `recall()`. Active goals' HDC signatures up-weight related episode matches by `goal_bias_weight * similarity(episode_sig, goal_sig)`.
- ☐ Add `Recall::active_goals` and `Recall::goal_biased` fields.
- ☐ Extend MCP server with goal/belief tools: `set_goal`, `revise_goal`, `assert_belief`, `revise_belief`, `what_do_i_believe`, `active_goals`.
- ☐ Extend Python bindings with the same.

### Week 17

- ☐ Belief context in recall results: `Recall::beliefs` field populated with beliefs about the queried subject.
- ☐ Concept-level belief lookups: `what_do_i_believe(ConceptId)` fast (indexed by belief.subject).
- ☐ Property test: belief revision log captures every change; replaying the log reconstructs current confidence.

### Week 18

- ☐ Integration test: 20-turn agent simulation where goals get set/revised/completed, beliefs get asserted/revised/withdrawn. Verify final state matches expected.
- ☐ Benchmark: `set_goal` ≤ 5ms, `assert_belief` ≤ 5ms, `revise_belief` ≤ 50ms (LLM-assisted path can be slower).
- ☐ Docs update: `cognitive-primitives.md` matches shipped behavior.

**Exit criterion:** 100-step goal mutation test passes. Belief revision audit log captures every change. Goal-biased retrieval working. **Phase 9 complete.**

---

## Weeks 19-22 — Phase 10: Sensory + self-model

**Goal:** floor 1 (sensory ring buffer with surprise gating) and floor 7 (learning event log + self-vector EMA).

### Week 19

- ☐ Add `agidb-core::sensory` module. Types: `SensoryFrame`, `SensoryData`, `Modality`, ring-buffer logic.
- ☐ New redb table: `sensory_buffer` (with ring-eviction semantics).
- ☐ Implement `Agidb::observe_sensory`, `working_state`, `surprise_score`.
- ☐ Surprise computation: `1 - similarity(new_sig, bundle_of(recent_beliefs))`.

### Week 20

- ☐ Surprise-gated promotion: sensory frames with `surprise > threshold` (default 0.4) auto-promote to episodic via internal `observe()` call.
- ☐ Add `agidb-core::learning_log` module. New redb table: `learning_events`.
- ☐ Implement `LearningEvent` enum (closed set per constitution XV implication). Emit events from every state-changing operation across the engine.

### Week 21

- ☐ Implement `Agidb::what_did_i_learn(since)` — query the learning log.
- ☐ Add `attention_trace` recording to the recall path. When `query.trace_attention = true`, build `AttentionTrace` and emit to learning log.
- ☐ Implement `Agidb::attention_trace(recall_id)` lookup.

### Week 22

- ☐ Self-vector implementation. New redb table: `self_vector_history` (originally scheduled for v2.1, brought forward into v2.0 because phase 11's unlearn needs it). 8192-bit HV, EMA update on each consolidate pass: `self_vec ← (1-α) self_vec + α bundle(consolidated_atoms)`.
- ☐ Implement `Agidb::self_vector`, `self_vector_at(time)`, `self_vector_history`.
- ☐ Wire self-vector update into the consolidation worker (extends phase 6 code).
- ☐ Benchmark: sensory ingest 1000 frames/sec, surprise gating promotes ~5%, learning log writes don't bottleneck observe.

**Exit criterion:** sensory buffer ingests at target rate. Surprise gating promotes only the novel. Learning log captures every state change. Self-vector drifts with consolidation. **Phase 10 complete.**

---

## Weeks 23-25 — Phase 11: Unlearn API

**Goal:** non-destructive cascading unlearn with self-vector subtraction and permanent audit. Constitution article XVI.

### Week 23

- ☐ Add `agidb-core::unlearn` module. Types: `UnlearnTarget`, `UnlearnReport`, `Tombstone`, cascade-graph computation.
- ☐ New redb table: `tombstones`.
- ☐ Cascade-graph algorithm: given a target (Concept/Episode/Belief/Session/Source), compute the full dependency set across episodes, beliefs, semantic atoms, procedures.
- ☐ Property test: cascade-graph correctly identifies all dependents (gold set of 20 hand-traced cascades).

### Week 24

- ☐ Implement `Agidb::unlearn(target, reason)`:
  1. Compute cascade.
  2. Tombstone all affected rows (set `tombstoned_at`).
  3. Invalidate signatures in mmap (mark in slot header).
  4. Cascade through beliefs: confidence reduce or withdraw; emit `BeliefRevision`.
  5. Cascade through semantic atoms: recompute without removed evidence; withdraw if evidence drops below threshold.
  6. **Self-vector subtraction:** `self_vec ← self_vec - α · bundle(tombstoned_sigs)`. Append corrected snapshot to `self_vector_history`.
  7. Emit `LearningEvent::Unlearned` (permanent, survives compaction).
- ☐ Implement `Agidb::unlearn_report`, `unlearn_history`, `restore_within_window` (30-day recovery).

### Week 25

- ☐ Bi-temporal filter in `recall()` extended: tombstoned rows excluded by default; `as_of` queries can still surface them within the 30-day window.
- ☐ Property tests: unlearn a 100-episode concept → all references gone within 100ms; self-vector hamming distance to pre-unlearn state matches `α · bundle(tombstoned)`.
- ☐ Compliance test: simulate a GDPR Article 17 request (BySource unlearn). Verify all data gone, audit log entry permanent.
- ☐ MCP + Python expose `unlearn`, `unlearn_history`, `restore_within_window`.

**Exit criterion:** 100-episode unlearn completes in ≤100ms. Self-vector verifiably no longer contains the unlearned concept. Audit log permanent. **Phase 11 complete.**

---

## Weeks 26-27 — Phase 12: Neurosymbolic interface

**Goal:** expose the implicit signature↔triple translation as a first-class API. Hybrid queries.

### Week 26

- ☐ Add `agidb-ns` crate (already scaffolded). Implement the five translation directions: triple_to_signature, signature_to_triples, cue_to_partial_signature, belief_to_signature, multimodal-factorization stub (full multimodal in phase 14).
- ☐ Implement `Agidb::neurosymbolic_query` with `HybridWeights`. Combines structured triple-pattern matching with fuzzy HDC similarity.
- ☐ Default hybrid weights for `recall()`: `{structured: 0.7, fuzzy: 0.3}`.

### Week 27

- ☐ Property tests: bind-then-unbind roundtrip recovers triples with low hamming error. Hybrid weights at extremes (1,0) and (0,1) reduce to pure structured / pure fuzzy.
- ☐ MCP + Python expose `neurosymbolic_query`, `signature_to_triples`, `triples_to_signature`.
- ☐ Docs: `neurosymbolic.md` matches shipped behavior.

**Exit criterion:** hybrid queries with 50/50 weights return appropriately blended results. **Phase 12 complete.**

---

## Weeks 28-30 — Phase 13: Cognitive benchmarks

**Goal:** the four cognitive benchmarks no other system can run on itself.

### Week 28

- ☐ Build `agidb-bench::cognitive` module with four benchmark suites:
  - **Goal consistency:** 50 simulated agent sessions with goal trees of depth 3; verify state machine never violates invariants.
  - **Belief revision:** 50 sequences of (assertion, contradiction, re-assertion) with known correct revision history; verify agidb's audit log matches.
  - **Unlearn cascade:** 30 GDPR-style requests; verify cascading removal completes correctly + self-vector reflects subtraction.
  - **Multi-floor retrieval:** 50 queries requiring information from 2+ floors (e.g. "what did Sarah say about my current goal?") — verify recall returns matches grounded across floors.

### Week 29

- ☐ Run benchmarks against agidb. Document thresholds: goal consistency ≥99%, belief revision audit ≥95% match, unlearn cascade ≥99%, multi-floor retrieval F1 ≥80%.
- ☐ Comparison baselines (where they're applicable): run goal consistency + belief revision against mem0/letta/zep — most will score near 0% because they don't have these primitives. That's the point.

### Week 30

- ☐ Write up cognitive benchmark whitepaper section (becomes part of the eventual v2.0 launch arxiv paper).
- ☐ Integrate cognitive benchmarks into CI: every PR runs goal consistency + multi-floor retrieval as smoke tests.

**Exit criterion:** all four cognitive benchmarks pass agidb thresholds. **Phase 13 complete.**

---

## Weeks 31-36 — Phase 8: Hardening + launch (v2.0 ships)

**Goal:** turn an in-progress engine into a launchable v2.0 substrate.

### Week 31-32

- ☐ Expand the harness: add a fuzz target for `observe` (random text strings) and `recall` (random queries); run 24h fuzz, fix anything that crashes.
- ☐ 30-day soak test: continuous load test simulating an agent that observes 100/day, consolidates daily, recalls 1000/day, unlearns 5/week. Run on a laptop; verify no leaks, no degradation, no corruption.
- ☐ Crash-recovery tests: kill mid-write at 100 random points; verify recovery to last commit.

### Week 33

- ☐ Write the v2.0 arxiv whitepaper. ~12 pages. Sections: introduction, related work (mem0/letta/zep/cognee/MemMachine), architecture, benchmark methodology, results, cognitive benchmark results, future work (v2.1 brain-alignment teased here).
- ☐ Internal review.

### Week 34

- ☐ Onboard 3-5 design partners. Outreach to: 2 frontier-adjacent startups, 1 regulated-industry team (legal or healthcare), 1 local-first AI builder, 1 academic researcher (Hyperon/Monty-adjacent).
- ☐ Each partner gets a private alpha + a slack channel + biweekly check-ins.
- ☐ Documentation pass: every public API method has rustdoc with examples.

### Week 35

- ☐ Launch blog post draft. Demo video (3 minutes): observe → recall → goal → belief → consolidate → unlearn → self-model query.
- ☐ Public website at agidb.ai. Landing + docs + blog.
- ☐ crates.io publish: `agidb` 0.1.0 + all sub-crates. PyPI publish: `agidb` 0.1.0. MCP-registry publish.

### Week 36

- ☐ Public launch. arxiv post. blog post. HN/X/lobste.rs announcements. Mastodon for the federated AI/ML crowd.
- ☐ Office hours for the first 2 weeks post-launch: 1h/day for issues + questions.
- ☐ **v2.0 SHIPS. Month 9 milestone reached.**

**Exit criterion:** `cargo add agidb` and `pip install agidb` work. 3+ design partners running agidb in something resembling production. arxiv paper posted. Blog post live. 1000+ GitHub stars by end of week 36 (aspirational, not exit-gating). **Phase 8 complete. v2.0 LAUNCHED.**

---

## Weeks 37-42 — Phase 14: Multimodal sensory (v2.1 begins)

**Goal:** V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B sensory encoders, Charikar 2002 random projection to 8192-bit HVs, VSA multimodal binding.

**Gate check:** v2.1 work begins ONLY if phase 7 decision was "Commit" AND v2.0 launched successfully. Constitution article XVIII clause 2 + XIII extension.

### Week 37

- ☐ Create `agidb-sensory` crate. Add to workspace.
- ☐ Wire `ort` (ONNX runtime) for V-JEPA 2 inference. Download V-JEPA 2 Gigantic-256 weights from HuggingFace (CC BY-NC); pin SHA.
- ☐ Implement `agidb-sensory::vjepa::VJepa2Encoder` with `encode(video: &VideoClip) -> Result<[f32; 1024]>`. Spatial mean pooling of the 8192-token output.
- ☐ Smoke test: encode a 64-frame video clip, verify output shape + reasonable values.

### Week 38

- ☐ Wire Wav2Vec-BERT 2.0. Download weights, pin SHA. Implement `agidb-sensory::wav2vec::Wav2VecBertEncoder` with `encode(audio: &AudioClip) -> Result<[f32; 1024]>`. Temporal mean pooling.
- ☐ Wire Llama-3.2-3B as a text encoder (forward pass only, not generation). Implement `agidb-sensory::llama::LlamaTextEncoder` with `encode(text: &str) -> Result<[f32; 2048]>`. Mean pooling of layer-32 hidden state.
- ☐ Inference performance baseline on a laptop: measure CPU latency for each.

### Week 39

- ☐ Implement `agidb-sensory::project::HDCProjector` — Charikar 2002 thresholded random projection. Per-encoder seeded matrices.
- ☐ Property tests: same input + same seed → same output (determinism). 1000 random latent pairs → hamming distance ordering preserves cosine distance ordering (Spearman correlation > 0.85).
- ☐ Add `MultimodalEncoder` trait. Each encoder gets `encode_and_project(input) -> Result<HV>`.

### Week 40

- ☐ Implement `agidb-sensory::multimodal::bind_multimodal_episode` — VSA role-filler binding: `episode = ROLE_VIDEO ⊕ sig_v XOR ROLE_AUDIO ⊕ sig_a XOR ROLE_TEXT ⊕ sig_t XOR ROLE_GOAL ⊕ sig_g XOR ROLE_TIME ⊕ sig_time`.
- ☐ Implement modality factorization: `extract_modality_signature(episode_sig, modality)` returns approximate sig + nearest-neighbor cleanup against per-modality codebook.
- ☐ Property test: bind 3 modalities, extract each individually with cleanup, hamming distance to original sig ≤ 200 bits (2.5% of 8192).

### Week 41

- ☐ Extend `Agidb::observe_multimodal(video, audio, text, ctx)` API. Wire into layer 3 storage: append per-modality signatures to mmap, store offsets in new `modality_signatures` column on episodes.
- ☐ Two new redb tables: `self_vector_history` (already added in phase 10, schema unchanged), `encoder_versions` (new).
- ☐ Encoder version mismatch detection: open a db with encoders X, binary uses encoders Y → error with migration message.
- ☐ Extend `recall()` to factor multimodal episodes: per-modality similarity scoring when query specifies a modality preference.

### Week 42

- ☐ End-to-end benchmark: 30s video + 30s audio + 100 tokens text → encoded → projected → bound → stored. P50 latency ≤ 2s CPU on a laptop.
- ☐ Optional Candle backend: pure-Rust ML inference path as alternative to ONNX. Identical outputs to within 1e-3.
- ☐ MCP + Python expose `observe_multimodal`.
- ☐ Docs update: `layer-2-extraction.md`, `brain-alignment.md`, `layer-3-storage.md` reflect shipped behavior.

**Exit criterion:** end-to-end multimodal observe pipeline works. P50 latency ≤ 2s on laptop CPU. Modality factorization works (extract recovers original sig with < 200 bits noise). **Phase 14 complete.**

---

## Weeks 43-46 — Phase 15: Brain-calibrated surprise

**Goal:** empirically fit the surprise threshold θ_brain against TRIBE v2 predicted neural surprise.

### Week 43

- ☐ Download TRIBE v2 weights from `huggingface.co/facebook/tribev2` (CC BY-NC; research use). Pin SHA.
- ☐ Build TRIBE v2 inference wrapper. v2.1 uses PyO3 subprocess call to a Python script running TRIBE v2 (because TRIBE's reference inference is Python; pure-Rust port deferred to v2.2+).
- ☐ Verify TRIBE v2 inference matches published numbers on a sample stimulus (within Pearson r±0.005 of the paper's reported value on a single subject single movie).

### Week 44

- ☐ Acquire Courtois NeuroMod dataset access (open access; requires acknowledgment + email registration).
- ☐ Acquire Algonauts 2025 OOD stimulus files (open access via algonauts.org).
- ☐ Pick a representative subject (e.g. Courtois NeuroMod subject 1) and a held-out movie segment (e.g. Pulp Fiction first 20 minutes).
- ☐ Run TRIBE v2 over the stimulus → predicted BOLD per parcel per TR.

### Week 45

- ☐ Compute neural surprise: at each TR, `neural_surprise(t) = || BOLD_pred(t) - sliding_mean(BOLD_pred, ±5 TRs) || ` over associative-cortex parcels (TPJ, dlPFC, DMN regions in Schaefer 1000 atlas).
- ☐ Run agidb's observe_multimodal pipeline over the same stimulus → signature stream.
- ☐ Compute agidb surprise: at each TR, `agidb_surprise(t) = 1 - hamming_sim(sig(t), bundle(sigs[t-K..t]))`.
- ☐ Fit threshold θ_brain to maximize Pearson correlation between Indicator(agidb_surprise > θ_brain) and Indicator(neural_surprise > σ × mean_neural_surprise) for σ ∈ {1.5, 2.0, 2.5}.

### Week 46

- ☐ Validate calibration on a held-out movie (Princess Mononoke or World of Tomorrow). Calibrated threshold should generalize within ±10% of fitted value.
- ☐ Publish calibrated θ_brain as the default surprise threshold for new v2.1 databases. Store in `manifest.toml` with provenance (calibration dataset SHA, TRIBE v2 version, fit date).
- ☐ Documentation: `brain-alignment.md` section on calibration includes the full reproducible recipe.
- ☐ Add `Agidb::brain_calibration()` and `Agidb::recalibrate(dataset)` APIs.
- ☐ Comparison plot: pre-calibration (θ=0.4) vs post-calibration (θ_brain) sensory promotion patterns on a held-out movie. Visually demonstrate the difference.

**Exit criterion:** calibrated θ_brain ships in v2.1. Reproducible calibration recipe documented. **Phase 15 complete.**

---

## Weeks 47-52 — Phase 16: BAMS benchmark + ICLR paper

**Goal:** ship the brain-aligned memory similarity benchmark suite, run all baselines, write and submit the ICLR 2026 MemAgents workshop paper.

### Week 47

- ☐ Create `agidb-bams` crate.
- ☐ Implement `agidb-bams::protocol` — the BAMS protocol (per bams-benchmark.md): stimulus loading, TRIBE v2 inference, per-network RDM construction, agent RDM construction, RSA scoring.
- ☐ Implement `agidb-bams::networks` — six functional cortical network definitions (DMN, visual, auditory, language, dorsal attention, frontoparietal), Schaefer-to-network mapping.

### Week 48

- ☐ Build baseline adapters: `agidb-bams::baselines::{mem0, letta, zep, hipporag, raw_vjepa, random}`. Each implements `AgentMemorySystem::replay_stimulus(stream) -> Vec<HV>`.
- ☐ For text-only baselines (mem0/letta/zep), replay strategy: feed text descriptions of stimuli (captions/transcripts) since they don't support multimodal natively. Document this as a methodological limitation in the paper.
- ☐ Random baseline: random 8192-bit HVs as the statistical null. Should score ~0.

### Week 49

- ☐ Run full BAMS suite: 6 movies × 7 systems × 6 networks. Estimated compute: ~8h on a laptop with GPU; ~24h CPU-only. Run on a cloud GPU for speed.
- ☐ Generate report (`agidb-bams report results.json --format html`). Overall + per-network + per-movie tables.
- ☐ Ablations: agidb without VSA binding (concatenation), agidb with attention fusion instead of XOR, agidb without brain-calibrated surprise, agidb without consolidation.

### Week 50

- ☐ Paper draft. Title: *Brain-Aligned Memory Retrieval: Measuring Cognitive Plausibility in Agent Memory Systems via TRIBE-Derived Ground Truth*. Target: ICLR 2026 MemAgents workshop (6-page version). Sections per `bams-benchmark.md` paper outline.
- ☐ Figures: overall BAMS scores table, per-network heatmap, ablation table, RDM visualizations (a few representative examples).
- ☐ Internal review.

### Week 51

- ☐ Address review feedback. Revise paper.
- ☐ Build reproduction kit: Docker container that runs the full BAMS suite end-to-end with one command. Pin all dependency versions, dataset SHAs, model weight hashes.
- ☐ Open-source `agidb-bams` on `github.com/agidb/agidb-bams` under Apache-2.0 (benchmark code) with explicit notes about TRIBE v2 CC BY-NC for the weight artifacts.

### Week 52

- ☐ Submit to ICLR 2026 MemAgents workshop. (If deadline missed, backup is CCN 2026.)
- ☐ Crates.io: publish `agidb 0.2.0` (v2.1) + `agidb-sensory 0.1.0` + `agidb-bams 0.1.0`. PyPI: publish `agidb 0.2.0`.
- ☐ Launch blog post for v2.1. Demo: observe a video clip, recall it via cue, factor by modality, run BAMS self-score.
- ☐ **v2.1 SHIPS. Month 12 milestone reached.**

**Exit criterion:** BAMS suite open-source with reproducible baselines. ICLR 2026 MemAgents paper submitted. agidb 0.2.0 published. **Phase 16 complete. v2.1 LAUNCHED.**

---

## Beyond week 52

After v2.1 ships, the focus shifts to:
- **Seed fundraise** (if not done sooner): now there's a substrate + a paper + design partners. Target $1-3M from a deep-tech-friendly fund.
- **v2.2 cognitive engine work** (2027): pattern completion, AGM belief revision, analogical retrieval. See `agi-trajectory.md`.
- **Community + ecosystem:** developer relations, conference talks (ICLR 2026 in person if accepted, CCN 2026, MLSys 2027 submission, RustConf workshop), contributor onboarding.
- **Hardening for the long tail:** issues from real production users, performance regressions, the things you only find by being in production for 6+ months.

## Risk register and mitigations

| Risk | Phase impacted | Mitigation |
|---|---|---|
| GLiNER F1 lower than 0.85 on real data | 3 | Augment with regex patterns + canonicalization rules; possibly add LLM-fallback for low-confidence extractions (write-time only) |
| Decision gate threshold ambiguous (close to threshold) | 7 | Pre-commit thresholds week 10; tiebreaker is noisy-cue degradation (the one Mem0 reliably loses) |
| Cognitive primitives ship but no design partners care | 9-13 | Talk to design partners *during* phases 9-13, not just at launch; iterate on the wedge based on real friction |
| V-JEPA 2 ONNX export incomplete/buggy | 14 | Fallback to Candle backend; or PyO3 subprocess to torch as last resort |
| TRIBE v2 inference too slow to calibrate | 15 | Use a smaller calibration subset (single movie, single subject) for v2.1; full calibration deferred to v2.2 |
| Courtois NeuroMod access friction | 15 | Backup: Algonauts 2025 OOD predictions are public-derivable from TRIBE v2 directly; doesn't strictly require Courtois |
| BAMS baselines (mem0/letta/zep) don't support multimodal | 16 | Document as methodological limitation; use text-only stimulus stream for those baselines; still scores meaningfully on language-network alignment |
| MemAgents deadline missed | 16 | Backup: CCN 2026 has a later deadline; if both missed, MLSys 2027 or NeurIPS 2026 main track |
| Burnout across 52 weeks | all | Pace: phases 9-13 are six weeks each, not three. Sleep more than the build dictates. Phases inherited from sochdb v1 = real savings, not aspirational. |

## What this roadmap doesn't try to cover

- Day-to-day engineering tasks (covered by issues + ADRs in the repo).
- Marketing + community-building beyond launch posts.
- Hiring (the plan is solo through v2.1; first hires post-seed in 2027).
- Detailed fundraise mechanics (separate doc when relevant).
- v2.2+ phase plans (see `agi-trajectory.md` for the 5-year shape; detailed roadmaps for v2.2+ get written when we get there).

This is a 52-week plan. It will slip. Slip-handling rule: when a phase runs over by more than 1 week, stop and decide explicitly whether to (a) cut scope of the current phase, (b) push everything downstream by the slip amount, or (c) deprioritize a later phase. Don't let slips compound silently.
