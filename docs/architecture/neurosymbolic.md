# agidb — Neurosymbolic Interface

> The bidirectional layer connecting agidb's HDC representations (signatures)
> to structured symbolic representations (triples, beliefs, atoms). Why both
> matter, how the translation works, and how queries blend the two.

## The premise

agidb stores cognition in two complementary forms:

| Form | What it is | Strengths |
|---|---|---|
| **Signatures (HDC)** | 8192-bit binary hypervectors | fast, robust to noise, content-addressable, similarity-by-bit-overlap, compositional via VSA binding |
| **Triples (symbolic)** | `(subject, predicate, object)` with confidence and provenance | exact match, structured queries, explainable, easy to display to humans |

Neither form alone is enough. Pure HDC loses the explicit structure needed for explaining beliefs and tracing provenance. Pure symbolic loses the gracefulness of similarity-based recall and the compositional algebra of VSA. agidb stores both, with explicit translation between them.

This makes agidb a **neurosymbolic system** in the literal sense: subsymbolic (continuous/distributed) and symbolic (discrete/structured) representations coexist with first-class translation operators.

## Why neurosymbolic matters

Consider two queries:
- "Show me everything I know about Sarah." → wants exact match on the `Sarah` concept and its connected facts. Symbolic wins.
- "Show me episodes that felt like the dinner at Bawri." → wants similarity-based gist retrieval over the experiential signature of that episode. HDC wins.

A single agent needs both. mem0 ships only embedding similarity. zep ships only knowledge-graph traversal. agidb ships both, with a first-class API for combining them.

In v2.1, the neurosymbolic interface extends to multimodal signatures: video, audio, and text components of an episode can be individually unbound from the bound signature and translated into structured representations.

## The five translation directions

### 1. Triple → Signature (write path)

When a triple gets stored, agidb binds it into a signature via role-filler binding:

```rust
fn triple_to_signature(triple: &Triple) -> HV {
    let subj_hv = concept_hv(&triple.subject);
    let pred_hv = predicate_hv(&triple.predicate);
    let obj_hv  = value_hv(&triple.object);

    ROLE_SUBJ.bind(&subj_hv)
        ^ ROLE_PRED.bind(&pred_hv)
        ^ ROLE_OBJ.bind(&obj_hv)
}
```

`ROLE_*` are fixed random 8192-bit HVs seeded at init. The triple's signature is the XOR-sum of role-bound concept signatures.

Bundling multiple triples into one episode:
```rust
fn triples_to_episode_signature(triples: &[Triple]) -> HV {
    let triple_sigs: Vec<HV> = triples.iter().map(triple_to_signature).collect();
    bundle(&triple_sigs)  // per-bit majority vote
}
```

### 2. Signature → Triple (read path, learned)

Going back from a signature to its component triples requires VSA factorization. agidb uses two methods:

**Method A — Cleanup memory.** For each role, XOR the episode signature with the role HV to recover the noisy filler, then cleanup with nearest-neighbor lookup in the concept codebook:
```rust
fn extract_subject(episode_sig: &HV, concept_codebook: &Codebook) -> Option<ConceptId> {
    let noisy = episode_sig.bind(&ROLE_SUBJ);  // XOR
    concept_codebook.nearest_neighbor(&noisy, threshold = 0.7)
}
```

**Method B — Learned probes.** For more complex unbinding (e.g. recovering relational structure from highly bundled signatures), agidb v2.2+ may train small MLPs as "signature-to-triple probes." Out of scope for v2.0/v2.1.

In v2.0/v2.1, every episode signature has its triples stored directly in `redb` alongside it (in the `episodes` table). So in practice, `signature → triple` is just "look up the triples we already stored." The signature is the search key; the triples are the retrieved structure.

### 3. Signature → Multimodal components (v2.1, new)

In v2.1, episode signatures are bound from multiple modality signatures. Any modality component can be recovered via XOR with its role HV:

```rust
fn extract_video_signature(episode_sig: &HV) -> HV {
    episode_sig.bind(&ROLE_VIDEO)  // XOR — recovers approximate sig_video
}

fn extract_audio_signature(episode_sig: &HV) -> HV {
    episode_sig.bind(&ROLE_AUDIO)
}

fn extract_text_signature(episode_sig: &HV) -> HV {
    episode_sig.bind(&ROLE_TEXT)
}
```

The recovered signatures are noisy approximations (because other modalities are still XOR'd in). Clean up via nearest-neighbor lookup against per-modality codebooks of stored signatures.

**Why this matters:**
- Query: "show me episodes where the audio sounded like X" → bind audio_query with ROLE_AUDIO, search for nearest episodes that produce a clean audio sig when unbound. Possible *only* with VSA binding; impossible with attention fusion.
- Ablation: "what did the video contribute to this episode?" → extract just the video signature, compare against silent-baseline.
- Debugging: "why did this episode rank high?" → factor by modality, see which component matched.

### 4. Cue (natural language) → Partial signature (read path)

When the user calls `recall("what did sarah say about thai food?")`, agidb extracts a partial triple shape:

```
Cue: "what did sarah say about thai food?"
↓ GLiNER + lightweight parser
Partial triple: { subj: ConceptId(Sarah), pred: ?, obj: thai food }
↓ binding (skipping unknowns)
Partial signature: ROLE_SUBJ ⊕ Sarah_HV ⊕ ROLE_OBJ ⊕ ThaiFood_HV
```

This partial signature is the search key. Episodes whose stored signatures have high overlap with this partial signature are tier B matches.

### 5. Belief → Signature (and back)

Beliefs are stored with both a structured form and a signature:

```rust
fn belief_to_signature(belief: &Belief) -> HV {
    let triple_sig = triple_to_signature(&Triple {
        subject: belief.subject,
        predicate: belief.predicate.clone(),
        object: belief.object.clone(),
    });
    let confidence_sig = ROLE_CONFIDENCE.bind(&confidence_quantized_hv(belief.confidence));

    triple_sig ^ confidence_sig
}
```

This means belief signatures can be compared via HDC similarity AND queried via structured `what_do_i_believe()`. Same data, two access patterns.

## The hybrid query API

The neurosymbolic interface exposes a unified hybrid query:

```rust
pub struct NeurosymbolicQuery {
    pub structured: Option<TriplePattern>,
    pub fuzzy_cue: Option<String>,
    pub weights: HybridWeights,
}

pub struct HybridWeights {
    pub structured: f32,  // [0, 1]
    pub fuzzy: f32,       // [0, 1]
}

impl Agidb {
    pub async fn neurosymbolic_query(
        &self,
        query: NeurosymbolicQuery
    ) -> Result<Recall>;
}
```

Internally, the query runs both retrieval paths and combines them:

```
1. STRUCTURED PATH (if pattern present)
   - Match TriplePattern against the triples table
   - Returns Vec<EpisodeId> with exact-match confidence

2. FUZZY PATH (if cue present)
   - Extract partial signature from cue (translation direction #4)
   - Tier B/C/D HDC similarity search
   - Returns Vec<EpisodeId> with similarity confidence

3. COMBINE
   - Union of episode IDs
   - For each, combined_confidence = w_s * structured_conf + w_f * fuzzy_conf
   - Re-rank by combined_confidence
```

Example usage:
```rust
// Pure structured query (HybridWeights { structured: 1.0, fuzzy: 0.0 })
let r1 = db.neurosymbolic_query(NeurosymbolicQuery {
    structured: Some(TriplePattern {
        subject: Some(ConceptId(Sarah)),
        predicate: Some("recommends".into()),
        object: None,
    }),
    fuzzy_cue: None,
    weights: HybridWeights::structured_only(),
}).await?;

// Pure fuzzy query (HybridWeights { structured: 0.0, fuzzy: 1.0 })
let r2 = db.neurosymbolic_query(NeurosymbolicQuery {
    structured: None,
    fuzzy_cue: Some("the dinner where sarah suggested thai food".into()),
    weights: HybridWeights::fuzzy_only(),
}).await?;

// Hybrid: 50/50
let r3 = db.neurosymbolic_query(NeurosymbolicQuery {
    structured: Some(TriplePattern { subject: Some(ConceptId(Sarah)), predicate: None, object: None }),
    fuzzy_cue: Some("food recommendation".into()),
    weights: HybridWeights::balanced(),
}).await?;
```

The default `recall()` API uses `HybridWeights { structured: 0.7, fuzzy: 0.3 }` — structured wins when triples match, fuzzy fills in when they don't.

## V-JEPA 2 ↔ symbolic translation (v2.1)

In v2.1, multimodal episodes bring an additional symbolic translation challenge: turning V-JEPA 2's dense visual latents into something a structured query can match.

**The agidb approach:** don't try to translate V-JEPA latents to triples directly. Instead:
1. V-JEPA latent → 8192-bit signature (via Charikar 2002 random projection).
2. The signature is the bridge: agidb stores it bound into the episode HV.
3. Symbolic queries match against the *triples* stored alongside (which came from text extraction).
4. Fuzzy queries match against the *signature* (which incorporates the video).
5. The hybrid query handles both.

For pure visual queries ("show me episodes where the video looked like X"), the user provides a video query → V-JEPA → signature → tier-C/D HDC search. No symbolic translation needed; the signature suffices.

For mixed queries ("what did sarah say in episodes where the room was crowded?"), the structured component matches text-derived triples (sarah, said, X), the fuzzy component matches the visual signature (crowded room), and the hybrid query returns episodes scoring well on both axes.

## The OpenCog Hyperon comparison

OpenCog Hyperon (MeTTa over AtomSpace) is the closest neurosymbolic neighbor. Differences:

| dimension | hyperon | agidb |
|---|---|---|
| symbolic layer | AtomSpace metagraph + MeTTa language | typed triples + bi-temporal supersession |
| subsymbolic layer | numeric truth values on atoms | 8192-bit HDC signatures with VSA binding |
| query language | MeTTa pattern rewriting | Rust API, no query language (constitution IX) |
| translation | implicit via atom truth values | explicit via the 5 translation directions above |
| storage | Distributed Atomspace (research) | redb + mmap (production) |
| multimodal | not first-class | first-class in v2.1 via V-JEPA 2 + Wav2Vec-BERT + VSA binding |
| audience | academic AGI research | developers building agents today |

Hyperon's neurosymbolic interface is deep but research-oriented. agidb's is shallower but production-grade. Different points on the same trade-off curve.

## Why explicit translation matters

Most "neurosymbolic" systems hide the seam. The user gives a query, the system internally decides whether to match structured or fuzzy, and returns a result. The translation is invisible.

agidb makes the seam explicit and addressable:
- The user can specify `HybridWeights { structured: 1.0, fuzzy: 0.0 }` for pure SQL-like queries.
- The user can specify `HybridWeights { structured: 0.0, fuzzy: 1.0 }` for pure similarity recall.
- The user can use the default 0.7/0.3 for the common case.
- The user can extract triples from a signature for explainability.
- The user can extract a modality signature from a bound episode for ablation.

Explicit seams are auditable. Invisible seams are convenient until they fail mysteriously.

## What this enables

| Capability | How |
|---|---|
| Exact-match queries | structured path with weights (1.0, 0.0) |
| Fuzzy recall | fuzzy path with weights (0.0, 1.0) |
| "What did I learn at the meeting yesterday?" | hybrid: structured on time-range, fuzzy on cue |
| Explainability ("why did this match?") | extract triples back from signature |
| Belief tracing ("what evidence supports this belief?") | structured query on belief table |
| Compositional reasoning ("X is to Y as Z is to ?") | VSA analogy binding |
| Modality-specific retrieval (v2.1) | factor episode signature by modality, search per-modality |
| Cross-modal queries (v2.1) | hybrid over structured + multi-modality fuzzy |
| Brain-aligned retrieval (v2.1) | BAMS scores can attribute alignment to specific modality components |

## What this doesn't try to do

- agidb does not try to learn the translation. The translation is explicit and deterministic. (Learned translation is v2.2+ territory.)
- agidb does not try to be a full logic programming system. No Prolog, no Datalog, no MeTTa. Translation is a substrate primitive; reasoning is the agent's job.
- agidb does not try to embed all of MeTTa's expressivity. We accept the narrower scope to ship a production substrate.

## The phase 12 deliverable

Implementing the neurosymbolic interface is phase 12 (weeks 26-27) of the v2.0 build:
- `agidb-ns` crate (already scaffolded)
- Five translation functions: triple_to_signature, signature_to_triples, cue_to_partial_signature, belief_to_signature, signature_to_modality (v2.1)
- `neurosymbolic_query` API on `agidb-core`
- Property tests: bind-then-unbind roundtrip, hybrid query weighting consistency, modality factorization
- Documentation: this doc + ADR-0013

The exit criterion: hybrid queries with 50/50 weights return appropriately blended results, with the structured component matching the triples table and the fuzzy component matching the signatures table.

In v2.1, the multimodal factorization extension lands in phase 14 alongside the multimodal sensory encoders. Same translation framework, extended with ROLE_VIDEO / ROLE_AUDIO / ROLE_TEXT unbinding.
