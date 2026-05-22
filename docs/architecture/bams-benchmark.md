# agidb — BAMS Benchmark (v2.1)

> The brain-aligned memory similarity benchmark. Protocol, baselines,
> implementation plan, paper plan. The first published evaluation of agent
> memory systems against TRIBE-derived cortical ground truth.

**Status:** v2.1 milestone, phase 16 (weeks 47-52, ~aug 2026). Gated on phase 14 (multimodal encoders) and phase 15 (brain-calibrated surprise) completing.

**Target venue:** ICLR 2026 MemAgents workshop. Backup: CCN 2026. Stretch: NeurIPS 2026 main.

## What BAMS is

**BAMS = Brain-Aligned Memory Similarity.**

A benchmark suite that scores agent memory systems by how well their internal representations align with predicted human cortical activations on matched naturalistic stimuli, measured via representational similarity analysis (RSA) across six functional cortical networks.

### What it measures

Given a stimulus stream (a movie clip with audio), at each TR (~1.5s window):
1. **TRIBE v2 predicts** cortical activation patterns across ~70k voxels for an average human watching that stimulus.
2. **The agent memory system under test** processes the same stimulus and produces an internal representation (an episode signature for agidb; a vector embedding for mem0/letta/zep; raw V-JEPA latents for the unprocessed-encoder baseline).
3. **RSA** compares the structural similarity of the two representation spaces over many TR pairs.

Score = mean Pearson correlation between the upper triangles of the TRIBE-derived representational dissimilarity matrix (RDM) and the agent's RDM, computed per functional cortical network and averaged.

### Why this is novel

Existing agent memory benchmarks fall into three categories:

| Category | Examples | Measures |
|---|---|---|
| Long-context QA | LongMemEval-S, BEAM | retrieval accuracy on synthetic long-context Q&A |
| Multi-session conversation | LoCoMo, PrefEval | memory consistency across sessions |
| Personalization | Mem0 internal, Hindsight | preference learning + recall |

**None measure cognitive plausibility.** Whether the memory's internal representations resemble how human memory organizes the same information has not been evaluated for any production agent memory system. BAMS fills this gap.

### Why now

Three converging conditions:

1. **TRIBE v2 made it tractable.** Before March 2026, you couldn't get well-validated cortical predictions on arbitrary naturalistic stimuli. TRIBE v2 changed that.
2. **RSA is the right comparison method.** Kriegeskorte et al. 2008 established RSA as the standard way to compare representations across systems (brains, models, behavior). The technique is well-understood and widely accepted.
3. **Agent memory is a category but the evaluations are converging on saturation.** LongMemEval and LoCoMo scores are crowding above 90%. The field needs a new axis. Cognitive plausibility is a defensible axis with empirical grounding.

## The protocol

### Input

- **Stimulus dataset:** 6 held-out naturalistic movies from Algonauts 2025 OOD set (Pulp Fiction, Princess Mononoke, Passe-Partout, World of Tomorrow, Planet Earth, Charlie Chaplin). Total ~6 hours. Public datasets accessible via Courtois NeuroMod / Algonauts pipeline.
- **TR resolution:** 1.49s (matches Courtois NeuroMod fMRI sampling).
- **Stimulus features:** video at 256×256 with 64-frame windows, audio at 16kHz with 60s windows, text (transcripts/captions where available).
- **Ground truth:** TRIBE v2 predicted BOLD across 1000 Schaefer parcels (v1 mode) or ~70k cortical surface vertices (v2 mode).

### Procedure

**Step 1 — Compute TRIBE-derived RDMs (offline, one-time).**

For each of 6 movies, for each TR t:
- Run TRIBE v2 over the (video, audio, text) stream at time t → predicted BOLD per parcel/voxel.
- For each of 6 functional cortical networks (DMN, visual, auditory, language, dorsal attention, frontoparietal), extract the predicted activation pattern over parcels assigned to that network.

For each network, compute the RDM:
```
RDM_brain[i][j] = 1 - pearson(activation_pattern[t_i], activation_pattern[t_j])
```

This gives 6 RDMs per movie, one per cortical network. Total over the suite: 36 RDMs.

**Step 2 — Compute agent memory RDMs (per system being evaluated).**

For each movie, replay the stimulus stream to the agent memory system. At each TR boundary, capture the agent's internal representation of "what has been observed so far." For agidb, this is the most recent episode signature (8192 bits). For mem0/letta/zep, this is the most recent stored embedding or the bundle of recent embeddings.

Compute the RDM:
```
RDM_agent[i][j] = distance(repr[t_i], repr[t_j])
```

Distance metric per system:
- agidb (binary HV): hamming distance / 8192
- raw V-JEPA / dense embeddings: cosine distance
- HippoRAG (graph): structural distance on retrieved subgraph

**Step 3 — RSA comparison.**

For each (movie, cortical network) pair:
```
RSA_score = pearson(upper_triangle(RDM_brain), upper_triangle(RDM_agent))
```

Higher = agent representations are more similar to predicted cortical representations.

**Step 4 — Aggregate.**

```
BAMS_score(system) = mean over (movies, networks) of RSA_score
BAMS_per_network(system, network) = mean over movies of RSA_score for that network
```

Both reported in publication. Per-network breakdown is more diagnostic than the aggregate.

### Reproducibility requirements

- Stimulus dataset must be accessible (Courtois NeuroMod is open).
- TRIBE v2 inference reproducible via published weights (CC BY-NC).
- Random seeds documented for the agent under test where applicable.
- Inference logs published.
- Docker container with full pipeline released alongside the paper.

## Baselines

BAMS evaluation must include these baselines for the paper to be credible:

### Tier A — Necessary baselines

| Baseline | Why | Expected score |
|---|---|---|
| **Raw V-JEPA 2 latents** | Establishes the encoder's own brain-alignment without memory machinery. Lower bound for "pure perception". | Mid range. V-JEPA 2 is part of TRIBE's encoder stack, so some alignment is expected; but raw latents aren't filtered/consolidated. |
| **Raw Wav2Vec-BERT latents** | Same for audio. | Mid for auditory network, low for others. |
| **Raw Llama-3.2-3B latents** | Same for text. | Mid for language network, low for others. |
| **Random representations** | Statistical null. Score should be ~0. | ~0 (sanity check). |
| **agidb v2.1** | The system under test. | TBD; hypothesis: wins associative-cortex networks (DMN, dorsal attention, frontoparietal) due to HDC binding's compositional structure. |

### Tier B — Competitor baselines

| Baseline | Architecture | What we test |
|---|---|---|
| **mem0** | LLM-extracted facts + vector DB | Does extractive memory align with cortex? Hypothesis: low for sensory networks, moderate for language. |
| **letta** | OS-inspired memory tiers + LLM-managed | Does agent-managed memory align? Hypothesis: similar to mem0. |
| **zep/graphiti** | Temporal knowledge graph | Does graph structure align? Hypothesis: low (graphs are structurally unlike cortex). |
| **hippoRAG** | PPR over LLM-extracted KG | Does hippocampally-inspired retrieval align? Hypothesis: moderate due to the explicit memory-systems framing. |
| **hippoMM** | Dentate gyrus + CA3 abstractions for audiovisual | Closest spirit-analog to agidb. Hypothesis: competitive. |

### Tier C — Ablation baselines (for the paper)

| Ablation | Tests |
|---|---|
| agidb without VSA binding (flat concatenation) | Whether role-filler binding matters |
| agidb with attention fusion instead of XOR | Whether factorability matters for alignment |
| agidb without brain-calibrated surprise (default 0.4 threshold) | Whether calibration matters |
| agidb with learned projection instead of random | Whether training the projection matters |
| agidb without consolidation | Whether sleep-like consolidation aligns with off-stimulus DMN |

The ablations are what make this a paper rather than a benchmark report.

## Implementation plan

### Crate: `agidb-bams`

New workspace crate. Pure Rust implementation. Calls out to TRIBE v2 via subprocess (Python) in v2.1; native Rust port deferred.

**Modules:**
- `protocol.rs` — the full BAMS protocol implementation
- `tribe.rs` — TRIBE v2 inference wrapper via PyO3 subprocess
- `rsa.rs` — representational similarity analysis (Kriegeskorte 2008)
- `networks.rs` — six functional cortical network definitions, Schaefer-to-network mapping
- `baselines/mem0.rs` — adapter to mem0 Python SDK
- `baselines/letta.rs` — adapter to Letta API
- `baselines/zep.rs` — adapter to Zep/Graphiti
- `baselines/hipporag.rs` — adapter to HippoRAG (Python via subprocess)
- `baselines/random.rs` — random representation baseline
- `cli.rs` — `agidb-bams` CLI for running the full suite

**CLI:**
```bash
agidb-bams run \
    --systems agidb,mem0,letta,zep,hipporag,raw-vjepa,random \
    --movies algonauts-2025-ood \
    --networks all \
    --output bams-results-2026-08.json

agidb-bams report bams-results-2026-08.json \
    --format html \
    --output bams-report.html
```

**Output schema:**
```json
{
  "version": "0.1.0",
  "tribe_version": "v2-march-2026",
  "agidb_version": "0.1.0-alpha",
  "timestamp": "2026-08-15T...",
  "results": {
    "agidb": {
      "overall_bams_score": 0.XX,
      "per_network": {
        "DMN": 0.XX,
        "visual": 0.XX,
        "auditory": 0.XX,
        "language": 0.XX,
        "dorsal_attention": 0.XX,
        "frontoparietal": 0.XX
      },
      "per_movie": { "pulp_fiction": {...}, ... }
    },
    "mem0": {...},
    ...
  },
  "reproduction": {
    "container_hash": "sha256:...",
    "seed": 42,
    "tribe_weights_hash": "..."
  }
}
```

### Dependencies for v2.1

- TRIBE v2 weights (CC BY-NC, research use; benchmark code Apache-2.0 with note)
- Courtois NeuroMod dataset access (open access, requires acknowledgment)
- Algonauts 2025 OOD stimulus files (open access via algonauts.org)
- PyO3 + Python 3.11 + TRIBE inference deps (torch, transformers, etc.) for the v2.1 ship
- Adapter packages for each baseline (mem0, letta-client, zep-python, hipporag)

### Performance targets

- Single-movie evaluation (all 6 networks): ≤ 30s on a laptop with GPU; ≤ 5min CPU-only
- Full suite (6 movies × 7 systems × 6 networks): ≤ 8 hours on a single machine; parallelizable across movies
- Single-movie RDM compute (one system): ≤ 5s

## The paper

### Title

*Brain-Aligned Memory Retrieval: Measuring Cognitive Plausibility in Agent Memory Systems via TRIBE-Derived Ground Truth*

### Authors (proposed)

Rohan [Lastname], Independent. Coauthors TBD as collaborations form.

### Venue priority

1. **ICLR 2026 MemAgents workshop** (target). Reasons: explicit scope match, deadline alignment with month 12 ship, light review cycle, established community for agent memory.
2. **CCN 2026** (Cognitive Computational Neuroscience). Backup if MemAgents misses deadline. Reasons: explicit brain+model interface community, oral presentation prestigious, but harder to slip a substrate-engineering paper through CCN reviewers expecting pure neuroscience.
3. **MLSys 2027.** Backup. Reasons: systems-paper-friendly, would emphasize the substrate engineering side. Timeline slips to 2027.
4. **NeurIPS 2026 main.** Stretch goal. Hard to land an agent-memory-systems paper here, but BAMS as a benchmark contribution could fit if framed right.

### Abstract (target ~250 words)

> Agent memory systems are typically evaluated on downstream QA benchmarks (LongMemEval, LoCoMo, BEAM, PrefEval) that score retrieval accuracy without reference to how human memory organizes the same information. We propose BAMS, a brain-aligned memory similarity benchmark scoring agent memory representations against ground-truth cortical activation patterns predicted by TRIBE v2 (Meta FAIR 2026), a foundation model predicting fMRI BOLD across 720 subjects watching naturalistic movies. Given a held-out audiovisual stimulus, we compare an agent's internal memory representation trajectory against TRIBE v2's predicted activation across six functional cortical networks (default mode, visual, auditory, language, dorsal attention, frontoparietal) using representational similarity analysis.
>
> We apply BAMS to (a) raw V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B latents, (b) agidb, an open-source rust-native HDC cognitive substrate projecting multimodal latents into 8192-bit binary signatures with VSA role-filler binding, (c) four existing agent-memory systems (mem0, letta, zep/graphiti, hippoRAG). We show: (i) HDC-binding-based memory representations are significantly more brain-aligned in associative cortex (DMN, dorsal attention, frontoparietal) than dense embedding retrieval, (ii) modality dropout during projection training mirroring TRIBE's recipe improves alignment by X%, (iii) surprise-gated admission with thresholds calibrated to TRIBE-predicted neural surprise yields agent memory retaining the high-saliency moments human cortex retains.
>
> We release agidb, BAMS, and a docker reproduction kit. Brain-alignment becomes a complementary evaluation axis for the agent memory community.

### Structure (6 pages workshop version)

1. **Introduction** (1 page) — agent memory category, evaluation gap, brain-alignment as a new axis, contributions.
2. **Background** (1 page) — TRIBE v2 architecture, JEPA family, agent memory landscape (mem0/letta/zep), RSA methodology, HDC/VSA primer.
3. **BAMS protocol** (1.5 pages) — stimulus dataset, TRIBE inference, per-network RDM construction, agent RDM construction, RSA aggregation, reproducibility.
4. **agidb-specific contribution** (1 page) — multimodal HDC pipeline, VSA role-filler binding, brain-calibrated surprise gating, the three claims (i)-(iii) above.
5. **Results** (1 page) — overall BAMS scores table, per-network breakdown, ablations, qualitative analysis.
6. **Discussion + limitations** (0.5 page) — what BAMS doesn't measure, TRIBE's noise ceiling, future directions.

Full version (NeurIPS-style, 9 pages) adds: extended results, full ablation table, additional baselines, longer related work.

### What we don't claim in the paper

- We don't claim agidb "thinks like a brain."
- We don't claim BAMS replaces existing benchmarks. It's complementary.
- We don't claim brain-alignment correlates with downstream agent task performance unconditionally. That's a research question for a separate paper.
- We don't claim TRIBE v2's predictions are perfect cortical ground truth. They are the best currently available, bounded by their own noise ceiling (~54% of explainable variance).

## What success at v2.1 looks like (BAMS-specific)

- BAMS suite open-source on github.com/agidb/agidb-bams under Apache-2.0 (benchmark code) + research note (TRIBE v2 CC BY-NC for weights).
- Docker container reproduces published numbers within 1% of reported values.
- agidb wins BAMS in at least 3 of 6 functional networks (target: DMN, dorsal attention, frontoparietal — the associative-cortex networks where HDC binding's compositional structure should help).
- Paper submitted to ICLR 2026 MemAgents workshop by deadline.
- Paper accepted (workshop acceptance rate typically 50-60%).
- Cited by at least one other agent-memory paper within 6 months of acceptance.

## What failure modes look like

- **BAMS shows no meaningful difference between systems.** Possible if RSA scores all cluster around the noise floor; would suggest BAMS isn't discriminative. Mitigation: add finer-grained per-stimulus-class analysis, include more diverse ablations.
- **agidb loses to raw V-JEPA latents.** Possible if HDC projection loses too much information vs the dense baseline. Indicates either projection bottleneck (revisit Charikar 2002 → learned quantization in v2.2) or the substrate adds noise without compositional benefit.
- **MemAgents workshop deadline missed.** Backup CCN 2026 has a later deadline. Worst case: defer to NeurIPS 2026 main track.
- **TRIBE v2 doesn't generalize as well as the v1 paper suggests.** Mitigation: report results bounded by TRIBE's own pearson scores; acknowledge the ceiling.

## Open questions for future BAMS iterations

1. **Can BAMS be extended to non-naturalistic stimuli?** Requires fMRI ground truth for the target stimulus class. Currently movies + podcasts + silent videos.
2. **Can BAMS be evaluated against MEG/EEG instead of fMRI?** Faster temporal resolution but lower spatial. Different ground-truth models needed (Brain-JEPA, signal-JEPA, Laya).
3. **Can a leaderboard format work?** Probably yes; the BAMS protocol is deterministic enough. Risk: gaming the benchmark (overfit to TRIBE's predictions rather than to actual cognitive structure).
4. **Should BAMS scores be released per cortical parcel rather than per network?** Higher resolution but more variance. v2.2+ decision.
5. **Can BAMS evaluate working memory and goal-directed retrieval, not just episodic encoding?** Yes, requires goal-conditioned stimulus design. v2.2+ extension.

## Why this matters in one line

BAMS gives agidb the only published evaluation that none of mem0, letta, zep, or cognee can run on themselves without rebuilding their architecture, because they don't use the same encoder stack as TRIBE v2 and don't have the factorable representation that lets per-modality alignment be measured. Brain-alignment is agidb's structural moat.
