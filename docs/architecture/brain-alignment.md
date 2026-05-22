# agidb — Brain Alignment (v2.1+)

> The full technical detail of how agidb v2.1 integrates Meta FAIR's
> V-JEPA 2, Wav2Vec-BERT, and Llama-3.2-3B sensory encoders, projects
> their latents to 8192-bit HDC signatures, binds them into multimodal
> episodes via VSA, and calibrates surprise gating against TRIBE v2
> brain-encoding ground truth.

**Status:** v2.1 milestone, target month 12 (aug 2026). Gated on v2.0 decision gate "Commit" outcome at week 12.

## What brain-alignment is and isn't

**What it is:**
- An empirical methodology for evaluating agent memory representations against human cortical activation patterns predicted by TRIBE v2 across 720 subjects on naturalistic movies.
- A measurement-grounded calibration of agidb's sensory surprise threshold.
- A multimodal sensory pipeline using the same encoder stack as TRIBE v2 (V-JEPA 2 video, Wav2Vec-BERT audio, Llama-3.2-3B text) so the comparison is meaningful.
- Constitution article XVIII.

**What it isn't:**
- A claim that agidb "thinks like a brain" (it doesn't).
- A brain-decoding service (we don't decode user brains).
- A replacement for the cognitive primitives (goals/beliefs/self-model still ship in v2.0 first).
- A change to the core HDC substrate (still 8192-bit BSC, still bind/bundle/hamming).

## Why this matters

Three reasons.

**1. agidb gains a unique evaluation axis.** Existing agent memory benchmarks (LongMemEval, LoCoMo, BEAM, PrefEval) measure downstream QA accuracy. None measure whether the memory's internal representations resemble human memory. TRIBE v2 (Meta FAIR, March 2026) made brain-aligned evaluation tractable for the first time by releasing open weights for a foundation model predicting fMRI BOLD across 720 subjects from V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B. agidb v2.1 inherits this evaluation surface.

**2. Surprise threshold gets a defensible value.** v2.0's surprise threshold is a magic number (default 0.4). v2.1 calibrates it against neural surprise predicted by TRIBE v2 on associative cortex. This is publishable methodology, not a guess.

**3. Multimodal episodes via VSA are factorable in a way attention fusion is not.** TRIBE v2 fuses modalities via attention into a dense hidden state — once fused, the components are not separately recoverable. agidb fuses via VSA role-filler binding (XOR) — any modality can be recovered from a stored episode signature by XORing with the appropriate role hypervector. This is the structural advantage over both TRIBE (attention) and mem0/letta/zep (dense embeddings).

## TRIBE v2 — what to know

TRIBE v2 was released by Meta FAIR on March 26, 2026. Paper: arxiv 2507.22229 (v1) and the v2 technical report. Weights: huggingface.co/facebook/tribev2 under CC BY-NC. Code: github.com/facebookresearch/tribev2.

**What it is:** a tri-modal foundation model predicting fMRI BOLD responses to naturalistic stimuli. Won Algonauts 2025 (first place out of 263 teams) as TRIBE v1. v2 scales to ~70k voxel-level predictions across 720 subjects.

**Architecture (v1, preserved in v2):**
- Three frozen modality encoders producing per-time-step features resampled to a common 2 Hz grid:
  - Text: **Llama-3.2-3B**, 1024 preceding words context, 2048-d output
  - Audio: **Wav2Vec-BERT 2.0**, 60s chunks resampled 50→2 Hz, 1024-d output
  - Video: **V-JEPA 2 Gigantic-256**, 64 frames over preceding 4s per 2Hz bin, 1280-d output (spatially averaged)
- Each modality projected linear+layernorm to shared dim 1024, concatenated → 3×1024 per timestep
- **Temporal transformer: 8 layers, 8 attention heads, hidden 3072.** Context window 100 TRs (~149s) with 10s jitter
- Per-subject personalization: (a) learnable subject embedding added to input, (b) subject-specific linear head at output
- **Modality dropout p=0.2** during training (randomly zeroes one modality)
- **Ensemble of 1000 models** with varied seeds, losses, layer aggregations
- **Per-parcel softmax** over validation pearson with T=0.3 picks ensemble weights

**Training data:**
- 451.6 hours of fMRI training from 25 subjects (movies, podcasts, silent videos)
- Evaluated on 1117.7 hours across 720 subjects (including HCP 7T)
- Schaefer 1000-parcel atlas (v1), ~70k cortical surface vertices (v2)

**v1 results on Algonauts 2025 OOD:**
- Mean OOD pearson r = 0.2146 (recovers ~54% of noise ceiling)
- Beat the next two teams (VIBE 0.2096, SDA 0.2094) by tight margins, decided by ensembling sophistication

**Critical assessment:**
- TRIBE predicts BOLD (a slow hemodynamic proxy lagged ~5s behind neural activity), not neural firing or cognition itself.
- Noise ceiling caps achievable correlation around r ≈ 0.4 on naturalistic movies.
- The "70× resolution" headline involves a tradeoff between resolution and per-target noise.
- "AlphaFold for neuroscience" is influencer framing, not Meta's official claim. Meta uses "in-silico neuroscience" / "digital twin of neural activity." TRIBE is more accurately "BERT for fMRI" — a real foundation model, not a paradigm shift.

**Why agidb uses it:** TRIBE v2 is the best available source of cortical ground truth on naturalistic stimuli. Using it as a benchmark target is well-founded; pretending its predictions are "the brain" is not. We use it for evaluation, not as a feature.

## The encoder stack

agidb v2.1 uses the same three frozen encoders as TRIBE v2, for exactly the alignment-by-shared-representation reason.

### V-JEPA 2 — video encoder

- **Repo:** github.com/facebookresearch/vjepa2
- **Paper:** arxiv 2506.09985
- **Weights:** huggingface.co/facebook/vjepa2-gigantic-256
- **License:** CC BY-NC
- **Size:** 1.2B parameters
- **Input:** 64 frames at 256×256, 2-frame tubelets
- **Output:** 8192 patch tokens × 1024-d embeddings per clip (already 8192-token natively!)
- **Backbone:** ViT with 3D rotary position embeddings (3D-RoPE)
- **Training:** self-supervised on 1M+ hours of internet video. EMA target network prevents collapse.
- **Benchmarks:** SSv2 77.3% top-1, Epic-Kitchens-100 39.7 R@5

**For agidb:**
- Take the 64-frame, 256×256 video window. Run V-JEPA 2 encoder. Get 8192 × 1024 tokens.
- Spatially average to a single 1024-d vector per clip (matches TRIBE's pooling).
- Project to 8192-bit HDC signature via Charikar 2002.

**Inference cost:**
- CPU (Apple M2, i7-12700H): ~1.5s per 64-frame clip
- GPU (M2 ANE, RTX 4090): ~200ms per clip

### Wav2Vec-BERT 2.0 — audio encoder

- **Paper:** Meta SSL audio 2024
- **Weights:** huggingface.co/facebook/w2v-bert-2.0
- **License:** CC BY-NC
- **Input:** 60s audio chunk at 16kHz
- **Output:** ~50 Hz frame-level latents at 1024-d
- **Training:** self-supervised on multilingual audio

**For agidb:**
- Take 60s audio window. Run W2V-BERT encoder. Get frame-level 1024-d latents.
- Temporally mean-pool to a single 1024-d vector per clip (matches TRIBE).
- Project to 8192-bit HDC signature.

**Inference cost:**
- CPU: ~400ms per 60s clip
- GPU: ~80ms per clip

### Llama-3.2-3B — text encoder

- **Weights:** huggingface.co/meta-llama/Llama-3.2-3B
- **License:** Llama 3.2 community license (commercial use OK with attribution)
- **Input:** up to 1024 tokens preceding context
- **Output:** layer-32 hidden state at 3072-d (last token); for compact storage use the final-layer mean-pooled hidden state at 2048-d after dimension reduction

**For agidb:**
- Take text window. Tokenize. Run Llama-3.2-3B (encoder usage = forward pass, no generation).
- Extract layer-32 mean-pooled hidden state.
- Project to 8192-bit HDC signature.

**Inference cost:**
- CPU: ~200ms per 1024-token window
- GPU: ~30ms per window

**Why Llama-3.2-3B and not something larger:** TRIBE v2 uses Llama-3.2-3B. Matching means alignment. Larger models (8B, 70B) would be wasteful for feature extraction and break the comparison.

## HDC projection — Charikar 2002

Each encoder produces a dense latent. agidb projects to 8192-bit signatures via thresholded random projection:

```rust
pub struct HDCProjector {
    matrix: [[i8; D_INPUT]; 8192],  // ±1 entries, seeded
    bias: [i32; 8192],              // optional, often zero
}

impl HDCProjector {
    pub fn project(&self, x: &[f32; D_INPUT]) -> HV {
        let mut sig = HV::zero();
        for bit_idx in 0..8192 {
            let mut acc: i32 = 0;
            for d in 0..D_INPUT {
                acc += (self.matrix[bit_idx][d] as i32) * (x[d] * SCALE) as i32;
            }
            if acc > self.bias[bit_idx] {
                sig.set_bit(bit_idx);
            }
        }
        sig
    }
}
```

**Why this works:**
- **Johnson-Lindenstrauss guarantee.** For a random projection matrix R ∈ {-1,+1}^(k × d), cosine distance in the original space is approximately preserved in hamming distance over `sign(Rx)`. Charikar 2002 "Similarity Estimation Techniques from Rounding Algorithms" proved this for the sign-projection case. JL bound: ε-distortion for k = O(log n / ε²), so 8192 bits is more than enough for our scales.
- **Deterministic.** Fixed seed → reproducible. Same input → same signature.
- **Training-free.** No learned parameters. Survives encoder version changes (just regenerate projection matrix).
- **Fast.** Multiply-add of 1024 or 2048 entries per bit. SIMD-friendly.

**Why not alternatives:**
- **Learned quantization** (small MLP, sign-quantize output): could optimize for downstream tasks but adds a training dependency. Locked out by article XVIII clause 5 in v2.1; revisit in v2.2 only if BAMS plateaus.
- **Thermometer coding** (per-dim ordinal binning): less expressive for high-dim semantic embeddings. Use only for scalar sensor channels in v2.3.
- **Sparse Binary Distributed Representation (SBDR, Kanerva sparse codes ~2% density):** matches biological sparsity, large capacity advantage for associative memory. Invasive to migrate from BSC. Consider for v2.5 substrate evolution.

**Projection matrix versioning:**
- Each encoder gets a deterministic seeded projection matrix.
- Matrix seeds stored in `manifest.toml`.
- Encoder version + projection seed = reproducibility.
- Encoder upgrade requires re-projection of old episodes (deferred, optional).

## VSA multimodal binding

Multimodal episodes are bound via XOR role-filler binding into a single 8192-bit episode signature:

```rust
pub fn bind_multimodal_episode(
    sig_video: Option<HV>,
    sig_audio: Option<HV>,
    sig_text: Option<HV>,
    goal_id: Option<GoalId>,
    belief_ids: &[BeliefId],
    time_bucket: TimeBucket,
) -> HV {
    let mut episode = HV::zero();

    if let Some(sv) = sig_video {
        episode ^= ROLE_VIDEO.bind(&sv);
    }
    if let Some(sa) = sig_audio {
        episode ^= ROLE_AUDIO.bind(&sa);
    }
    if let Some(st) = sig_text {
        episode ^= ROLE_TEXT.bind(&st);
    }
    if let Some(g) = goal_id {
        episode ^= ROLE_GOAL.bind(&goal_signature(g));
    }
    for b in belief_ids {
        episode ^= ROLE_BELIEF.bind(&belief_signature(*b));
    }
    episode ^= ROLE_TIME.bind(&time_signature(time_bucket));

    episode
}
```

`ROLE_*` are fixed random 8192-bit hypervectors seeded at workspace init.

**Factorability — the key property:**
```rust
pub fn extract_audio_signature(episode: &HV) -> HV {
    episode.bind(&ROLE_AUDIO)  // XOR with role HV → recovers approximately sig_audio
}
```

The recovered signature is an approximation (noise from bundling other modalities), cleaned up by nearest-neighbor search against the audio-signature codebook. This is the standard VSA unbind-and-cleanup pattern.

**Why factorability matters:**
- TRIBE v2 fuses via attention into a dense hidden state. You cannot recover the original audio from the fused state — the fusion is lossy and entangled.
- agidb fuses via XOR. Audio is recoverable.
- This enables: querying "show me episodes where the audio sounded like X" by binding ROLE_AUDIO with query audio and finding nearest stored episodes that produce a clean audio signature when unbound.
- Also enables: ablation studies, debugging, attribution. You can ask "what was the audio component of this episode's signature?" and answer it.

## Brain-calibrated surprise gating

v2.0's surprise threshold is a magic number. v2.1's is empirically fit.

### The calibration protocol

```
1. SELECT a paired stimulus dataset (movie clips with available TRIBE-aligned fMRI ground truth)
2. For each clip at each TR (1.49s window):
   a. Compute TRIBE v2 predicted BOLD across associative cortex parcels
      (TPJ, dlPFC, DMN regions in Schaefer 1000 atlas)
   b. Compute neural_surprise(t) = || BOLD_pred(t) - sliding_mean(BOLD_pred, ±5 TRs) ||
   c. Compute agidb signature for same clip via observe_multimodal pipeline
   d. Compute agidb_surprise(t) = 1 - hamming_sim(sig(t), bundle(sigs[t-K..t]))
3. FIT threshold θ_brain to maximize Pearson correlation between:
   - Indicator(agidb_surprise(t) > θ_brain)
   - Indicator(neural_surprise(t) > σ × mean_neural_surprise)
   where σ ∈ {1.5, 2.0, 2.5} is the neural threshold sweep
4. PUBLISH calibrated θ_brain with reproduction kit
```

### Where the calibration data comes from

- **Courtois NeuroMod:** 6 subjects, ~80h each of naturalistic movies (Friends seasons 1-7, four feature films). Open access. The training data for TRIBE.
- **Algonauts 2025 held-out movies:** 6 OOD films (Pulp Fiction, Princess Mononoke, Passe-Partout, World of Tomorrow, Planet Earth, Charlie Chaplin). TRIBE v2 has predicted BOLD here.
- **HCP 7T:** higher-resolution but smaller naturalistic stimulus set.

For v2.1 ship: calibrate on a single representative subject from Courtois NeuroMod, validate on Algonauts OOD held-outs. Document the protocol so users can recalibrate against their own ground truth.

### The expected outcome

θ_brain ≈ 0.45-0.55, slightly higher than v2.0's default 0.4. This makes sensory promotion more selective — closer to how human cortex actually filters input. Should empirically increase BAMS score because the resulting episodes will be more concentrated on high-saliency moments that match human attentional patterns.

### What we don't claim

- We don't claim agidb's surprise threshold "matches the human brain." We claim it correlates with neural surprise predicted by TRIBE v2 on associative cortex.
- We don't claim brain-calibrated surprise will improve downstream agent task performance unconditionally. We claim it's a measurement-grounded default that's defensible in papers and reproducible.
- The calibration is bounded by TRIBE v2's own noise ceiling (~54% of explainable variance). agidb-derived surprise can't be more brain-aligned than TRIBE's predictions are themselves accurate.

## Implementation plan

### Phase 14 — Multimodal sensory encoders (weeks 37-42)

**Goal:** end-to-end pipeline from raw video+audio+text to 8192-bit episode HV.

**Deliverables:**
1. `agidb-sensory::vjepa.rs` — V-JEPA 2 ONNX runtime wrapper, 64-frame video → 1024d
2. `agidb-sensory::wav2vec.rs` — Wav2Vec-BERT 2.0 wrapper, 60s audio → 1024d
3. `agidb-sensory::llama.rs` — Llama-3.2-3B wrapper, 1024-token text → 2048d
4. `agidb-sensory::project.rs` — Charikar 2002 thresholded random projection
5. `agidb-sensory::multimodal.rs` — VSA role-filler binding + unbinding API
6. `AgiDb::observe_multimodal()` API extension to `agidb-core`
7. ONNX backend by default; Candle backend as optional pure-Rust path
8. Property tests: project-then-unproject preserves distance ordering; bind-then-unbind recovers signatures with low hamming noise

**Exit criterion:** 30s video+audio clip → encoder inference → projection → binding → stored episode HV. P50 latency ≤ 2s on a laptop CPU.

### Phase 15 — Brain-calibrated surprise gating (weeks 43-46)

**Goal:** empirically calibrate θ_brain against TRIBE v2 predicted neural surprise.

**Deliverables:**
1. TRIBE v2 inference wrapper (Python subprocess via PyO3 for simplicity in v2.1; native Rust port later)
2. Calibration protocol implementation in `agidb-sensory::calibrate.rs`
3. Calibration script + dataset documentation (Courtois NeuroMod open access)
4. `manifest.toml` entry for calibrated θ_brain with provenance (calibration dataset, TRIBE v2 version, fit date)
5. Comparison plot: pre-calibration vs post-calibration sensory promotion patterns on a held-out movie

**Exit criterion:** calibrated θ_brain ships in v2.1. Documentation includes reproducible recipe. Calibration runs in CI nightly against fixed reference.

### Phase 16 — BAMS benchmark suite (weeks 47-52)

**Goal:** ship the brain-aligned memory similarity benchmark, baselines, and ICLR 2026 paper. See [bams-benchmark.md](./bams-benchmark.md) for the full protocol.

## Open questions for v2.2+

1. **Can a learned projection beat random projection on BAMS?** Lock article XVIII says no in v2.1; revisit if BAMS plateaus.
2. **Should the encoder stack evolve to V-JEPA 3 / TRIBE v3 when those land?** Likely yes, but recalibration cost is non-trivial.
3. **Can BCI input (Brain-JEPA, signal-JEPA) work as another sensory modality?** Speculative. v2.4 territory.
4. **Should agidb ship its own brain-encoder?** No. Out of scope per article XII. Use TRIBE v2 as published.
5. **Can BAMS be extended to non-naturalistic stimuli?** Yes, but requires fMRI ground truth for the target stimulus class. Currently movies + podcasts + silent videos cover most generic content.

## Operational notes

**GPU is helpful but not required.** v2.1 ships CPU-first. V-JEPA 2 on CPU takes ~1.5s per 64-frame clip; acceptable for most agent workloads where multimodal observations happen seconds-to-minutes apart, not per-frame.

**Encoder weights are downloaded on first use, not bundled.** Manifest pins the HuggingFace SHA. Ensures binary stays small (~100MB without weights, ~4GB with).

**Encoder versions are pinned per database.** A database created with V-JEPA 2 Gigantic-256 weights at hash X cannot be opened by a binary using hash Y unless re-projection is run. Documented in the migration guide.

**Brain-calibrated surprise is one-shot per database.** Set at database creation time from the global calibrated default. Users can recalibrate against their own fMRI data if they have any; documented but not required.

**ONNX vs Candle backend.** ONNX is the default (broadest hardware support). Candle is the experimental pure-Rust path for environments where ONNX runtime is unavailable (some embedded targets, WASM). Identical outputs to within numerical noise.

## What this gets us, in one paragraph

agidb v2.1 is the first agent memory substrate to ship with brain-aligned multimodal sensory encoding using the same encoder stack as Meta FAIR's TRIBE v2 brain-encoding foundation model, with surprise gating calibrated against 720-subject fMRI ground truth, and a published benchmark (BAMS) measuring representational similarity to predicted human cortical activations across six functional networks. None of the funded agent-memory competitors (mem0, letta, zep, cognee, supermemory) have published anything comparable. This is the paper-sized contribution that turns agidb from "another rust memory library" into "an artifact of brain-aligned cognitive science research with production rust deployment."
