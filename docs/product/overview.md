# sochdb — product overview

## the problem

AI agents are getting longer-running, more autonomous, and more stateful. an agent that helps you write code, manage your inbox, or run your business needs to remember things across days, weeks, and years. it needs to remember what you told it, what it tried, what worked, who said what, and when.

today, every agent solves this badly. the standard pattern is:

1. embed every conversation into vectors
2. store them in a vector database (Pinecone, Qdrant, Weaviate, pgvector)
3. at recall time, embed the query, do similarity search, get back the top-k chunks
4. optionally rerank with another LLM call
5. stuff the chunks into the prompt and hope the model finds the right one

this pipeline has six problems:

- **latency.** every recall is multiple network calls. p95 is often 1-3 seconds. some systems (LangMem) hit 60 seconds.
- **cost.** every recall is multiple embedding API calls plus context-window tokens. for an agent doing 10,000 recalls/day, this is a real bill.
- **no temporal grounding.** vector dbs don't know what was true when. if you told the agent in January that you live in Mumbai, and in May that you moved to Berlin, a vector search for "where do I live" returns both with no way to know which is current.
- **no provenance.** vector retrieval returns chunks of text without strong attribution. the agent can confidently claim a fact whose source is impossible to trace.
- **no graceful degradation.** if the cosine similarity threshold isn't met, the query returns nothing. the agent then either hallucinates or asks the user a question it should have known the answer to.
- **no consolidation.** the vector db grows without bound. there's no equivalent of sleep — no compaction of episodic memory into semantic memory, no decay of unused facts, no contradiction detection.

these aren't bugs. they're properties of the wrong primitive. vector databases were designed for RAG over documents, not for agent memory over time.

## the solution

sochdb is a new database primitive designed specifically for agent memory. it replaces the six-step pipeline above with one function call.

```
sochdb.recall("what did sarah say about thai food?")
   → returns the matching memory + confidence + provenance
```

the call runs locally, in under 50 milliseconds on a laptop, with zero network calls. it returns matches with explicit confidence scores. if confidence is low, you know. if there's no match, you get the nearest neighbors anyway.

sochdb integrates binding and recall without an external index — storage and retrieval share the same representation, so retrieval doesn't detour through a separate vector or graph lookup. this is the architectural wedge.

internally, sochdb borrows three ideas from how biological memory works:

1. **content-addressable storage.** memories are stored as high-dimensional binary signatures. retrieval is bit-overlap counting, not query parsing. you give the db a partial pattern; it returns the full pattern. this is how your hippocampus works.

2. **bi-temporal supersession.** new facts don't overwrite old ones. they get marked as the current version with `t_valid_start = now`, and the old fact gets `t_valid_end = now - 1` and `superseded_by = new_id`. you can query the db "as of" any historical date. this is how legal and financial systems track changing facts, and it's how human memory works — you remember that you used to live in Mumbai *and* that you live in Berlin now.

3. **sleep-like consolidation.** a background worker periodically scans episodic signatures, bundles repeated patterns into semantic concepts, decays unused memory, and flags contradictions. this is what your hippocampus does during sleep — the McClelland-McNaughton-O'Reilly "complementary learning systems" model from cognitive neuroscience.

## who sochdb is for

**developers building AI agents** who currently use mem0, Letta, Zep, Cognee, LangMem, or a hand-rolled vector-db-plus-graph stack. sochdb gives them:
- faster recall (sub-50ms p95 vs 200ms-3s)
- lower cost (no embedding API calls in the read path, no token tax)
- better grounding (bi-temporal + provenance)
- simpler API (one function vs a pipeline)

**developers building local-first / offline-first AI applications** — coding agents, desktop assistants, on-device personal AIs — who can't or won't depend on cloud services for memory. sochdb runs fully offline by default. zero network. zero API keys.

**developers in regulated industries** — healthcare, legal, finance — who need auditable memory with full provenance and non-destructive updates. every claim in sochdb traces back to a verbatim source observation. contradictions are preserved, not silently overwritten.

## who sochdb is not for

- **applications that need a general-purpose database.** sochdb is a memory db for agents. it is not a transactional store for orders or users.
- **applications that need full-text search over a document corpus.** use tantivy or elastic.
- **applications doing pure similarity search over fixed embeddings.** use lancedb or qdrant. sochdb's signature is computed from extracted structure, not from a learned embedding.
- **applications that need a hosted service today.** sochdb is embedded-first. a cloud tier is on the roadmap but not a v0.1 priority.

## comparison to alternatives

### vs Mem0

Mem0 is the velocity leader in agent memory (51,800 GitHub stars, $24M funded, used as the default memory layer in the AWS Agent SDK). it's a strong general-purpose choice. sochdb's differences:

- **embedded vs hosted-first.** Mem0 is primarily a hosted SaaS with an open-source python sdk. sochdb is embedded, rust, single binary.
- **no LLM in the read path.** Mem0 uses LLMs for extraction at write time and (optionally) at read time. sochdb uses LLMs only optionally at write time, never at read time.
- **bi-temporal supersession.** Mem0 supports updates but not first-class historical querying.
- **content-addressable retrieval.** Mem0 is hybrid vector + graph + key-value with rule-based extraction. sochdb uses hyperdimensional signatures as the unified primitive.

### vs Zep / Graphiti

Zep/Graphiti is the temporal-knowledge-graph leader (25,759 GitHub stars). it has bi-temporal grounding and runs against Neo4j or Kuzu. sochdb's differences:

- **no external graph database.** Zep depends on Neo4j or Kuzu. sochdb is self-contained in one binary.
- **content-addressable recall.** Zep uses Cypher-style traversals plus embeddings. sochdb uses HDC signatures.
- **rust vs python.** Zep is python. sochdb is rust top to bottom.

### vs Letta (formerly MemGPT)

Letta is the OS-inspired memory leader ($10M Felicis seed, #1 on Terminal-Bench). it tiers memory like RAM/disk and lets the agent self-edit. sochdb's differences:

- **memory primitive vs full agent runtime.** Letta is an agent framework with memory built in. sochdb is just the memory layer — composable with any agent framework.
- **embedded vs hosted.** Letta is primarily a cloud product.
- **no self-editing.** Letta has the agent issue memory tool calls (memorize, remember). sochdb makes memory a database operation, not a tool call.

### vs Cognee

Cognee ($7.5M seed led by Pebblebed, Feb 2026; 12K+ stars) does Extract-Cognify-Load over vector + graph. sochdb's differences:

- **pure-rust vs python.** Cognee has a rust engine for edge devices on their roadmap but hasn't shipped it. sochdb is rust from day one.
- **HDC signatures vs hybrid vector + graph.** different fundamental representation.
- **single-binary embedded vs python + LanceDB + Kuzu.** simpler operational story.

### vs LangMem

LangMem is the LangChain memory layer. it's slow — Mem0's published benchmarks show p95 search latency of 59.82 seconds, "rendering it impractical for interactive applications" (Chhikara et al., ECAI 2025). sochdb targets p95 under 50ms.

## why now

three things changed between 2024 and 2026 that make sochdb a viable product:

1. **agent memory became a category.** mem0, letta, cognee, zep all raised real funding in 2024-2026. the market knows it needs this.
2. **HDC/VSA research matured for production use.** Torchhd shipped in 2023. PathHD demonstrated encoder-free HDC retrieval over knowledge graphs in December 2025. HPE's "Hippocampus" paper (Feb 2026) showed binary-signature retrieval beating vector databases by 31× latency and 14× token cost. the math is no longer experimental.
3. **the embedded-db category got serious in rust.** duckdb, lancedb, redb, surrealdb, tigerbeetle all matured. an embedded memory db in rust is now a natural fit for the ecosystem.

## non-goals

sochdb is deliberately *not*:

- a general-purpose database
- a hosted cloud service (in v0.1)
- a replacement for vector databases for RAG
- a search engine over documents
- a multimodal store (text first; images/audio deferred)
- a distributed/sharded database (single-node only in v0.1)
- a knowledge graph editor with a UI

these may come later. they are not the v0.1 product.

## what success looks like

at the end of 6 months, sochdb should:

- match or beat Zep/Graphiti on LongMemEval-S (≥ 64 accuracy)
- have ≥ 3× lower retrieval latency than Mem0 (target: p95 under 50ms)
- have ≥ 3× lower token cost than Mem0 (target: under 2,500 tokens/query)
- support 1M+ episodes on a laptop with sub-100ms p99 recall
- be installable as `cargo add sochdb` and `pip install sochdb`
- expose an MCP server so any MCP-compatible agent can use it as a tool
- have full bi-temporal querying and a `consolidate()` API working end to end
- have 500+ GitHub stars and 3+ design-partner deployments

if these targets are met, sochdb is a real product worth raising a seed round on. if they aren't met, it's a learning that gets folded back into ctxgraph and the architecture moves on.

see [ROADMAP.md](./roadmap.md) for the milestone plan.
