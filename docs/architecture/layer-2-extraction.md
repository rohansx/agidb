# agidb — Layer 2: Extraction

> The scaffolding. Turns raw input (text in v2.0; text + video + audio in
> v2.1) into the structured signatures layer 1 binds. GLiNER for text,
> V-JEPA 2 for video, Wav2Vec-BERT for audio, Llama-3.2-3B for text encoding,
> all projecting to 8192-bit HDC signatures.

## What layer 2 is

Layer 2 sits between the user's raw input and layer 1's signature representation. Its job: turn unstructured input into something compositional.

In v2.0, layer 2 is text-only: GLiNER ONNX extracts entities and relations as typed triples, then layer 1 binds those triples into episode signatures.

In v2.1, layer 2 extends to multimodal: V-JEPA 2 turns video into 1024-d dense latents, Wav2Vec-BERT turns audio into 1024-d latents, Llama-3.2-3B turns text into 2048-d latents. Each latent projects to an 8192-bit HV via Charikar 2002 thresholded random projection. Then layer 1's VSA binding fuses them into one episode signature.

The constitutional rule is that layer 2 never runs at read time (constitution article IV). All extraction is write-time. Read path stays deterministic math over stored signatures.

## v2.0 text extraction pipeline

```
USER text → GLiNER (entities + relations)
         → time anchor parser
         → alias resolver
         → predicate canonicalizer
         → belief extractor (if applicable)
         → Vec<Triple> with confidences
         → layer 1 binds into episode signature
```

### GLiNER ONNX

GLiNER (Generalist and Lightweight model for Named Entity Recognition, Zaratiana et al. 2023) is the chosen extractor. Why:

- **Local** — runs on CPU via ONNX, no API key, no cloud
- **Fast** — ~150ms for typical observation lengths on a laptop
- **Zero-shot for entity types** — define entity schemas at call time, no fine-tuning
- **No hallucination at write time** — extractive only, doesn't invent

```rust
pub struct GLiNERExtractor {
    session: ort::Session,
    entity_types: Vec<String>,
    tokenizer: Tokenizer,
}

impl GLiNERExtractor {
    pub fn extract(
        &self,
        text: &str,
        relation_types: &[&str]
    ) -> Result<Vec<Triple>> {
        let tokens = self.tokenizer.encode(text)?;
        let entity_spans = self.session.run(tokens)?;
        let entities = self.decode_entities(entity_spans)?;
        let triples = self.build_triples(&entities, relation_types);
        Ok(triples)
    }
}
```

Phase 3 implements this. Vendored from ctxgraph (sochdb's predecessor); already working code, port + integration only.

### Time anchor parser

Turns natural-language time expressions into bi-temporal stamps:
- "yesterday" → `valid_time = (yesterday 00:00, yesterday 23:59)`
- "last weekend" → `valid_time = (last Saturday 00:00, last Sunday 23:59)`
- "two months ago" → `valid_time = (2026-03-20)`
- "by next Friday" → deadline annotation on Goal

```rust
pub fn parse_time_anchor(text: &str, observation_time: DateTime<Utc>) -> Option<TimeRange> {
    use chrono_english::parse_date_string;
    parse_date_string(text, observation_time, Dialect::Us)
        .ok()
        .map(|dt| TimeRange::point(dt))
}
```

Phase 3 ships this. Fallback: observation time.

### Alias resolver

Canonicalizes entity names: "Sarah," "Sarah Lee," "Lee," and "S. Lee" all → same `ConceptId`. Uses:
1. Exact match on canonical name (most cases)
2. Levenshtein distance < 3 for typos
3. Embedding similarity for cross-language / nicknames (optional, v0.3+)

```rust
pub fn resolve_alias(&self, mention: &str) -> Result<ConceptId> {
    if let Some(id) = self.store.lookup_concept(mention).await? {
        return Ok(id);
    }
    // ... fuzzy match logic
    let new_concept = Concept::new(mention.to_string());
    self.store.create_concept(new_concept).await
}
```

Phase 3 ships this. Phase 9 extends with belief-derived aliases ("I believe S. Lee is the same as Sarah").

### Predicate canonicalizer

Maps surface predicates to canonical forms:
- "recommended," "suggested," "told me about" → `recommends`
- "lives in," "is from" → `located_in`
- "works at," "is employed by" → `works_at`

Default canonicalization rules come from a curated list. Custom rules per-deployment via config. Phase 3 ships.

### Belief extractor (phase 9)

When extracted triples carry high-confidence patterns ("X said Y," "X believes Y," "X claimed Y"), promote them to `Belief` candidates.

```rust
pub fn extract_beliefs(&self, text: &str, triples: &[Triple]) -> Vec<Belief> {
    let mut beliefs = vec![];
    for t in triples {
        if BELIEF_PREDICATES.contains(&t.predicate.as_str()) {
            beliefs.push(Belief::from_triple(t, /*default_confidence=*/0.7));
        }
    }
    beliefs
}
```

Phase 9 ships this. LLM-based belief extraction (v2.2+) for harder cases.

## v2.1 multimodal extraction pipeline

```
USER (video + audio + text) →
        V-JEPA 2 (1024-d) →┐
        Wav2Vec-BERT (1024-d) →┐ Charikar 2002 random projection (per modality)
        Llama-3.2-3B (2048-d) →┘
                              ↓
                     three 8192-bit HVs
                              ↓
              VSA role-filler binding (layer 1)
                              ↓
                  one 8192-bit episode HV
```

In v2.1, layer 2 grows three new encoders, each producing a dense latent that gets projected to an 8192-bit HV. Layer 1 then binds them.

### V-JEPA 2 video encoder

- **Source:** Meta FAIR, github.com/facebookresearch/vjepa2
- **Size:** 1.2B parameters
- **Input:** 64 frames at 256×256 resolution (typical clip ~3 seconds at 24fps; sample 64 frames uniformly)
- **Output (used for agidb):** 1024-d spatially-averaged latent per clip
- **Backbone:** ViT with 3D rotary position embeddings
- **License:** CC BY-NC

```rust
pub struct VJEPA2Encoder {
    session: ort::Session,
    config: VJEPAConfig,
}

impl VJEPA2Encoder {
    pub fn encode(&self, video_clip: &VideoClip) -> Result<[f32; 1024]> {
        let frames = self.sample_frames(video_clip, 64)?;
        let preprocessed = self.preprocess(frames, 256, 256)?;
        let tokens_8192x1024 = self.session.run(preprocessed)?;
        let mean_pooled = self.spatial_mean_pool(&tokens_8192x1024)?;
        Ok(mean_pooled)
    }
}
```

**Why spatially-averaged not flattened:** TRIBE v2 uses spatial mean pooling. Matching encoder usage = encoder representations cooperate for BAMS evaluation. Full 8192-token output is also accessible for v2.2+ experiments where richer representations are needed.

**Inference cost:** ~1.5s CPU on M2 / i7-12700H per 64-frame clip; ~200ms on GPU (M2 ANE or RTX 4090).

### Wav2Vec-BERT audio encoder

- **Source:** Meta FAIR, huggingface.co/facebook/w2v-bert-2.0
- **Input:** 60s audio chunk at 16kHz
- **Output:** ~50Hz frame-level latents at 1024-d, mean-pooled → single 1024-d vector
- **License:** CC BY-NC

```rust
pub struct Wav2VecBertEncoder {
    session: ort::Session,
}

impl Wav2VecBertEncoder {
    pub fn encode(&self, audio_clip: &AudioClip) -> Result<[f32; 1024]> {
        let waveform = self.resample(audio_clip, 16000)?;
        let frame_latents = self.session.run(waveform)?;
        let mean_pooled = self.temporal_mean_pool(&frame_latents)?;
        Ok(mean_pooled)
    }
}
```

**Inference cost:** ~400ms CPU per 60s clip; ~80ms GPU.

### Llama-3.2-3B text encoder

- **Source:** Meta, huggingface.co/meta-llama/Llama-3.2-3B
- **Input:** up to 1024 tokens of preceding text context
- **Output:** layer-32 mean-pooled hidden state at ~3072-d; project down to 2048-d via fixed linear (or use the mean-pooled last-layer directly)
- **License:** Llama 3.2 community license

```rust
pub struct LlamaEncoder {
    session: ort::Session,
    tokenizer: Tokenizer,
}

impl LlamaEncoder {
    pub fn encode(&self, text: &str) -> Result<[f32; 2048]> {
        let tokens = self.tokenizer.encode(text)?;
        let hidden_states = self.session.run(tokens)?;
        let last_layer = &hidden_states[32];
        let mean_pooled = self.mean_pool_2048(last_layer)?;
        Ok(mean_pooled)
    }
}
```

**Inference cost:** ~200ms CPU per 1024-token window; ~30ms GPU.

**Why Llama-3.2-3B specifically:** TRIBE v2 uses Llama-3.2-3B. Matching = alignment. Larger models (8B, 70B) would be wasteful for feature extraction and break the brain-alignment comparison.

### Charikar 2002 random projection

Each encoder produces a dense latent. agidb projects to 8192-bit signatures via thresholded random projection:

```rust
pub struct HDCProjector {
    matrix: Vec<i8>,    // [-1, +1], flat (8192 × D_INPUT)
    seed: u64,
    d_input: usize,
}

impl HDCProjector {
    pub fn new(d_input: usize, seed: u64) -> Self {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let matrix: Vec<i8> = (0..8192 * d_input)
            .map(|_| if rng.gen_bool(0.5) { 1 } else { -1 })
            .collect();
        Self { matrix, seed, d_input }
    }

    pub fn project(&self, x: &[f32]) -> HV {
        debug_assert_eq!(x.len(), self.d_input);
        let mut sig = HV::zero();
        for bit_idx in 0..8192 {
            let row_start = bit_idx * self.d_input;
            let mut acc: f32 = 0.0;
            for d in 0..self.d_input {
                acc += (self.matrix[row_start + d] as f32) * x[d];
            }
            if acc > 0.0 { sig.set_bit(bit_idx); }
        }
        sig
    }
}
```

**Why this works:**
- **Johnson-Lindenstrauss guarantee.** For random projection matrix R ∈ {-1,+1}^(8192 × D), cosine distance in the original space is approximately preserved in hamming distance over `sign(Rx)`.
- **Charikar 2002** "Similarity Estimation Techniques from Rounding Algorithms" proved this for sign-projection.
- **Deterministic.** Fixed seed → reproducible. Same input → same signature.
- **Training-free.** No learned parameters. Survives encoder upgrades.

**One projector per modality:** different D values (1024 for V-JEPA / Wav2Vec, 2048 for Llama) → different projection matrices. Each has its own seed, stored in `manifest.toml`.

### Layer 1 binding handoff

Each modality's projected HV becomes a filler bound by its role HV. Layer 1's `encode_multimodal_episode()` (see layer-1-recall.md) takes the three HVs and produces one episode signature. Layer 2's job ends at the projection step.

## Belief extraction (phase 9)

Beyond entity-relation extraction, layer 2 also produces `Belief` candidates from text.

Pattern matchers identify belief-like statements:
- "X said Y" → belief with subject=X, predicate=said, object=Y, confidence=0.6
- "X believes Y" → belief with confidence=0.8
- "X claims Y" → belief with confidence=0.5
- "I think X" → belief with subject=self, confidence=0.7

Beliefs flow to floor 6 (Goals + Beliefs) where they enter the revision/audit lifecycle.

```rust
pub fn extract_beliefs(text: &str, triples: &[Triple]) -> Vec<Belief> {
    triples.iter()
        .filter(|t| BELIEF_PREDICATES.contains_key(t.predicate.as_str()))
        .map(|t| {
            let confidence = *BELIEF_PREDICATES.get(t.predicate.as_str()).unwrap();
            Belief::from_triple(t, confidence)
        })
        .collect()
}
```

Phase 9 ships this. v2.2+ may add LLM-based extraction for harder belief patterns.

## Encoder versioning

A v2.1 agidb database stores encoder versions in `manifest.toml`:

```toml
[encoders]
vjepa2 = { version = "gigantic-256-2026-06", weight_sha = "sha256:...", projection_seed = 42 }
wav2vec_bert = { version = "2.0", weight_sha = "sha256:...", projection_seed = 43 }
llama_text = { version = "3.2-3B", weight_sha = "sha256:...", projection_seed = 44 }
gliner = { version = "small-v2.5", weight_sha = "sha256:..." }
```

**Constraint:** an agidb database created with encoder version X cannot be opened by a binary using encoder version Y, unless re-projection is run on all old episodes.

**Migration tool (v2.2+):** `agidb migrate-encoders --from old.agidb --to new.agidb`. For v2.1 ship: documented warning, no automatic migration.

## Performance characteristics (v2.1)

| Operation | CPU (M2 / i7-12700H) | GPU (M2 ANE / RTX 4090) | Notes |
|---|---|---|---|
| GLiNER extraction (300 chars text) | ~150ms | n/a | ONNX, CPU is fine |
| Time anchor parsing | < 1ms | n/a | chrono_english |
| Alias resolution | < 1ms | n/a | hash table |
| Predicate canonicalization | < 100µs | n/a | trie lookup |
| Belief extraction | < 1ms | n/a | pattern match over triples |
| V-JEPA 2 (64 frames, 256×256) | ~1.5s | ~200ms | dominates v2.1 latency |
| Wav2Vec-BERT (60s @ 16kHz) | ~400ms | ~80ms | |
| Llama-3.2-3B (1024 tokens) | ~200ms | ~30ms | |
| Charikar projection (1024-d) | ~1ms | ~0.1ms | SIMD-friendly |
| Charikar projection (2048-d) | ~2ms | ~0.2ms | |
| **Total observe_multimodal p50 (CPU)** | **~2s** | | V-JEPA is bottleneck |
| **Total observe_multimodal p50 (GPU)** | | **~500ms** | |

These are end-to-end including layer 3 storage. Acceptable for agent workloads where multimodal observations happen seconds-to-minutes apart, not per-frame.

## Why this stack and not alternatives

| Alternative | Why not |
|---|---|
| Whisper for audio | TRIBE v2 uses Wav2Vec-BERT; using whisper breaks alignment for BAMS |
| Llama 3.1 8B / Llama 4 for text | overkill for feature extraction; doesn't match TRIBE encoder; ~3-10× slower |
| CLIP for video | image-only, not video; no temporal modeling |
| MMS (Massively Multilingual Speech) | not what TRIBE used; would force re-running BAMS calibration |
| ImageBind (multimodal joint embedding) | not factorable; loses VSA binding's compositional advantage |
| Learned quantization (small MLP from latent to 8192 bits) | adds training dependency; locked out by constitution article XVIII clause 5 in v2.1 |
| Thermometer / one-hot coding | poor for high-dim semantic embeddings; loses information |
| GPT-4o / Claude for triple extraction | not local; API key required; constitution article IV (no LLM in write OR read for extraction; LLM only for revision/consolidation) |

The chosen stack (GLiNER + V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B + Charikar 2002) is what enables agidb to be (a) fully local, (b) TRIBE-aligned for BAMS, (c) constitution-compliant.

## What this layer doesn't do

- **Store anything.** Layer 3's job.
- **Retrieve anything.** Layer 1's job.
- **Decide what to consolidate.** Consolidation worker.
- **Run any LLM.** Layer 2 uses frozen feature extractors; for belief extraction, pattern matching not LLM.
- **Manage encoder downloads.** Manifest specifies the SHA; `agidb-cli setup-encoders` handles downloads.

## Dependency graph

```
GLiNER ONNX (phase 3)
   ↓
text observe() unlocks tier B + alias resolution + belief extraction in phase 9

V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B (phase 14, v2.1)
   ↓
observe_multimodal() unlocks multimodal recall + brain-aligned surprise + BAMS

Charikar 2002 projection (phase 14, v2.1)
   ↓
multimodal HVs flow into layer 1's encode_multimodal_episode()
```

## Test coverage

| Test | What it verifies |
|---|---|
| GLiNER extraction property tests | F1 > 0.85 against 100-sample human-labelled gold set |
| Time anchor parsing | 50 test cases (yesterday/last week/ISO dates/etc) |
| Alias resolution | exact match wins, Levenshtein < 3 fuzzy match |
| Predicate canonicalization | 30 surface predicates → canonical |
| Belief extraction | F1 > 0.70 against 50-sample belief gold set |
| V-JEPA 2 wrapper (phase 14) | inference roundtrip on test video; output matches reference within 1e-3 |
| Wav2Vec-BERT wrapper (phase 14) | inference roundtrip on test audio |
| Llama text encoder wrapper (phase 14) | inference roundtrip on test text |
| HDC projection determinism | same input + seed → same output |
| HDC projection distance preservation | JL bound holds on 1000 random latent pairs |
| Encoder version mismatch | opening v2.1 db with wrong encoder → clear error |

Phase 3 covers text extraction tests. Phase 14 covers multimodal extraction tests.
