# layer 2 — extraction (the scaffolding layer)

this layer turns natural language observations into structured triples that can be bound into robust signatures. it is the **preprocessing step** that makes layer 1's HDC retrieval work well.

without good extraction, layer 1's signatures would encode phrasing rather than meaning. "Sarah recommended Bawri" and "Bawri was recommended by Sarah" would produce different signatures even though they mean the same thing. with good extraction, both reduce to the same triple `(Sarah, recommended, Bawri)` and produce the same signature.

## the role of this layer

layer 2 has exactly one job:

> take a natural-language observation, produce a list of structured triples with attached metadata (entities, relations, time anchors, confidence scores).

it does **not**:
- store anything (that's layer 3)
- compute signatures (that's layer 1)
- decide what to retrieve (that's the user via layer 1)

it is a stateless transformation, runnable on any input.

## why GLiNER, not an LLM

the most common pattern in the agent-memory category right now (Mem0, Cognee, Letta, A-Mem) is to use an LLM for extraction at write time. send the observation to GPT-4o or Claude, prompt it to return JSON, parse the result. this works but has four problems:

1. **hallucination at write time.** LLMs can and do invent entities and relations that aren't in the source text. once an invented fact is bound into a signature, it becomes a permanent part of the agent's memory and propagates downstream.
2. **API key dependency.** every observation triggers a network call. agents that need to work offline can't.
3. **cost.** at scale, every write is a few cents of LLM compute. for an agent observing thousands of events per day, this adds up.
4. **latency.** even fast LLMs add 200-500ms per write. sochdb's target is sub-200ms total, with most of that budget spent on signature math, not extraction.

GLiNER (Generalist Lightweight Named-entity Identification) solves all four:

1. **discriminative, not generative.** GLiNER finds spans in the input text that match entity types. it cannot fabricate entities not present in the text. it can mis-classify or miss, but it cannot invent.
2. **runs locally via ONNX.** no API key, no network call.
3. **free at inference time.** runs on CPU, no per-call cost.
4. **fast.** ~150ms per observation on a modern laptop CPU, sub-50ms with quantization.

GLiNER v2.1 is the production model. it handles zero-shot entity types (you give it a label and it finds spans for that label) and runs efficiently as an ONNX graph. ctxgraph already uses GLiNER; sochdb inherits that.

**LLM extraction is offered as an optional alternative** for users who want richer extraction at the cost of network calls. defaults off. when enabled, sochdb routes observations through a configurable LLM (OpenAI, Anthropic, Ollama, local vllm) and accepts the LLM's triples — but still marks them with provenance noting the LLM source so the user can audit.

## what gets extracted

from each observation, layer 2 produces:

```rust
struct Extraction {
    entities: Vec<Entity>,
    triples:  Vec<Triple>,
    time:     Option<TimeAnchor>,
    confidence: f32,    // overall extraction confidence
}

struct Entity {
    span:        (usize, usize),       // byte offsets in source text
    surface:     String,                // raw text of the span
    canonical:   String,                // resolved canonical name
    entity_type: String,                // "Person", "Location", "Concept", ...
    confidence:  f32,
}

struct Triple {
    subject:     EntityRef,
    predicate:   String,                // "recommended", "located_in", ...
    object:      EntityRef,
    confidence:  f32,
}

struct TimeAnchor {
    valid_start: DateTime<Utc>,
    valid_end:   Option<DateTime<Utc>>, // None = open-ended
    source_span: (usize, usize),        // the text that gave us the time
    confidence:  f32,
}
```

example. observation:

```
"Sarah recommended a thai place called Bawri in Bandra last weekend"
```

extraction:

```
entities:
  - { surface: "Sarah",  canonical: "Sarah",  type: Person,     conf: 0.97 }
  - { surface: "Bawri",  canonical: "Bawri",  type: Restaurant, conf: 0.91 }
  - { surface: "Bandra", canonical: "Bandra", type: Location,   conf: 0.99 }

triples:
  - { Sarah, recommended, Bawri,        conf: 0.94 }
  - { Bawri, type,        thai_restaurant, conf: 0.71 }
  - { Bawri, located_in,  Bandra,       conf: 0.88 }

time:
  - { valid_start: 2026-05-09T00:00Z, valid_end: 2026-05-11T00:00Z, conf: 0.82 }

overall_confidence: 0.86
```

## entity resolution and canonicalization

raw GLiNER output gives us text spans. but "Sarah", "sarah", "Sarah Chen", and "Ms. Chen" might all refer to the same person. layer 2 canonicalizes:

1. **string normalization.** lowercase, strip punctuation, collapse whitespace.
2. **alias lookup.** check the alias index in redb — has this surface form mapped to a canonical name before?
3. **fuzzy match.** if no exact alias, compute Levenshtein distance to known canonical names. if within threshold, link.
4. **HDC similarity match.** for ambiguous cases (multiple matches above threshold), compute the partial signature of the observation and compare to existing concept signatures for each candidate. pick the one with highest contextual overlap.
5. **fallback: create new canonical.** if no match, mint a new canonical entity. mark with `is_new: true` so the consolidation worker can flag it for review later.

canonicalization is *not* perfect. mistakes happen — two different Sarahs may get merged, or one Sarah may get split into two. sochdb mitigates by:

- storing the original surface form alongside the canonical (you can always disambiguate later)
- exposing a `merge_concepts(a, b)` and `split_concept(c, criterion)` API
- flagging low-confidence canonicalizations in the consolidation log

this is consistent with the project-wide principle: **errors should be detectable and reversible, not hidden.**

## relation extraction

GLiNER's primary job is entity identification. for relations, sochdb uses a small library of patterns:

1. **dependency parsing.** a lightweight parser (we use `rust-bert` with a distilled model, or fall back to regex patterns) identifies subject-verb-object structures.
2. **template matching.** common patterns ("X said Y", "X is in Y", "X is a kind of Y") are recognized via templates.
3. **predicate canonicalization.** the surface predicate ("recommended", "suggested", "told me about") is mapped to a canonical predicate atom via a small synonym table maintained in redb.

predicate atoms are themselves hypervectors. semantically similar predicates can be linked via shared atoms or via a small learned similarity matrix (deferred to v0.2 — in v0.1 we use exact synonym match).

## time extraction

every observation has a transaction time (when sochdb received it — automatic) and an optional valid time (when the fact was true in the world — extracted if present).

time extraction uses a hybrid approach:

1. **explicit timestamps.** ISO-8601 strings, dates ("2026-05-09"), or absolute references get parsed directly.
2. **relative times.** "yesterday", "last weekend", "three weeks ago" are resolved against the observation time using `chrono` and a small grammar.
3. **implicit times.** "last summer", "when I was in college" — these are uncertain. sochdb marks them with low confidence and a wide valid-time window.
4. **no time present.** if no temporal information is found, valid_time defaults to "from now onwards" (open-ended).

bi-temporal storage requires being honest about both axes. transaction time is always certain (the system clock); valid time is often uncertain. layer 2 records that uncertainty explicitly.

## confidence and how it propagates

every output from layer 2 carries a confidence score:

- entity confidence comes from GLiNER's softmax output
- relation confidence is the product of subject confidence, object confidence, and pattern strength
- time confidence is high for explicit dates, low for vague references
- overall extraction confidence is a function of the above

these confidences travel with the triple all the way through layer 1 binding and layer 3 storage, ending up surfaced to the user in the `recall()` result.

**confidence is not optional.** every claim in sochdb has a confidence score. the agent decides what threshold matters for its use case.

## robustness — how sochdb avoids hallucinated facts

the six-layer defense recapped from the project overview:

1. **deterministic extractor by default.** GLiNER is discriminative — it cannot generate text not in the input.
2. **confidence-gated insertion.** low-confidence triples are still stored, but marked, and downweighted at retrieval.
3. **provenance.** every triple links to the source observation; users can always verify.
4. **bi-temporal supersession.** errors don't destroy correct data; they're superseded with timestamps.
5. **consolidation cross-checks.** the background worker flags outliers against semantic consensus.
6. **optional strict mode.** for high-stakes applications, only triples with confidence ≥ threshold are bound into the primary signature.

the result: sochdb's agent-memory layer is significantly more trustworthy than an LLM-extracted memory. fabrication is structurally impossible in the default configuration; the only failure modes are extraction misses (recoverable via gist fallback in layer 1) and canonicalization errors (recoverable via merge/split API).

## what's deferred to v0.2+

- **learned predicate similarity.** v0.1 uses synonym tables; v0.2 may add a small learned matrix.
- **coreference resolution beyond simple pronouns.** v0.1 handles "she" → most recent female entity; richer coreference is v0.2.
- **multilingual extraction.** v0.1 is English-first; multilingual GLiNER models exist and can be plugged in.
- **multimodal extraction.** v0.1 is text-only; image and audio caption ingestion is v0.3+.
- **active learning.** v0.1 has no feedback loop to improve extraction; v0.2 may add user corrections that adjust the alias and predicate tables.

## next reads

- [LAYER_3_STORAGE.md](./layer-3-storage.md) — where the triples end up on disk
- [LAYER_1_RECALL.md](./layer-1-recall.md) — how triples become signatures and get retrieved
- [TECH_SPEC.md](../spec/tech-spec.md) — the rust types for `Extraction`, `Entity`, `Triple`, `TimeAnchor`
