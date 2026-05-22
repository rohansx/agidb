# phase 15 — brain-calibrated surprise

**duration:** weeks 43-46
**status:** not started
**depends on:** [phase 14](./phase-14-multimodal-sensory.md)

> **v2.1 — constitutionally gated.** This phase is part of the v2.1 track, which proceeds ONLY if the phase-7 decision was "Commit" AND v2.0 launched successfully.

## goal

empirically fit the surprise threshold θ_brain against TRIBE v2 predicted neural surprise. run TRIBE v2 over a movie stimulus to derive predicted BOLD per parcel, run agidb's multimodal observe pipeline over the same stimulus, and fit θ_brain so agidb's surprise gating tracks associative-cortex neural surprise — then validate on a held-out movie and ship the calibrated threshold as the v2.1 default.

## deliverables

### week 43

- [ ] download TRIBE v2 weights from `huggingface.co/facebook/tribev2` (CC BY-NC; research use); pin the SHA
- [ ] build a TRIBE v2 inference wrapper — v2.1 uses a PyO3 subprocess call to a Python script running TRIBE v2 (TRIBE's reference inference is Python; a pure-Rust port is deferred to v2.2+)
- [ ] verify TRIBE v2 inference matches published numbers on a sample stimulus (within Pearson r±0.005 of the paper's reported value on a single subject single movie)

### week 44

- [ ] acquire Courtois NeuroMod dataset access (open access; requires acknowledgment + email registration)
- [ ] acquire Algonauts 2025 OOD stimulus files (open access via algonauts.org)
- [ ] pick a representative subject (e.g. Courtois NeuroMod subject 1) and a held-out movie segment (e.g. Pulp Fiction first 20 minutes)
- [ ] run TRIBE v2 over the stimulus → predicted BOLD per parcel per TR

### week 45

- [ ] compute neural surprise: at each TR, `neural_surprise(t) = || BOLD_pred(t) - sliding_mean(BOLD_pred, ±5 TRs) ||` over associative-cortex parcels (TPJ, dlPFC, DMN regions in the Schaefer 1000 atlas)
- [ ] run agidb's `observe_multimodal` pipeline over the same stimulus → signature stream
- [ ] compute agidb surprise: at each TR, `agidb_surprise(t) = 1 - hamming_sim(sig(t), bundle(sigs[t-K..t]))`
- [ ] fit threshold θ_brain to maximize Pearson correlation between `Indicator(agidb_surprise > θ_brain)` and `Indicator(neural_surprise > σ × mean_neural_surprise)` for σ ∈ {1.5, 2.0, 2.5}

### week 46

- [ ] validate the calibration on a held-out movie (Princess Mononoke or World of Tomorrow); the calibrated threshold should generalize within ±10% of the fitted value
- [ ] publish the calibrated θ_brain as the default surprise threshold for new v2.1 databases; store it in `manifest.toml` with provenance (calibration dataset SHA, TRIBE v2 version, fit date)
- [ ] documentation: the `brain-alignment.md` calibration section includes the full reproducible recipe
- [ ] add `Agidb::brain_calibration()` and `Agidb::recalibrate(dataset)` APIs
- [ ] comparison plot: pre-calibration (θ=0.4) vs post-calibration (θ_brain) sensory promotion patterns on a held-out movie — visually demonstrate the difference

## exit criterion

calibrated θ_brain ships in v2.1. Reproducible calibration recipe documented. **Phase 15 complete.**

## see also

- [../product/roadmap.md](../product/roadmap.md)
- [../spec/constitution.md](../spec/constitution.md)
- [../architecture/brain-alignment.md](../architecture/brain-alignment.md)
