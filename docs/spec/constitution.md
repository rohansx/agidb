# sochdb — constitution

> immutable principles that govern every decision in sochdb. if a feature, dependency, or design choice contradicts the constitution, it is out of scope. amendments require an explicit ADR.

## article 1 — the one-liner

sochdb is **an embedded, content-addressable memory database for AI agents — storage and retrieval share the same hyperdimensional representation, bi-temporal by default, with automatic consolidation. one binary, one api, no query language.**

if a proposed feature does not reinforce this one-liner, it does not ship.

## article 2 — the wedge

sochdb integrates binding and recall without an external index — storage and retrieval share the same representation, so retrieval doesn't detour through a separate vector or graph lookup. this is the non-negotiable architectural commitment.

violations:
- introducing a separate vector index (pgvector, qdrant, lancedb) as a primary read path
- introducing a separate graph store (neo4j, kuzu) as a primary read path
- requiring an LLM call on the read path

## article 3 — embedded-first, forever

sochdb is a library that runs in the user's process. like sqlite, not like postgres.

- single binary, no server
- works fully offline by default
- zero required network calls at read or write time
- no required API keys
- a hosted tier is permitted **only as an optional v1.0+ deployment mode**; the embedded engine must remain the canonical product

## article 4 — no LLM in the read path

`recall()`, `what_about()`, `between()`, `recall_procedure()` must complete with zero LLM invocations.

LLMs are permitted only:
- optionally, at write time, behind an explicit feature flag, for extraction enrichment
- in the eval harness, never in the production path

## article 5 — bi-temporal supersession over destructive update

every fact has `t_valid_start`, `t_valid_end`, `t_tx_start`. contradictions supersede, never overwrite. the answer to "what did we believe about X on date Y?" must always be answerable.

## article 6 — never return empty

`recall()` returns matches at one of four tiers: exact → similarity → gist → nearest-neighbor. confidence is always explicit. a query never returns the empty set — it returns the nearest neighbors with `low_confidence=true`.

## article 7 — provenance always

every claim sochdb makes traces back to a verbatim source observation. opaque embeddings as the only provenance are forbidden. consolidated semantic atoms must link back to their source episode ids.

## article 8 — rust top to bottom

the engine is rust. bindings (python via pyo3, MCP) wrap the rust engine. no python or javascript in `sochdb-core`.

permitted exceptions: ONNX runtime (`ort` crate) and tokenizers — both already pure-rust wrappers.

forbidden: GC-language reimplementations of core paths, async-std (use tokio), C++ deps when a rust equivalent exists.

## article 9 — no query language

users call functions. they don't write SQL, Cypher, JSON-path, or any custom DSL. if a feature requires a query language, it's the wrong feature.

## article 10 — benchmark honestly

every public performance or accuracy claim is reproducible from a published harness with raw logs. no cherry-picked single numbers. the standard reporting stack is **BLEU + F1 + LLM-judge (binary) + token cost + p95 latency**, plus a noisy-cue degradation test. raw logs and the harness commit hash ship with every claim.

## article 11 — small core, composable surface

the public API is the `Memory` trait: `observe`, `observe_procedure`, `recall`, `recall_procedure`, `what_about`, `between`, `consolidate`, `close`. additions require an ADR. removals require a major version.

## article 12 — non-goals are sacred

these are explicitly **not** sochdb and never will be in v0.x:

- a general-purpose database
- a transactional store for orders/users/payments
- a full-text search engine over documents
- a pure similarity search over fixed embeddings
- a hosted-only service
- a multimodal store (text-first; v0.3+ may add image/audio)
- a distributed/sharded database
- a knowledge graph editor with a UI
- a fine-tuning service
- a query language

if a customer needs one of these, the answer is "use the right tool, then sochdb on top."

## article 13 — the decision gate is binding

at week 12, the project commits, repositions, or retreats based on the published thresholds in [phases/phase-7-decision-gate.md](../phases/phase-7-decision-gate.md). the gate is not negotiable.

## article 14 — respect existing ctxgraph code

vendor what's reusable from ctxgraph (GLiNER ONNX loading, predicate canonicalization, alias resolution). do not rewrite for the sake of rewriting. do not block on ctxgraph parity — sochdb can advance independently.

## article 15 — amendments

this constitution may be amended by a written ADR in `docs/adr/`. amendments must explain:

1. what the old principle said
2. what the new principle says
3. what concrete decision forced the amendment
4. what consequences follow

amendments are dated. amendments cannot retroactively reinterpret prior commitments.
