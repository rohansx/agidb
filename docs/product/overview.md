# agidb — Overview

> The product overview. What agidb is, who it's for, what it replaces, how it
> compares to the competition, and why now. Brain-alignment is the v2.1
> additive milestone, not the founding story.

## What agidb is, in one minute

agidb is a database with a single purpose: be the persistent memory and cognitive state of an autonomous AI agent. It is not a vector database, not a graph database, not a key-value store. It is a **cognitive substrate** — built around the seven things an autonomous agent needs to persist (sensory input, working memory, episodic memories, semantic facts, procedural skills, goals and beliefs, and a self-model audit log), each as a first-class typed shape.

The agent talks to agidb through one API: `observe()` to record, `recall()` to retrieve, `set_goal()` / `assert_belief()` / `unlearn()` for the cognitive primitives. No SQL. No Cypher. No embedding API calls. No separate vector store. No rerank step. Everything is local, deterministic, sub-50ms, with full provenance.

agidb v2 inherits sochdb v1's working HDC kernel, bi-temporal storage, episode binding, tiered recall, and consolidation. The v2.0 pivot adds the cognitive primitives (goals, beliefs, sensory buffer, self-model, unlearn, neurosymbolic interface) that make it an AGI substrate. **agidb v2.1 extends this with brain-aligned multimodal sensory encoding** (V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B, the same encoder stack as Meta FAIR's TRIBE v2 brain-encoding model) and ships the BAMS benchmark — the first agent memory evaluation grounded in human cortical activation patterns.

## Who it's for

**Primary: developers building autonomous AI agents who have outgrown mem0/letta/zep.** Their pattern is: agent loop calls mem0 for memory, parses responses, glues in graphiti for relations, hand-rolls a vector store for embeddings, juggles three sets of credentials, hits 1-3 second p95 latencies on recall, and pays embedding API costs per query. They want one binary, one API, deterministic retrieval, and cognitive primitives the existing stack doesn't provide.

**Secondary: teams building toward AGI.** Frontier-adjacent startups and research groups who need a substrate where goals, beliefs, and self-model are first-class types — not bolted on. The audience is smaller but higher-value, and the multi-year positioning is what justifies the AGIDB name.

**Tertiary: developers in regulated industries.** Healthcare, legal, finance — anyone who needs auditable memory with full provenance, non-destructive updates, and an unlearn API to handle right-to-be-forgotten compliance. agidb is the first agent memory layer that takes these seriously from v0.1.

**Also: local-first / offline-first builders.** Coding agents (Claude Code, Cursor), desktop assistants, on-device personal AIs — anyone who cannot depend on cloud services for memory. agidb is a single Rust binary, runs entirely local, no API keys.

**New audience in v2.1: cognitive science researchers and NeuroAI labs.** Anyone benchmarking agent memory architectures against human cortical ground truth. agidb is the first system shipping with BAMS evaluation built in.

## Who it's not for

- People who want a general-purpose database. Use postgres.
- People who want a knowledge graph editor with a UI. Use neo4j browser.
- People who want pure document RAG with rerankers. Use any vector DB + cohere.
- People who want a multimodal-document store. Use lancedb or weaviate.
- People who want a hosted-only managed service. agidb is embedded-first; hosted is v0.4+.
- People who want a distributed sharded database. agidb is single-node by design.

Use the right tool for those problems, then put agidb on top.

## What agidb replaces

A typical agent memory stack today is six components glued together:

```
Agent → embedding API call → vector DB query →
        graph DB query →
        rerank LLM call →
        synthesis LLM call →
        result
```

p95 latency: 1-3 seconds. Per-query cost: $0.001-$0.01 in API calls. Provenance: weak. Temporal grounding: none. Goal awareness: zero. Belief revision: nobody handles it.

agidb replaces all six with one local function call:

```
Agent → agidb.recall(cue) → result with provenance and confidence
```

p95 latency: under 50ms. Per-query cost: zero (local CPU). Provenance: complete. Temporal grounding: bi-temporal by default. Goal awareness: recall is goal-biased. Belief revision: first-class with audit.

## Comparisons

### vs Mem0

| dimension | mem0 | agidb |
|---|---|---|
| storage | vector DB + graph DB + KV cache | one Rust binary, redb + mmap |
| retrieval | semantic similarity over embeddings | content-addressable HDC, bit-overlap counting |
| dependencies | LLM API + embedding API + vector DB | none (LLM optional, write-time only) |
| latency p95 | 1-3 seconds (API-dependent) | < 50ms (local) |
| token cost | $0.001-0.01 per recall | $0 |
| temporal grounding | flat timestamps, mutable | bi-temporal valid+tx, non-destructive supersession |
| consolidation | none | surprise-gated background worker |
| first-class goals | no | yes (state machines) |
| first-class beliefs | no | yes (revisable with audit) |
| unlearn API | DELETE | non-destructive cascading with audit |
| self-model | no | append-only learning event log + self-vector EMA |
| multimodal sensory | yes (LLM-extracted) | yes (V-JEPA 2 + Wav2Vec-BERT, factorable via VSA) [v2.1] |
| brain-alignment | no | BAMS benchmark + brain-calibrated surprise [v2.1] |
| embedded | no | yes |
| funding | $24M total across Seed + Series A (Oct 2025) | bootstrap → seed at week 12 if gate passes |
| stars (May 2026) | 41K | starting at 0, target 1000+ at v2.0 launch, 5000+ at v2.1 launch |

### vs Letta (formerly MemGPT)

| dimension | letta | agidb |
|---|---|---|
| paradigm | LLM-as-OS with memory tiers | brain-inspired substrate |
| storage | core/recall/archival memory blocks in postgres | typed cognitive floors in redb + mmap |
| retrieval | LLM-orchestrated memory paging | deterministic HDC + tiered fallback |
| latency | model-bound (LLM in the loop) | sub-50ms (no LLM in read path) |
| stateful agents | yes — that's their wedge | yes (as a side effect) |
| first-class goals | tools + agent state | first-class typed state machines |
| first-class beliefs | text in core memory | first-class revisable |
| unlearn | edit core memory | non-destructive cascading + self-vector subtraction |
| brain-alignment | no | yes [v2.1] |
| embedded | requires server | yes |
| funding | $10M seed (Sept 2024, Felicis lead) | bootstrap → seed |
| stars (May 2026) | ~22K | starting at 0 |

Letta is a stateful agent runtime that happens to have memory. agidb is a memory substrate that any stateful agent runtime can sit on top of. They are complements, not direct competitors — but most teams will need to pick one for the day-1 build.

### vs Zep / Graphiti

| dimension | zep/graphiti | agidb |
|---|---|---|
| storage | temporal knowledge graph on Neo4j/Kuzu/FalkorDB | embedded, redb + mmap, no graph DB dependency |
| temporal model | 4-timestamp bi-temporal edges | 4-timestamp bi-temporal columns on every fact |
| retrieval | cypher-based + vector hybrid | content-addressable HDC, no query language |
| LLM dependency | yes (extraction + retrieval) | extraction only (write-time), no LLM at read |
| latency | LLM-bound at read | sub-50ms at read |
| consolidation | rebuilds knowledge graph | surprise-gated semantic atom creation |
| first-class goals/beliefs | no | yes |
| unlearn | DELETE the graph node | non-destructive cascading + self-vector subtraction |
| brain-alignment | no | yes [v2.1] |
| embedded | no (requires graph DB) | yes |
| stars (May 2026) | 25,759 | starting at 0 |

Zep got the bi-temporal pattern right and ships it well. agidb shares that pattern but doesn't require a separate graph database — bi-temporal is a column on every row, the graph is implicit in the inverted index, and the cognitive primitives sit on top of the same substrate.

### vs Cognee

| dimension | cognee | agidb |
|---|---|---|
| target | ML-engineering teams | autonomous agents |
| storage | pluggable backends (NetworkX/Neo4j/Kuzu/FalkorDB + vector) | single Rust binary, redb + mmap |
| paradigm | knowledge-graph-first | cognitive-substrate-first |
| LLM dependency | yes (multiple roles) | extraction only (write-time) |
| first-class goals/beliefs | no | yes |
| brain-alignment | no | yes [v2.1] |
| funding | €7.5M seed (Feb 2026, Pebblebed lead) | bootstrap → seed |
| rust engine | on roadmap | shipping |
| stars (May 2026) | ~12K | starting at 0 |

### vs MemMachine / MemOS / Hindsight

These are 2025/2026-vintage open-source memory systems achieving high scores on LongMemEval/LoCoMo. MemMachine reports 91.69% LoCoMo with gpt-4.1-mini, MemOS reports 35.24% token savings, Hindsight 20/20 91.4% LongMemEval. **All are Python frameworks operating above the LLM.** agidb is a Rust substrate operating beneath the agent loop. Different layer entirely.

### vs HippoRAG / HippoMM (the brain-inspired neighbors)

| dimension | hippoRAG | hippoMM | agidb |
|---|---|---|---|
| neural-symbolic | KG + PPR (personalized pagerank) | dentate gyrus + CA3 abstractions | HDC binding + signatures |
| modality | text | audiovisual | text now, multimodal in v2.1 |
| retrieval mechanism | graph traversal via PPR | pattern completion | tiered HDC cascade |
| consolidation | none | dual-process | surprise-gated semantic atoms |
| unlearn | none | none | first-class cascading |
| performance claim | "10 to 30× cheaper, 6 to 13× faster than IRCoT" (NeurIPS 2024) | 78.2% HippoVlog, 5× faster than RAG | sub-50ms p95, 8× smaller than dense |
| code | OSU-NLP-Group/HippoRAG | linyueqian/HippoMM | agidb/agidb |
| substrate vs application | application on LLM | application on LLM | substrate beneath agent |

agidb's brain-alignment in v2.1 is more rigorous than hippoRAG/hippoMM — it doesn't just *claim* hippocampal inspiration, it benchmarks against TRIBE v2 cortical predictions via RSA.

### vs OpenCog Hyperon (the AGI substrate from the academic world)

| dimension | hyperon | agidb |
|---|---|---|
| paradigm | metagraph + MeTTa language | cognitive substrate + Rust API |
| AGI claim | explicit ("Baby Hyperon → adolescent → adult") | implicit (substrate, not full AGI) |
| backing | SingularityNET / Goertzel | bootstrap, deep-tech VCs |
| developer onramp | hard (MeTTa, AtomSpace, metagraph theory) | easy (cargo add, pip install) |
| audience | academic | developers building production agents |
| status | active research, no killer app | targeting benchmark-credible v2.0 at month 9 |
| productization | low | high (this is the whole point) |

Hyperon is the closest intellectual neighbor and the only other open AGI substrate. They are deeper on theory, slower on productization, and target academic researchers. agidb targets developers building agents today, with theory backing the design but not gating the API.

### vs Numenta thousand brains / Monty

| dimension | monty | agidb |
|---|---|---|
| paradigm | sensorimotor learning, cortical columns, reference frames | HDC cognitive substrate |
| language | python | rust |
| target | sensorimotor robotics + embodied AI | agent memory + cognition |
| code | thousandbrainsproject/tbp.monty | agidb/agidb |
| funding | Gates Foundation | bootstrap |
| brain-alignment claim | architectural (cortical columns) | empirical (BAMS RSA against TRIBE v2) [v2.1] |

Complementary. Monty handles the perceptual front-end; agidb handles the persistent cognitive substrate.

### vs agentmemory (rohitg00) — the rust neighbor

| dimension | agentmemory | agidb |
|---|---|---|
| language | rust | rust |
| storage | RocksDB | redb + mmap |
| retrieval | BM25 + HNSW hybrid | HDC tiered cascade |
| interface | MCP server | embedded library + MCP server + CLI + pyo3 |
| cognitive primitives | none | goals, beliefs, sensory, self-model |
| temporal model | flat | bi-temporal supersession |
| unlearn | DELETE | non-destructive cascading + self-vector subtraction |
| brain-alignment | no | yes [v2.1] |
| LoCoMo claim | 87.8% (self-reported) | TBD post-decision-gate |

Closest competitor on the language axis. But agentmemory is a memory *server*, not a *substrate*. Positioning: "agentmemory is a rust memory server; agidb is a rust cognitive substrate with first-class cognitive primitives, bi-temporal supersession, and brain-aligned multimodal sensory."

## Why now

Five converging trends make May 2026 the right moment:

**1. Agent memory became a category in 2024-2025.** Mem0 raised $24M total in October 2025 across two announced rounds (Seed led by Kindred Ventures, Series A led by Basis Set Ventures with Peak XV / GitHub Fund / Y Combinator). Letta raised $10M seed in September 2024 (Felicis lead, with Jeff Dean, Clem Delangue, Ion Stoica among angels). Zep, Cognee (€7.5M seed in Feb 2026 led by Pebblebed), Supermemory, Graphiti, MemoryOS, MemMachine all have funded teams and production users. The category exists; the cognitive-substrate wedge is empirically unoccupied.

**2. HDC/VSA research matured for production.** Torchhd (JMLR 2023) is the canonical HDC library. Karunaratne et al. 2020 in Nature Electronics demonstrated in-memory HDC at scale. PathHD (December 2025) showed structured composition over hypervectors at scale. The math is settled; the productization gap is open.

**3. The embedded-database renaissance is real in Rust.** redb (1.0 stable since June 2023) is the right default for embedded ACID storage. LanceDB, surrealdb, tigerbeetle all proved the embedded-rust pattern works in production. agidb fits the same niche: single binary, embedded-first, sqlite-grade ergonomics.

**4. Frontier labs are not building externalizable substrates.** Anthropic's September 2025 memory tool is a CRUD interface over a `/memories` file directory — explicitly not a database. OpenAI's April 2025 ChatGPT memory upgrade is a product feature. Google's Personal Context in Gemini is a product feature. **No frontier lab is shipping a vendor-neutral substrate.** The wedge is open.

**5. NEW: Brain-encoding foundation models matured in March 2026.** Meta FAIR released TRIBE v2 with open weights, predicting fMRI BOLD across 720 subjects from V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B. This makes brain-aligned evaluation tractable for the first time. agidb v2.1 is built on the same encoder stack to inherit alignment. The brain-alignment benchmark (BAMS) is now a paper-sized contribution. **No other agent memory system can ship this because none of them are HDC-binding-first.**

## What agidb is not claiming

To be precise about what agidb is and isn't:

- **agidb is not AGI.** It is the database AGI will run on top of. The model layer, the reasoning layer, the action layer — those are separate concerns. agidb provides the substrate; somebody else (probably a frontier lab) provides the cognition.
- **agidb is not a research project.** It is production infrastructure. Every claim is reproducible; every API is shippable. The research happens in academic papers along the way; the product ships every week.
- **agidb is not a complete cognitive architecture in v2.0.** v2.0 is the substrate. The cognitive engine extensions (pattern completion, analogical reasoning, belief revision with formal semantics) are v2.2+. v2.0 is the minimum credible cognitive substrate, not the complete one.
- **agidb v2.1's brain-alignment is empirical, not aspirational.** We don't claim agidb "thinks like a brain." We claim agidb's internal representations align with TRIBE-predicted cortical activations on matched stimuli, measurable via RSA across six functional networks. That's a defensible empirical claim with a reproducible benchmark, not a marketing slogan.
- **TRIBE v2 is not "alphafold for neuroscience".** Predicting BOLD (a hemodynamic proxy lagged ~5s behind neural activity) is not the same as predicting cognition itself. TRIBE achieves ~54% of the noise ceiling on out-of-distribution movies. That's a real result, not a discontinuous jump. agidb integrates TRIBE for evaluation purposes; we don't inherit its hype.

## What success at month 9 looks like (v2.0)

- agidb v2.0 launched publicly with arxiv whitepaper
- Match/beat Zep/Graphiti on LongMemEval-S (≥ 64 accuracy)
- ≥ 3× lower retrieval latency than Mem0 (p95 < 50ms)
- ≥ 3× lower token cost than Mem0 (< 2,500 tokens/query)
- All four cognitive benchmarks pass with documented thresholds
- 1000+ GitHub stars in week 1
- 5+ design-partner deployments
- `cargo add agidb` + `pip install agidb` both work
- MCP server in the official MCP registry

## What success at month 12 looks like (v2.1)

- v2.0 success criteria all hold
- `agidb-sensory` ships with V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B integration
- Multimodal `observe_multimodal()` works on a laptop (≤ 2s for 30s video+audio clip)
- Brain-calibrated surprise threshold released with reproducible calibration recipe against TRIBE v2
- BAMS benchmark suite open-source with baseline scores (mem0, letta, zep, hippoRAG, raw V-JEPA latents)
- agidb wins BAMS in associative-cortex networks (DMN, dorsal attention, frontoparietal)
- ICLR 2026 MemAgents workshop paper accepted, or CCN 2026 oral presentation
- 5000+ GitHub stars cumulative
- 10+ design-partner deployments
- Seed round closed ($1-3M from a deep-tech-friendly fund)

## The 5-year vision

agidb v2.0 (2026) is the substrate. v2.1 (2026) is brain-aligned multimodal sensory + the BAMS benchmark. v2.2-v2.5 is the path to AGI-grade: pattern completion as first-class operation, belief revision with formal semantics, analogical reasoning via HDC binding, causal claim storage, world model fragments, closed-loop self-modification, formal safety guarantees, production-grade enterprise tier. See [agi-trajectory.md](./agi-trajectory.md) for the full path.

That 5-year vision is what justifies the AGIDB name. The 12-month v2.1 launch is what justifies the next step.
