# sochdb

> sochdb is an embedded, content-addressable memory database for AI agents — storage and retrieval share the same hyperdimensional representation, bi-temporal by default, with automatic consolidation. one binary, one api, no query language.

[![Crates.io](https://img.shields.io/crates/v/sochdb.svg)](https://crates.io/crates/sochdb)
[![Docs](https://docs.rs/sochdb/badge.svg)](https://docs.rs/sochdb)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

## what is sochdb

sochdb is a new category of database — a **content-addressable memory database** designed for AI agents that need to remember instead of query.

most databases force you to write queries: SQL, Cypher, JSON paths, vector embeddings. sochdb doesn't. you give it observations in plain text, and you retrieve memories by partial cue. retrieval is one function call, runs in milliseconds on a laptop, makes zero network calls, and never returns "no results" — only matches with explicit confidence.

```rust
let db = sochdb::open("./memory.soch").await?;

db.observe("Sarah recommended a thai place called Bawri in Bandra last weekend").await?;

let result = db.recall("what thai place did sarah mention?").await?;
// → Bawri, confidence 0.94, with provenance back to the original observation
```

no SQL. no Cypher. no embedding API calls. no separate vector db. no rerank step. one function.

sochdb integrates binding and recall without an external index — storage and retrieval share the same representation, so retrieval doesn't detour through a separate vector or graph lookup.

## why sochdb exists

every existing database was designed for a different consumer:

- **postgres / mysql** for accountants doing ledger transactions
- **mongodb** for app developers managing semi-structured documents
- **neo4j / graphiti** for analysts running ad-hoc graph traversals
- **pinecone / qdrant / pgvector** for retrieval pipelines doing similarity search

none of them were designed for an autonomous AI agent that needs to remember things over months and years, with provenance, with temporal grounding, and with graceful degradation under uncertainty. agents using these databases pay a tax on every recall — embed the query, hit the vector db, fetch chunks, rerank, post-process, stuff into the prompt. six steps, multiple network calls, seconds of latency, dollars of token cost.

sochdb collapses that pipeline into one local function call.

## key properties

- **embedded.** one rust binary, runs locally, no server required. like sqlite, not like postgres.
- **content-addressable.** retrieval by partial cue, not by query. like remembering a song from humming three notes.
- **bi-temporal by default.** every fact has valid-time (when it was true) and transaction-time (when we learned it). contradictions supersede rather than overwrite. you can always ask "what did we believe about X on date Y?"
- **all five biological memory tiers.** episodic (events with time/place/people), semantic (consolidated facts), procedural (workflows and skills), working (session-scoped recall with recency boost). sensory memory is explicitly upstream. see [docs/product/biological-mapping.md](./docs/product/biological-mapping.md).
- **no LLM in the read path.** recall is deterministic math over stored signatures. no API keys, no network calls, no hallucination at retrieval time.
- **tiered confidence.** `recall()` never returns empty. it returns exact match → similarity match → gist → nearest neighbors, each with explicit confidence scores.
- **automatic consolidation.** a background worker compacts repeated episodic patterns into semantic concepts, decays unused memory, flags contradictions. the db manages its own working set.
- **full provenance.** every claim traces back to the verbatim observation that produced it. no opaque embeddings, no untraceable facts.

## quickstart

```bash
cargo add sochdb
```

```rust
use sochdb::{Sochdb, Query};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = Sochdb::open("./memory.soch").await?;

    db.observe("Sarah told me about Letta's three-tier memory at PyCon", None).await?;
    db.observe("Sarah's favourite thai place is Bawri in Bandra", None).await?;

    let recall = db.recall(Query::cue("what did sarah say about memory?")).await?;
    for r in recall.matches {
        println!("[{:.2}] {}", r.confidence, r.text);
    }

    Ok(())
}
```

## the architecture in one paragraph

sochdb is built in three layers. **layer 1** is the mind-like layer: every memory becomes an 8192-bit hyperdimensional signature, and retrieval is bit-overlap counting (POPCOUNT) over stored signatures. **layer 2** is the scaffolding: a local GLiNER ONNX model extracts entities and relations from observations so that signatures encode meaning, not phrasing — "Sarah recommended Bawri" and "Bawri was recommended by Sarah" produce the same signature. **layer 3** is the storage: redb for metadata and bi-temporal indexes, mmap'd flat files for signatures. the user only ever sees layer 1.

See [docs/architecture/architecture.md](./docs/architecture/architecture.md) for the full picture.

## documentation

full docs live in [`docs/`](./docs/README.md). entry points:

- [docs/product/overview.md](./docs/product/overview.md) — product overview, who it's for, comparisons
- [docs/architecture/architecture.md](./docs/architecture/architecture.md) — system architecture and data flow
- [docs/product/biological-mapping.md](./docs/product/biological-mapping.md) — how sochdb maps to the five biological memory tiers
- [docs/architecture/layer-1-recall.md](./docs/architecture/layer-1-recall.md) — the mind-like layer (HDC signatures, recall)
- [docs/architecture/layer-2-extraction.md](./docs/architecture/layer-2-extraction.md) — the scaffolding (GLiNER, triples)
- [docs/architecture/layer-3-storage.md](./docs/architecture/layer-3-storage.md) — the storage layer (redb, mmap)
- [docs/spec/tech-spec.md](./docs/spec/tech-spec.md) — rust API, types, performance targets
- [docs/spec/constitution.md](./docs/spec/constitution.md) — immutable project principles
- [docs/product/roadmap.md](./docs/product/roadmap.md) — v0.1 → v1.0 milestones
- [docs/phases/](./docs/phases/README.md) — per-phase build plan (phase 0 through phase 8)

## status

sochdb is pre-alpha. v0.1 targets a 6-month build to a benchmark-credible release against Mem0, Zep/Graphiti, and Letta on LongMemEval-S, LoCoMo, and BEAM. see [docs/product/roadmap.md](./docs/product/roadmap.md) for the full plan.

## license

Apache-2.0
