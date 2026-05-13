# layer 1 — recall (the mind-like layer)

this is the layer that makes sochdb feel like a mind instead of a database. it implements the brain-inspired primitive at the core of the system: content-addressable retrieval through hyperdimensional computing.

if you only read one layer doc, read this one — it's the differentiator.

## the core idea

every memory stored in sochdb becomes a single **8192-bit binary vector** called a *signature*. retrieval works by finding stored signatures that overlap (in bits) with a partial signature computed from the query.

there is no query language. there is no separate index. the signature *is* the index. retrieval is bit-counting.

## why hyperdimensional computing

hyperdimensional computing (HDC), also called vector symbolic architectures (VSA), is a 30-year-old line of research started by Tony Plate (HRR, 1995), Pentti Kanerva (Sparse Distributed Memory, 1988; Hyperdimensional Computing, 2009), and Ross Gayler (VSA, 2003). it has three properties that conventional ML doesn't:

1. **content-addressable.** given a partial pattern, the system retrieves the nearest complete pattern. this is exactly what human memory does and what databases don't.

2. **algebraically composable.** you can bind two vectors into a "role-filler" pair, bundle vectors into sets, and unbind to recover components — all with deterministic math, no learning. compositions are still hypervectors and can be retrieved like any other.

3. **graceful degradation.** noisy or partial queries still return reasonable results. the system doesn't have a binary hit/miss threshold; it has a continuous confidence gradient.

modern relevance: Bricken & Pehlevan (NeurIPS 2021) proved that transformer attention is mathematically equivalent to Kanerva's sparse distributed memory. PathHD (arXiv 2512.09369, December 2025, UC Irvine Imani group) demonstrated encoder-free knowledge graph retrieval with HDC matching neural baselines at 40-60% lower latency. HPE's Hippocampus paper (arXiv 2602.13594, Feb 2026) showed binary-signature retrieval beating vector databases on agent-memory benchmarks.

the math is solid, the references are real, and the productization gap is wide open. sochdb is the bet that this is the right primitive for agent memory in rust, embedded.

## the building blocks

### concept vectors (atoms)

every entity, relation, and time anchor that sochdb knows about has a fixed 8192-bit hypervector called an *atom*. atoms are generated once and never change.

```
Sarah_hv        : 8192-bit vector, ~50% bits set, random
Bawri_hv        : 8192-bit vector, ~50% bits set, random
recommended_hv  : 8192-bit vector, ~50% bits set, random
SUBJ_hv         : 8192-bit vector, ~50% bits set, random
PRED_hv         : 8192-bit vector, ~50% bits set, random
OBJ_hv          : 8192-bit vector, ~50% bits set, random
TIME_hv         : 8192-bit vector, ~50% bits set, random
2026-05-09_hv   : 8192-bit vector, derived from date encoding
```

atoms are pseudo-random — generated deterministically from a hash of the canonical name, so the same entity always gets the same atom across runs. two unrelated atoms are nearly orthogonal (hamming distance ~4096 out of 8192).

### binding (⊗)

binding combines two hypervectors into one, producing a *role-filler* pair. binding is implemented as XOR for binary spatter codes (BSC):

```
A ⊗ B  =  A XOR B
```

properties:
- the result is approximately orthogonal to both A and B
- binding is its own inverse: `(A ⊗ B) ⊗ B = A`
- binding is commutative for XOR — order doesn't matter (use permutation for order-aware binding)

we use binding to attach roles to fillers:

```
SUBJ ⊗ Sarah       = "Sarah in the subject role"
PRED ⊗ recommended = "recommended in the predicate role"
OBJ  ⊗ Bawri       = "Bawri in the object role"
```

### bundling (⊕)

bundling combines multiple hypervectors into one *set-like* superposition. bundling is implemented as majority vote for binary vectors:

```
A ⊕ B ⊕ C  =  per-bit majority of A, B, C
```

properties:
- the result is similar (in hamming distance) to each input
- the result is *not* equal to any input — it's a superposition
- you can ask "is X in the bundle?" by computing hamming(bundle, X) — if it's much less than 4096, yes

we use bundling to combine triples into an episode:

```
episode = (SUBJ⊗Sarah ⊕ PRED⊗recommended ⊕ OBJ⊗Bawri)
        ⊕ (SUBJ⊗Bawri ⊕ PRED⊗located_in ⊕ OBJ⊗Bandra)
        ⊕ (TIME ⊗ 2026-05-09)
```

### unbinding for retrieval

if you have the bundled episode and you know the role you want to query, you can *unbind* to get back a noisy estimate of the filler:

```
episode ⊗ SUBJ  ≈  Sarah  (noisy)
```

the result isn't exactly Sarah's atom — it's contaminated by the other parts of the bundle. but it's close enough to Sarah's atom that a cleanup step (modern Hopfield or hamming nearest-neighbor over known atoms) recovers the clean Sarah_hv.

this is how you do "what did sarah say?" — bind a probe pattern, hamming-search against stored signatures, return matches.

## the recall pipeline in detail

### tier A — exact match

if the query mentions a known canonical entity ("Sarah", "Bawri"), we first check the concept index in redb:

```
concept_index[Sarah] → [episode_id_1, episode_id_42, ...]
```

if there are exact matches with high enough confidence, return them immediately. this is the fast path — microseconds. handles "tell me about Sarah" trivially.

confidence: 1.0 if canonical match, drops if multiple entities share the name (disambiguation needed).

### tier B — similarity match (the main path)

if tier A misses or confidence is low, we compute a partial signature from the query and run a hamming-distance scan.

steps:

1. **compute query signature.** GLiNER extracts the partial triple shape from the query. unknown slots (the thing being asked about) are left blank. bind known slots:

```
query_sig = (SUBJ⊗Sarah ⊕ PRED⊗mentioned ⊕ OBJ⊗?)
          ⊕ (TYPE ⊗ thai_restaurant)
```

2. **first-pass filter via inverted index.** the inverted index in redb maps each active dimension to the list of episode_ids whose signatures have that bit set. we intersect the posting lists for the top-K most active dims in the query signature, giving us a candidate set of (typically) a few hundred episodes — not the full 100k+ database.

3. **POPCOUNT hamming distance.** for each candidate, XOR the query signature against the stored signature and count the set bits. AVX-512 POPCOUNT processes 512 bits per instruction; a full 8192-bit comparison is 16 instructions and runs in nanoseconds.

```
hamming(a, b) = popcount(a XOR b)
similarity    = 1 - (hamming / 8192)
```

4. **rank and threshold.** top-K candidates by similarity. confidence is derived from the hamming distance: a perfect match is similarity 1.0; orthogonal vectors are around 0.5. anything above ~0.7 is a meaningful match.

5. **return.** typically 5-50ms p95 on a laptop for 100k stored episodes.

### tier C — gist match (fallback)

if tier B returns no high-confidence matches, sochdb falls back to a *gist signature* — a separate signature computed from the raw text of the observation (via sparse hashing of tokens) rather than from the structured triples. gist signatures are less precise but capture meaning even when entity extraction missed something.

confidence at this tier is capped at 0.7 — sochdb is explicitly telling the user "i found something kind of relevant but the structured match failed."

### tier D — nearest neighbors (last resort)

if every tier fails to find a confident match, sochdb returns the K nearest signatures by hamming distance, all marked with `low_confidence: true`. **sochdb never returns an empty result set.** the agent always has something to work with, and always knows how much to trust it.

this is the "graceful degradation" property that distinguishes content-addressable memory from query-based memory. a SQL query that misses returns zero rows. a vector search below threshold returns empty. sochdb returns the nearest available memory with explicit confidence — the agent can decide what to do with it.

## why this works — the math intuition

three intuitions worth carrying:

**1. high dimensions are sparse.** in 8192 dimensions, two random vectors are almost certainly orthogonal. this means atoms don't interfere with each other much — binding `SUBJ ⊗ Sarah` doesn't accidentally look like `OBJ ⊗ Bawri` because the role atoms and filler atoms are independent random patterns.

**2. bundles preserve approximate membership.** when you bundle 10 hypervectors, each one is still recognizable in the result — the hamming distance from any member to the bundle is around 30% (not 0%, not 50%), well below the orthogonal baseline. this is what makes retrieval-by-overlap work even after lots of bundling.

**3. binding is a clean inverse.** because XOR is its own inverse, unbinding a role from a bundle gives back a noisy copy of the filler. the noise is bounded and predictable, and a cleanup step recovers the clean atom from a small known dictionary.

these three properties together give you the brain-like behavior: store many composite memories in one bundle, retrieve by partial cue, get a noisy result, clean it up. all with simple bitwise operations.

## performance characteristics

| operation | target | mechanism |
|---|---|---|
| signature compute (write) | < 1ms | XOR + majority over ~10 triples |
| inverted-index lookup | < 1ms | roaring bitmap intersection |
| POPCOUNT scan over 100k candidates | < 5ms | AVX-512, 16 instructions per pair |
| recall end-to-end (tier B) | p50 < 20ms, p95 < 50ms | dominated by GLiNER on query |
| signature size on disk | 1 KB | 8192 bits |
| 1M episodes on disk | ~1 GB | linear in episode count |
| 1M episode hamming scan | < 50ms | with inverted-index pre-filter |

these are achievable on a single laptop CPU. no GPU. no network.

## what about scale beyond 1M episodes?

for very large memory stores (10M+ episodes), the linear hamming scan starts to dominate. mitigations, in order of increasing complexity:

1. **inverted-index pre-filter** (already in v0.1). cuts candidate set to a few hundred regardless of total store size.
2. **locality-sensitive hashing (LSH).** index signatures by hashed projections of their active dims. enables sub-linear nearest-neighbor.
3. **HNSW over binary vectors.** the `hnsw_rs` crate works on hamming distance. log-scale recall above 10M.
4. **Dynamic Wavelet Matrix (DWM).** the structure described in the HPE Hippocampus paper. enables compressed search over very large stores. deferred to v1.0+.

v0.1 targets 1M episodes with the inverted-index path. anything beyond is a v0.2 problem.

## comparison to alternatives

| approach | retrieval mechanism | latency at 100k | model dependency |
|---|---|---|---|
| sochdb (HDC) | POPCOUNT hamming + bitmap intersect | 20-50ms | none (encoder-free) |
| dense vector DB (Pinecone, Qdrant) | cosine similarity over learned embeddings | 50-200ms | embedding model required |
| knowledge graph (Zep, Graphiti) | Cypher traversal + reranking | 200ms-1s | LLM for extraction + traversal |
| LLM-judged retrieval (LangMem) | LLM scores candidates | 5-60s | LLM in hot path |
| Mem0 (hybrid) | multi-signal: vector + graph + KV | 100-500ms | LLM for extraction, optional rerank |

sochdb's tradeoff: no learned semantic generalization (a learned embedding will match "physician" to "doctor"; sochdb won't unless the extractor links them). mitigation: tier C gist + a small learned concept-similarity layer in v0.2.

## the episodic-semantic split in retrieval

sochdb stores both **episodic memories** (specific events with time and provenance) and **semantic atoms** (consolidated facts decoupled from specific events). these are biologically distinct in the brain — episodic memory lives in the hippocampus initially, semantic memory consolidates to the neocortex — and they answer different questions in agent retrieval.

an agent asks: *"what does sarah like to eat?"*

- **episodic answer:** "on April 12, sarah said she liked the thai place." "on April 28, sarah ordered thai again." "on May 9, sarah recommended Bawri."
- **semantic answer:** "sarah likes thai food. (evidence: 7 episodes, confidence 0.91)"

both are correct. they answer different questions. the agent might want one, the other, or both.

### how sochdb returns both

`recall()` returns a `Recall` struct with two parallel result lists:

```rust
pub struct Recall {
    pub matches:         Vec<RecallMatch>,    // episodic matches
    pub semantic_atoms:  Vec<SemanticMatch>,  // consolidated facts
    pub tier_used:       Tier,
    pub elapsed_ms:      u32,
}
```

both lists are populated in one query. the agent decides which to use, or uses both. typical patterns:

- *"give me the latest on X"* → use `matches`, sorted by `valid_time` descending
- *"what's generally true about X"* → use `semantic_atoms`, sorted by confidence
- *"answer this question"* → use both; prefer semantic if evidence count is high, fall back to episodic

### when an episode becomes a semantic atom

the consolidation worker (described in ARCHITECTURE.md) is what bridges the two. it scans recent episodes, finds clusters of similar bound patterns, and produces a `SemanticAtom` when:

- evidence count ≥ 3 (configurable)
- cluster cohesion (mean pairwise hamming distance) below threshold
- no contradictions within the cluster

once a semantic atom exists, future recalls on the same concept return it alongside any new episodic evidence. the semantic atom is updated (`evidence_count++`) but not replaced — provenance to source episodes is preserved.

### why two lists, not one ranked list

we could merge episodes and semantic atoms into a single ranked result. mem0 mostly does this. we don't, for three reasons:

1. **type is information.** the agent often needs to know "this is a remembered specific event" vs "this is consolidated general knowledge." different downstream behavior.
2. **provenance is different.** an episode points to one observation. a semantic atom points to many. surfacing them separately makes the evidence trail clear.
3. **confidence is computed differently.** episodic confidence comes from extraction and HDC similarity; semantic confidence comes from evidence count and cluster cohesion. mixing them in a single score hides the difference.

## working memory — session scoping and recency

biological working memory is the active context — what the agent is currently thinking about. it's not a separate store; it's *attention* over the long-term store. sochdb supports this with two mechanisms.

### session scoping

every observation can be tagged with a `session_id`:

```rust
db.observe(
    "the user asked about deploying to staging",
    ObserveOpts {
        provenance: Some(Provenance {
            session_id: Some("sess_abc123".into()),
            ..Default::default()
        }),
        ..Default::default()
    },
).await?;
```

queries can scope to a session, or boost results from a session:

```rust
// scope: only recall from this session
db.recall(Query::cue("deploy")
    .session_id("sess_abc123")
    .session_only(true)
).await?;

// boost: include all results, but rank current session higher
db.recall(Query::cue("deploy")
    .session_id("sess_abc123")
    .session_boost(2.0)
).await?;
```

### recency weighting

even without explicit session scoping, sochdb applies a small recency boost by default. the formula:

```
final_confidence = base_confidence × recency_factor

recency_factor = 1.0 + 0.2 × exp(-Δt / τ)
```

where Δt is the time since the episode's transaction time and τ is a half-life parameter (default: 1 hour for working-memory feel, configurable up to 7 days for longer-horizon memory).

this gives recent episodes a small confidence boost without overwhelming high-confidence older episodes. "what did i just say?" surfaces the last few observations; "what does sarah like?" pulls in older consolidated knowledge with equal weight.

### the result — working-memory feel without a separate store

with these two mechanisms, an agent using sochdb gets:

- a session feels live — recent observations rank higher than equivalent older ones
- the agent can scope to "what happened in this conversation" or open up to "what do we know in general"
- there is no separate working-memory store to manage, evict, or sync with long-term storage

this is the brain's pattern: working memory is not a separate substrate; it's the recently-activated part of the same memory system, with elevated attention.

## next reads

- [LAYER_2_EXTRACTION.md](./layer-2-extraction.md) — how triples get extracted before binding
- [LAYER_3_STORAGE.md](./layer-3-storage.md) — how signatures live on disk
- [TECH_SPEC.md](../spec/tech-spec.md) — the rust types and traits
