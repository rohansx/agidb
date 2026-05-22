# phase 16 — BAMS benchmark + ICLR paper

**duration:** weeks 47-52
**status:** not started
**depends on:** [phase 15](./phase-15-brain-calibrated-surprise.md)

> **v2.1 — constitutionally gated.** This phase closes the v2.1 track, which proceeds ONLY if the phase-7 decision was "Commit" AND v2.0 launched successfully.

## goal

ship the brain-aligned memory similarity (BAMS) benchmark suite, run it across all baselines, and write and submit the ICLR 2026 MemAgents workshop paper. BAMS scores agent memory systems by RSA-correlating their representational geometry against TRIBE-derived neural ground truth across six functional cortical networks — then v2.1 ships.

## deliverables

### week 47

- [ ] create the `agidb-bams` crate
- [ ] implement `agidb-bams::protocol` — the BAMS protocol (per `bams-benchmark.md`): stimulus loading, TRIBE v2 inference, per-network RDM construction, agent RDM construction, RSA scoring
- [ ] implement `agidb-bams::networks` — six functional cortical network definitions (DMN, visual, auditory, language, dorsal attention, frontoparietal), Schaefer-to-network mapping

### week 48

- [ ] build baseline adapters: `agidb-bams::baselines::{mem0, letta, zep, hipporag, raw_vjepa, random}`; each implements `AgentMemorySystem::replay_stimulus(stream) -> Vec<HV>`
- [ ] for text-only baselines (mem0/letta/zep), replay strategy: feed text descriptions of stimuli (captions/transcripts) since they don't support multimodal natively; document this as a methodological limitation in the paper
- [ ] random baseline: random 8192-bit HVs as the statistical null; should score ~0

### week 49

- [ ] run the full BAMS suite: 6 movies × 7 systems × 6 networks; estimated compute ~8h on a laptop with GPU, ~24h CPU-only; run on a cloud GPU for speed
- [ ] generate the report (`agidb-bams report results.json --format html`) — overall + per-network + per-movie tables
- [ ] ablations: agidb without VSA binding (concatenation), agidb with attention fusion instead of XOR, agidb without brain-calibrated surprise, agidb without consolidation

### week 50

- [ ] paper draft — title *Brain-Aligned Memory Retrieval: Measuring Cognitive Plausibility in Agent Memory Systems via TRIBE-Derived Ground Truth*; target ICLR 2026 MemAgents workshop (6-page version); sections per the `bams-benchmark.md` paper outline
- [ ] figures: overall BAMS scores table, per-network heatmap, ablation table, RDM visualizations (a few representative examples)
- [ ] internal review

### week 51

- [ ] address review feedback; revise the paper
- [ ] build a reproduction kit: a Docker container that runs the full BAMS suite end-to-end with one command; pin all dependency versions, dataset SHAs, model weight hashes
- [ ] open-source `agidb-bams` on `github.com/agidb/agidb-bams` under Apache-2.0 (benchmark code), with explicit notes about TRIBE v2 CC BY-NC for the weight artifacts

### week 52

- [ ] submit to the ICLR 2026 MemAgents workshop (if the deadline is missed, the backup is CCN 2026)
- [ ] crates.io: publish `agidb 0.2.0` (v2.1) + `agidb-sensory 0.1.0` + `agidb-bams 0.1.0`; PyPI: publish `agidb 0.2.0`
- [ ] launch blog post for v2.1 — demo: observe a video clip, recall it via cue, factor by modality, run a BAMS self-score
- [ ] **v2.1 SHIPS. Month 12 milestone reached.**

## exit criterion

BAMS suite open-source with reproducible baselines. ICLR 2026 MemAgents paper submitted. agidb 0.2.0 published. **Phase 16 complete. v2.1 LAUNCHED.**

## see also

- [../product/roadmap.md](../product/roadmap.md)
- [../spec/constitution.md](../spec/constitution.md)
- [../architecture/bams-benchmark.md](../architecture/bams-benchmark.md)
