# phase 14 — multimodal sensory

**duration:** weeks 37-42
**status:** not started
**depends on:** [phase 8](./phase-8-hardening-launch.md)

> **v2.1 — constitutionally gated.** This phase begins ONLY if the phase-7 decision was "Commit" AND v2.0 launched successfully (constitution article XVIII clause 2 + the article XIII extension).

## goal

stand up the v2.1 multimodal sensory stack: V-JEPA 2, Wav2Vec-BERT, and Llama-3.2-3B sensory encoders, Charikar 2002 thresholded random projection to 8192-bit HVs, and VSA role-filler binding of multimodal episodes. agidb can observe video + audio + text, project each into hyperdimensional space, bind them into a single episode signature, and factor a stored episode back into per-modality signatures.

## deliverables

### week 37

- [ ] create the `agidb-sensory` crate; add it to the workspace
- [ ] wire `ort` (ONNX runtime) for V-JEPA 2 inference; download V-JEPA 2 Gigantic-256 weights from HuggingFace (CC BY-NC); pin the SHA
- [ ] implement `agidb-sensory::vjepa::VJepa2Encoder` with `encode(video: &VideoClip) -> Result<[f32; 1024]>` — spatial mean pooling of the 8192-token output
- [ ] smoke test: encode a 64-frame video clip, verify output shape and reasonable values

### week 38

- [ ] wire Wav2Vec-BERT 2.0; download weights, pin SHA; implement `agidb-sensory::wav2vec::Wav2VecBertEncoder` with `encode(audio: &AudioClip) -> Result<[f32; 1024]>` — temporal mean pooling
- [ ] wire Llama-3.2-3B as a text encoder (forward pass only, not generation); implement `agidb-sensory::llama::LlamaTextEncoder` with `encode(text: &str) -> Result<[f32; 2048]>` — mean pooling of the layer-32 hidden state
- [ ] inference performance baseline on a laptop: measure CPU latency for each encoder

### week 39

- [ ] implement `agidb-sensory::project::HDCProjector` — Charikar 2002 thresholded random projection; per-encoder seeded matrices
- [ ] property tests: same input + same seed → same output (determinism); 1000 random latent pairs → hamming distance ordering preserves cosine distance ordering (Spearman correlation > 0.85)
- [ ] add the `MultimodalEncoder` trait; each encoder gets `encode_and_project(input) -> Result<HV>`

### week 40

- [ ] implement `agidb-sensory::multimodal::bind_multimodal_episode` — VSA role-filler binding: `episode = ROLE_VIDEO ⊕ sig_v XOR ROLE_AUDIO ⊕ sig_a XOR ROLE_TEXT ⊕ sig_t XOR ROLE_GOAL ⊕ sig_g XOR ROLE_TIME ⊕ sig_time`
- [ ] implement modality factorization: `extract_modality_signature(episode_sig, modality)` returns an approximate sig plus nearest-neighbor cleanup against a per-modality codebook
- [ ] property test: bind 3 modalities, extract each individually with cleanup, hamming distance to the original sig ≤ 200 bits (2.5% of 8192)

### week 41

- [ ] extend `Agidb::observe_multimodal(video, audio, text, ctx)` API; wire into layer 3 storage — append per-modality signatures to mmap, store offsets in a new `modality_signatures` column on episodes
- [ ] two new redb tables: `self_vector_history` (already added in phase 10, schema unchanged), `encoder_versions` (new)
- [ ] encoder version mismatch detection: open a db with encoders X while the binary uses encoders Y → error with a migration message
- [ ] extend `recall()` to factor multimodal episodes — per-modality similarity scoring when the query specifies a modality preference

### week 42

- [ ] end-to-end benchmark: 30s video + 30s audio + 100 tokens text → encoded → projected → bound → stored; P50 latency ≤ 2s CPU on a laptop
- [ ] optional Candle backend: pure-Rust ML inference path as an alternative to ONNX; identical outputs to within 1e-3
- [ ] MCP + Python expose `observe_multimodal`
- [ ] docs update: `layer-2-extraction.md`, `brain-alignment.md`, `layer-3-storage.md` reflect shipped behavior

## exit criterion

end-to-end multimodal observe pipeline works. P50 latency ≤ 2s on laptop CPU. Modality factorization works (extract recovers the original sig with < 200 bits noise). **Phase 14 complete.**

## see also

- [../product/roadmap.md](../product/roadmap.md)
- [../spec/constitution.md](../spec/constitution.md)
- [../architecture/layer-2-extraction.md](../architecture/layer-2-extraction.md)
- [../architecture/brain-alignment.md](../architecture/brain-alignment.md)
- [../architecture/layer-3-storage.md](../architecture/layer-3-storage.md)
