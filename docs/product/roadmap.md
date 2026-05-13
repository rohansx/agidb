# sochdb — roadmap

a 6-month plan to take sochdb from concept to benchmark-credible v0.1, with a hard decision gate at week 12.

## guiding principles

- **ship small.** v0.1 is the smallest credible memory db, not the most complete one.
- **benchmark honestly.** every claim is reproducible. publish raw logs.
- **defer hosting.** stay embedded. cloud is v1.0 territory.
- **respect existing ctxgraph code.** vendor what's reusable. don't rewrite for the sake of rewriting.
- **decision gate at week 12.** if numbers don't justify the bet, reposition before week 26.

## v0.1 — the 6-month build

### phase 0 — setup (week 0)

- buy `sochdb.com` and `sochdb.dev`
- create `sochdb/sochdb` repo on github
- reserve `sochdb` on crates.io and PyPI
- initialize workspace structure (sochdb-core, sochdb-extract, sochdb-cli, sochdb-mcp, sochdb-py, sochdb-bench)
- set up CI (github actions, cargo test + clippy + rustfmt)
- write the OVERVIEW.md, ARCHITECTURE.md, LAYER_1, LAYER_2, LAYER_3, TECH_SPEC, ROADMAP

### phase 1 — the HDC kernel (weeks 1-2)

deliverables:
- `sochdb-core/src/hdc.rs` — HV type, bind, bundle, hamming, similarity, active_dims
- AVX-512 POPCOUNT path + portable fallback
- NEON POPCOUNT path for aarch64 / Apple silicon
- property tests for HDC algebra (binding inverse, bundling membership)
- micro-benchmarks: signature compute, bind, bundle, hamming over 100k

exit criterion: 8192-bit hamming-distance scan over 100k random signatures completes in under 5ms on M2.

### phase 2 — storage (weeks 3-4)

deliverables:
- `sochdb-core/src/store.rs` — redb tables for episodes, triples, concepts, indexes, consolidation_log
- `sochdb-core/src/signatures.rs` — mmap'd signatures.dat with append + offset lookup
- bi-temporal column model fully wired
- crash-safety tests: kill mid-write, verify recovery
- export / import via jsonl

exit criterion: open, observe (with placeholder extraction), close, reopen, recall by exact match. ACID at the episode level.

### phase 3 — extraction (weeks 5-6)

deliverables:
- vendor / port the GLiNER ONNX loading and inference code from ctxgraph
- `sochdb-extract/src/lib.rs` — `Extraction` pipeline producing entities, triples, time anchors
- alias resolution and concept canonicalization
- predicate canonicalization with synonym table
- explicit time parsing (chrono + small grammar for "last weekend" etc)
- confidence scoring propagation

exit criterion: `observe()` correctly extracts triples from 20 sample observations with >85% F1 against a human-labeled gold set.

### phase 4 — binding + tiered recall (weeks 7-8)

deliverables:
- `sochdb-core/src/episode.rs` — bind triples into role-filler patterns, bundle into episode signature
- `sochdb-core/src/recall.rs` — tier A (exact), tier B (similarity), tier C (gist), tier D (NN)
- inverted-index update path on observe
- confidence calibration across tiers
- end-to-end `Sochdb::recall()` working

exit criterion: recall on 1,000-episode synthetic dataset returns expected matches with calibrated confidence; p95 < 50ms.

### phase 5 — MCP + python bindings (weeks 9-10)

deliverables:
- `sochdb-mcp/src/main.rs` — MCP server exposing observe, recall, what_about, between, consolidate
- `sochdb-py/src/lib.rs` — pyo3 bindings with async support
- claude desktop integration tested
- python wheel build for linux/macOS/windows

exit criterion: claude desktop can use sochdb as a memory tool via MCP; `pip install sochdb` works.

### phase 6 — consolidation (weeks 11-12)

deliverables:
- `sochdb-core/src/consolidate.rs` — background tokio task
- clustering by hamming distance
- semantic atom creation when evidence ≥ 3
- contradiction detection with supersession
- decay function for unreferenced atoms
- consolidation log + audit trail
- `consolidate()` synchronous API for tests

exit criterion: consolidation reduces a 10k-episode store by ≥ 30% in semantic-atom count without losing recall accuracy.

### phase 7 — DECISION GATE (week 12)

run benchmarks against Mem0, Zep/Graphiti, Letta on **three benchmarks** using a shared harness, publishing **all metrics, never a single number**:

- **LongMemEval-S** — long-context episodic recall (Wu et al., 2024)
- **LoCoMo** — long-conversation memory across 10+ sessions (Maharana et al., 2024)
- **BEAM** — Mem0's own scale + contradiction-resolution benchmark (Mem0, 2026); specifically stresses supersession, which is sochdb's bet

every run publishes the full **metric stack**: BLEU, F1, LLM-judge (binary), token cost, p95 latency, plus a **noisy-cue degradation test** (20% cue tokens corrupted) to verify graceful fallback through tiers C and D. raw logs and the harness commit hash ship with every claim. baselines run from pinned versions on the same day.

**commit threshold (all four must hold):**
- sochdb ≥ Zep/Graphiti accuracy on LongMemEval-S — within 1pp on F1 **and** within 1pp on LLM-judge
- sochdb ≥ 3× lower p95 retrieval latency than Mem0
- sochdb ≥ 3× lower token cost than Mem0 (target < 2,500 tokens/query against Mem0's ~7k)
- sochdb wins the noisy-cue degradation test against all baselines
- holds on LongMemEval-S **and** LoCoMo **and** BEAM (no cherry-picking)

if met:
- proceed to phase 8 (hardening + launch)
- begin investor conversations (Lightspeed India, Accel, Peak XV, 100x)
- file the YC summer batch application

**reposition threshold:** sochdb within 3 pp of Mem0 F1 on LongMemEval-S **and** ≥ 10× memory footprint savings.

if met but commit threshold isn't:
- reposition as "embedded memory for edge agents"
- ship anyway, smaller positioning, no fundraise
- continue to v0.2 with refined focus

**retreat threshold:** sochdb more than 10 pp behind dense baselines on LongMemEval-S F1 **and** the gap doesn't close with reranking.

if hit:
- drop the HDC bet
- reposition as "Graphiti without Neo4j"
- merge sochdb learnings back into ctxgraph
- continue Utkrusht role full time

### phase 8 — hardening + launch (weeks 13-26)

assuming commit threshold met:

- expand benchmark harness to full LoCoMo and full BEAM (phase 7 used sampled subsets)
- continue publishing the full metric stack (BLEU + F1 + LLM-judge + token cost + p95 latency + noisy-cue) on every release
- write the arxiv whitepaper (target 12-15 pages, NeurIPS workshop or ICLR system track)
- design 3 design-partner deployments
- write the launch blog post
- record a 60-second demo video
- prepare HN Show post, Product Hunt, X thread
- file Cargo crate, PyPI wheel, MCP registry
- public launch at week 26

target metrics at launch:
- 500+ GitHub stars in week 1
- 3+ design-partner deployments confirmed
- benchmarks reproducible by external developers (verified: one external dev runs the harness end-to-end before launch)
- documentation complete and tested by 3 external readers

## v0.2 — the consolidation release (months 7-9)

after launch, the highest-leverage improvements:

- **encryption at rest.** AES-GCM over signatures.dat and redb file. necessary for personal-data customers.
- **learned predicate similarity.** small distilled model that maps surface predicates to a continuous similarity space. improves recall on semantically equivalent phrasings.
- **LSH over signatures.** for stores beyond 1M episodes. sub-linear nearest-neighbor.
- **batch observe.** ingest a JSONL of historical events efficiently. unlocks migration from existing memory systems.
- **WAL streaming.** event log of all observes for external mirroring / replication.

## v0.3 — the multimodal release (months 10-12)

- **image captions.** observe an image by extracting a caption (via local vision model or external) and binding the caption text + image embedding.
- **audio transcription.** observe a meeting by transcribing + extracting + binding.
- **richer time semantics.** ranges, recurring events, time zones.

## v0.4 — the multi-agent release (months 13-15)

- **memory sharing.** multiple agents can observe the same sochdb instance with provenance tracking who observed what.
- **scoping.** per-agent and per-session memory windows.
- **conflict resolution policies.** when two agents disagree on a fact, policy-driven supersession.

## v1.0 — production stability (month 18)

- **API stability commitment.** no breaking changes after 1.0 without 1.x deprecation cycle.
- **on-disk format frozen.** migrations always supported.
- **operational tooling.** integration with prometheus/otel/datadog.
- **DWM compression.** the structure from the HPE Hippocampus paper. unlocks 10M+ episode scale.
- **optional hosted tier.** for users who can't or won't run embedded. multi-tenant cloud product, sold separately, OSS engine remains free.

## what's NOT on the roadmap (out of scope)

- distributed / sharded sochdb. when you need that, you've outgrown the use case.
- a query language. the whole point is no query language.
- a UI. sochdb is a library; UIs are downstream.
- replacing vector databases for RAG over documents. lancedb / qdrant do that well.
- replacing graphs for general-purpose graph workloads. neo4j / kuzu do that.
- a fine-tuning service. out of scope.

## monthly review cadence

at the end of each month, review:

- progress against phase deliverables
- benchmark numbers (whenever harness is running)
- competitive landscape (mem0, zep, letta, cognee releases)
- decision-gate trajectory: are we on track for commit at week 12?

at the end of each quarter, review:

- roadmap horizon (does the next version still make sense?)
- positioning (does the one-liner still hold?)
- community signals (stars, issues, contributors)

## the explicit risk register

| risk | probability | mitigation |
|---|---|---|
| HDC recall accuracy too low vs dense embeddings | high | hybrid HDC + dense rerank as escape hatch |
| Mem0 / Cognee ships rust binding before we launch | medium | move fast; sochdb's wedge is embedded + bi-temporal, not just rust |
| solo-dev timeline slips past 26 weeks | very high | scope cut: ship observe + recall only at v0.1, defer rest |
| GLiNER ONNX latency too slow | medium | distill smaller model; quantize; fallback to regex+BM25 |
| HDC kernel bugs | high | proptest invariants; mirror Torchhd tests; fuzz |
| benchmark dispute (zep/mem0 LoCoMo style) | certain | publish harness + raw logs from day one |
| nobody understands what content-addressable means | medium | invest in the OVERVIEW.md and the demo video; frame in dev terms |
| utkrusht day job conflicts | high | prior inventions email already on file; sochdb on personal time |

## the one-liner doesn't change

through every phase, the product is:

> **sochdb is an embedded, content-addressable memory database for AI agents — storage and retrieval share the same hyperdimensional representation, bi-temporal by default, with automatic consolidation. one binary, one api, no query language.**

if a feature doesn't reinforce this one-liner, it's out of scope.
