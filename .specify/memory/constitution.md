# sochdb Constitution

> Immutable principles that govern every decision in sochdb. If a feature, dependency, or design choice contradicts the constitution, it is out of scope. Amendments require an explicit ADR in `docs/adr/`.

## Core Principles

### I. The One-Liner

sochdb is **an embedded, content-addressable memory database for AI agents — storage and retrieval share the same hyperdimensional representation, bi-temporal by default, with automatic consolidation. One binary, one API, no query language.**

If a proposed feature does not reinforce this one-liner, it does not ship.

### II. The Wedge (NON-NEGOTIABLE)

sochdb integrates binding and recall without an external index — storage and retrieval share the same representation, so retrieval doesn't detour through a separate vector or graph lookup. This is the non-negotiable architectural commitment.

Violations:
- Introducing a separate vector index (pgvector, qdrant, lancedb) as a primary read path
- Introducing a separate graph store (neo4j, kuzu) as a primary read path
- Requiring an LLM call on the read path

### III. Embedded-First, Forever

sochdb is a library that runs in the user's process. Like sqlite, not like postgres.
- Single binary, no server
- Works fully offline by default
- Zero required network calls at read or write time
- No required API keys
- A hosted tier is permitted **only as an optional v1.0+ deployment mode**; the embedded engine must remain the canonical product

### IV. No LLM in the Read Path

`recall()`, `what_about()`, `between()`, `recall_procedure()` must complete with zero LLM invocations.

LLMs are permitted only:
- Optionally, at write time, behind an explicit feature flag, for extraction enrichment
- In the eval harness, never in the production path

### V. Bi-Temporal Supersession Over Destructive Update

Every fact has `t_valid_start`, `t_valid_end`, `t_tx_start`. Contradictions supersede, never overwrite. The answer to "what did we believe about X on date Y?" must always be answerable.

### VI. Never Return Empty

`recall()` returns matches at one of four tiers: exact → similarity → gist → nearest-neighbor. Confidence is always explicit. A query never returns the empty set — it returns the nearest neighbors with `low_confidence=true`.

### VII. Provenance Always

Every claim sochdb makes traces back to a verbatim source observation. Opaque embeddings as the only provenance are forbidden. Consolidated semantic atoms must link back to their source episode IDs.

### VIII. Rust Top to Bottom

The engine is Rust. Bindings (Python via pyo3, MCP) wrap the Rust engine. No Python or JavaScript in `sochdb-core`.

Permitted exceptions: ONNX runtime (`ort` crate) and tokenizers — both already pure-Rust wrappers.

Forbidden: GC-language reimplementations of core paths, async-std (use tokio), C++ deps when a Rust equivalent exists.

### IX. No Query Language

Users call functions. They don't write SQL, Cypher, JSON-path, or any custom DSL. If a feature requires a query language, it's the wrong feature.

### X. Benchmark Honestly (NON-NEGOTIABLE)

Every public performance or accuracy claim is reproducible from a published harness with raw logs. No cherry-picked single numbers. The standard reporting stack is **BLEU + F1 + LLM-judge (binary) + token cost + p95 latency**, plus a noisy-cue degradation test. Raw logs and the harness commit hash ship with every claim.

### XI. Small Core, Composable Surface

The public API is the `Memory` trait: `observe`, `observe_procedure`, `recall`, `recall_procedure`, `what_about`, `between`, `consolidate`, `close`. Additions require an ADR. Removals require a major version.

### XII. Non-Goals Are Sacred

These are explicitly **not** sochdb and never will be in v0.x:
- A general-purpose database
- A transactional store for orders/users/payments
- A full-text search engine over documents
- A pure similarity search over fixed embeddings
- A hosted-only service
- A multimodal store (text-first; v0.3+ may add image/audio)
- A distributed/sharded database
- A knowledge graph editor with a UI
- A fine-tuning service
- A query language

If a customer needs one of these, the answer is "use the right tool, then sochdb on top."

### XIII. The Decision Gate Is Binding

At week 12, the project commits, repositions, or retreats based on the published thresholds in `docs/phases/phase-7-decision-gate.md`. The gate is not negotiable.

### XIV. Respect Existing ctxgraph Code

Vendor what's reusable from ctxgraph (GLiNER ONNX loading, predicate canonicalization, alias resolution). Do not rewrite for the sake of rewriting. Do not block on ctxgraph parity — sochdb can advance independently.

## Additional Constraints

### Performance Targets

sochdb is held to user-perceivable performance contracts at v0.1, measured on the benchmark laptop (Apple M2 or Intel i7-12700H, 16 GB RAM, NVMe SSD):

- `recall` p95 ≤ 50ms on 100k-episode store
- `observe` p95 ≤ 200ms
- 8192-bit hamming-distance scan over 100k signatures ≤ 5ms (AVX-512 or NEON path)
- Binary size ≤ 60 MB rust-stripped, no LLM weights
- Memory footprint at idle ≤ 80 MB (mmap doesn't count toward RSS)

Full table in `docs/spec/tech-spec.md` § Performance Targets.

### Benchmark Suite

Every public release runs the full three-benchmark suite — **LongMemEval-S + LoCoMo + BEAM** — against pinned baselines (Mem0, Zep/Graphiti, Letta) and publishes all six metrics with raw logs. No single-number claims. See `docs/phases/phase-7-decision-gate.md` for the decision thresholds.

### Dependency Constraints

Pure-Rust dependencies wherever a Rust equivalent exists. C/C++ FFI only when no viable Rust crate ships (e.g., ONNX runtime via `ort`). LLM SDKs (`openai`, `anthropic`) forbidden in `sochdb-core`; permitted in `sochdb-extract` only behind a feature flag.

## Development Workflow

### Phase Gating

The build is organized into phases 0-8 (see `docs/phases/`). A phase exits only when its exit criterion is met on a reproducible benchmark. Partial implementations do not exit a phase.

### Test-First Discipline

All new behavior begins with a failing test. Property tests via `proptest` for HDC algebra invariants, supersession, and confidence monotonicity. Unit tests for each crate. Integration tests for the public API surface. CI runs unit + property tests on every PR; benchmarks are gated to nightly.

### Code Review

Every change goes through review against this constitution. PRs that violate a Core Principle are rejected and routed to an ADR discussion.

### ADRs for Amendments

Architectural decisions and constitutional amendments are recorded in `docs/adr/`. Each ADR documents: what the old principle said, what the new principle says, what concrete decision forced the change, and what consequences follow. Amendments are dated and cannot retroactively reinterpret prior commitments.

## Governance

This constitution supersedes all other practices and conventions within the sochdb codebase. PRs and reviews must verify compliance with every Core Principle that touches the change. Complexity must be justified — preferably in an ADR.

Amendments require:
1. An ADR in `docs/adr/` proposing the change
2. Approval logged on the ADR
3. Migration plan for any code or docs affected
4. Update to this constitution with an incremented version

Use `docs/spec/tech-spec.md` for runtime development guidance and the per-phase docs in `docs/phases/` for week-by-week milestones.

**Version**: 1.0.0 | **Ratified**: 2026-05-14 | **Last Amended**: 2026-05-14
