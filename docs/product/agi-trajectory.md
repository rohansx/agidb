# agidb — AGI Trajectory

> The 5-year roadmap from agidb v2.0 (substrate, 2026) to v2.5 (AGI-grade,
> 2031). Brain-alignment integrated as the v2.1 additive milestone.

## The shape of the bet

agidb is a 5-year commitment, not a 9-month launch. The cognitive-substrate framing is what justifies the AGIDB name; the 12-month v2.1 brain-aligned launch is what justifies the first round of funding; the 5-year trajectory is what justifies the long-term existence of the company.

Each major version adds a capability frontier that compounds on the previous:

| Version | Year | What it adds | Decision gate |
|---|---|---|---|
| **v2.0** | 2026 (m9) | Substrate — episodic, semantic, procedural, working, sensory, goals, beliefs, self-model, unlearn, neurosymbolic interface | Phase 7, week 12: commit / reposition / retreat |
| **v2.1** | 2026 (m12) | Brain-alignment — V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B multimodal sensory, brain-calibrated surprise, BAMS benchmark, ICLR 2026 paper | Phase 16, week 52: paper accepted? BAMS wins associative networks? |
| **v2.2** | 2027 | Cognitive engine v0.1 — Hopfield pattern completion, AGM belief revision, analogical retrieval via HDC binding, learned projection (if BAMS plateaus) | End of 2027: design partner production deployments |
| **v2.3** | 2028 | Causal layer — causal claim storage with intervention semantics, world model fragments, on-line learning, Causal-JEPA-style object-centric masking | End of 2028: enterprise deal pipeline |
| **v2.4** | 2029-2030 | Production-grade — full enterprise tier, distributed mode, formal safety guarantees on self-modification, BCI input experimental (Brain-JEPA, signal-JEPA) | Mid-2030: revenue >$5M ARR |
| **v2.5** | 2031 | AGI-grade — substrate for true autonomous systems; closed-loop self-modification, causal reasoning over learned beliefs, cognitive engine fully realized | Year 5: agidb is the de facto AGI substrate or it isn't |

## v2.0 — Substrate (2026, month 9)

The first credible AGI substrate. Inherits sochdb v1's working HDC kernel, bi-temporal storage, episode binding, tiered recall, and consolidation. Adds five new phases (9-13) for the AGI pivot.

### What ships

- All seven cognitive floors with first-class typed shapes
- Goals as state machines with parent-child hierarchy
- Beliefs as revisable claims with audit trails
- Sensory buffer with surprise gating (hand-tuned threshold, no brain calibration yet)
- Self-model audit log + self-vector EMA
- Non-destructive cascading unlearn with self-vector subtraction
- Neurosymbolic interface (signature ↔ triple translation)
- 9 crates: agidb-core, agidb-extract, agidb-ns, agidb-skills, agidb-cli, agidb-mcp, agidb-py, agidb-bench, agidb umbrella

### Decision gate

Phase 7, week 12. The benchmark suite vs Mem0, Zep/Graphiti, Letta. Three outcomes:
- **Commit** — proceed to v2.1 + fundraise
- **Reposition** — ship as "agidb-lite: embedded cognitive memory for edge agents"
- **Retreat** — fold back into ctxgraph (predecessor)

See [PROJECT.md](../PROJECT.md) section 11 for the full threshold definitions.

### Success at v2.0 launch (month 9)

- 1M+ episodes on a laptop with sub-100ms p99 recall
- Match/beat Zep on LongMemEval-S (≥ 64 accuracy)
- 3× lower retrieval latency than Mem0 (p95 < 50ms)
- 3× lower token cost than Mem0 (< 2,500 tokens/query)
- All four cognitive benchmarks pass
- 1000+ GitHub stars
- 5+ design-partner deployments
- arxiv whitepaper posted

## v2.1 — Brain-alignment (2026, month 12)

**Additive expansion.** v2.0 substrate stays the core. Brain-alignment is the publishable differentiator that turns agidb from "another rust memory library" into "an artifact of brain-aligned cognitive science research with production rust deployment."

### What ships

- `agidb-sensory` crate — V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B encoders, Charikar 2002 random projection, VSA multimodal binding
- `observe_multimodal()` API — 30s video + audio + text → one episode HV
- Brain-calibrated surprise gating — θ_brain fit against TRIBE v2 predicted neural surprise on associative cortex
- `agidb-bams` crate — BAMS benchmark suite, six-cortical-network RSA harness, baselines (mem0, letta, zep, hipporag, raw V-JEPA)
- ICLR 2026 MemAgents workshop paper (or CCN 2026 backup)
- 11 crates total (v2.0's 9 + agidb-sensory + agidb-bams)

### Decision gate at v2.1

Phase 16, week 52. The brain-alignment work is judged on:
- BAMS suite open-source with reproducible baselines
- agidb wins BAMS in at least 3 of 6 functional networks (target: DMN, dorsal attention, frontoparietal)
- ICLR 2026 MemAgents paper accepted (or CCN 2026)
- Multimodal pipeline p50 latency ≤ 2s on a laptop CPU

If yes → proceed to seed round + v2.2 cognitive engine.
If no → reassess brain-alignment as a v2.2 retry or deprioritize.

See [brain-alignment.md](../architecture/brain-alignment.md) and [bams-benchmark.md](../architecture/bams-benchmark.md) for full detail.

## v2.2 — Cognitive engine (2027)

The first cognitive engine on top of the substrate. Adds operations that turn stored memory into active reasoning.

### What ships

- **Pattern completion via Hopfield networks.** Modern Hopfield (Ramsauer et al. 2021) over stored signatures. Given a partial cue, retrieve the full pattern. Implements "remembering" as continuous attractor dynamics over the signature space, not just nearest-neighbor lookup.
- **AGM belief revision.** Alchourrón-Gärdenfors-Makinson belief revision semantics. New evidence triggers principled revision of dependent beliefs. Replaces the v2.0 ad-hoc confidence math.
- **Analogical retrieval via HDC binding.** "If A is to B as X is to ?": bind(A, B) ⊕ X → answer signature. Recover via nearest-neighbor cleanup. Classic VSA analogy mechanism.
- **Learned projection** (if BAMS plateaus). Article XVIII clause 5 explicitly leaves this open for v2.2+. Replace Charikar 2002 random projection with a small MLP optimized against BAMS, only if the random baseline saturates.
- **Background consolidation scheduler.** Tokio-task-based, runs during idle periods. v2.0 ships synchronous consolidate(); v2.2 makes it automatic.
- **Procedure success-rate-based retrieval reweighting.** Floor 5 procedures with execution traces now influence which skills get retrieved in similar contexts.

### Decision gate at v2.2

End of 2027. Three design-partner production deployments running >6 months. Multi-week zero-touch uptime. At least one revenue-generating customer. If yes → v2.3 fundraise.

### Why this comes after brain-alignment

Pattern completion, AGM, and analogical retrieval are cognitive *operations* on top of the substrate. They need a credible substrate first (v2.0), benefit from brain-aligned encoders (v2.1), and add new capabilities on top. If brain-alignment validates the representations, v2.2 turns those representations into reasoning.

## v2.3 — Causal layer (2028)

Add causal reasoning capabilities. The substrate becomes capable of representing not just what happened but why it happened.

### What ships

- **Causal claim storage.** First-class `CausalClaim` type: "A caused B" with conditions, confidence, evidence. Stored as bound HDC patterns over (cause, effect, condition).
- **Intervention semantics.** Pearl-style do-calculus operations over stored causal claims. "What would have happened if X hadn't occurred?" answered via counterfactual replay.
- **World model fragments.** First-class `WorldModel` type. Causal claims compose into world model fragments. Models can be composed for prediction.
- **Causal-JEPA-style object-centric masking** (if relevant work has matured). Object-level latent prediction for compositional causal reasoning.
- **On-line learning state.** Persisted hyperparameter and online-learning rate state, recovers correctly across restarts.
- **HRR (Holographic Reduced Representations) as a secondary VSA format.** Real-valued vectors with circular convolution binding. Useful for analog scalar values (temperatures, scores, probabilities) that BSC can't represent natively.

### Decision gate at v2.3

End of 2028. Enterprise deal pipeline established. At least 3 paying customers with 6+ figure annual contracts. Series A raised.

## v2.4 — Production-grade (2029-2030)

The system goes from research-credible to enterprise-grade. Distributed mode, hardened safety, BCI experimentation.

### What ships

- **Distributed mode** (still optional). Replication, sharding by entity or session, cross-region failover. Embedded-first OSS remains canonical (constitution article III).
- **Formal safety guarantees on self-modification.** When the agent unlearns or revises core beliefs, formal guarantees about what was changed, audit trail completeness, and recoverability. Type-system enforced where possible.
- **Enterprise tier:** SSO, audit-log encryption, role-based access, compliance certifications (SOC 2, HIPAA, ISO 27001).
- **BCI input experimental.** `agidb-bci` crate. EEG/MEG ingestion via Brain-JEPA (arxiv 2406.19260) or signal-JEPA encoders. Surprise gating extends to neural signals.
- **Multi-agent shared memory.** Beyond v2.4's single-agent focus: shared memory pools, conflict resolution, federated consolidation. Inspired by BMAS multi-agent architectures.

### Decision gate at v2.4

Mid-2030. Revenue > $5M ARR. Series B raised. agidb is a production database with enterprise deployments and a >$50M valuation.

## v2.5 — AGI-grade (2031)

The full cognitive substrate. By year 5, agidb is either the de facto AGI substrate (because frontier labs and OSS AGI projects build on it) or it isn't (because the field moved past current paradigms — V-JEPA → next paradigm, HDC → spiking, etc).

### What ships (if the bet pays off)

- **Closed-loop self-modification.** The agent can rewrite its own goals, beliefs, and even procedures, with formal safety boundaries.
- **Causal reasoning over learned beliefs** as a core API.
- **Cognitive engine fully realized.** Pattern completion, AGM revision, analogical retrieval, causal reasoning, sleep-like consolidation, brain-aligned encoding — all integrated into one substrate.
- **Established interop standards.** Standard formats for cognitive substrate (.agidb files), shared benchmarks (BAMS evolved to BAMS-2), and interop with the broader AGI ecosystem (OpenCog Hyperon, Monty, frontier-lab proprietary substrates).
- **Production-grade with formal verification** where applicable. Critical paths formally verified for safety properties.

### If the bet doesn't pay off

The field will have moved past current paradigms. V-JEPA may be replaced by something post-JEPA. HDC may be replaced by spiking neural networks on neuromorphic hardware. agidb v2.5 either pivots aggressively (becomes v3) or sunsets gracefully with substantial OSS legacy. Both outcomes are acceptable if the journey produces real value along the way.

## What stays constant across the 5 years

- **Constitution articles I-XVIII.** The principles are the invariants. Code rots; principles don't.
- **The wedge.** Content-addressable HDC retrieval, bi-temporal supersession, embedded Rust binary, no LLM in read path, first-class cognitive primitives, non-destructive unlearn. These differentiators don't change.
- **The audience.** Developers building autonomous agents, regulated industries, AGI-curious researchers, local-first builders.
- **The OSS-first commitment.** The embedded engine stays free, complete, self-hostable, Apache-2.0.

## What evolves across the 5 years

- **The encoder stack.** V-JEPA 2 → V-JEPA 3 (likely 2026-2027) → post-JEPA paradigm (2028+). agidb tracks the best available open-weight encoders.
- **The VSA format.** Default BSC throughout, with HRR as secondary in v2.3, SBDR (sparse) as candidate for v2.5.
- **The brain-encoding ground truth.** TRIBE v2 → TRIBE v3 (likely 2027) → whatever the next-best brain encoder is.
- **The benchmark surface.** LongMemEval/LoCoMo/BEAM + BAMS in v2.1. New benchmarks emerge; agidb runs them all.
- **The substrate's scale.** v2.0 single-laptop; v2.4 enterprise multi-node; v2.5 substrate for the AGI ecosystem.

## Risks and mitigations

| Risk | Probability | Mitigation |
|---|---|---|
| v2.0 decision gate fails | 30% | Reposition path defined; sochdb code valuable even if standalone |
| BAMS paper rejected at all venues | 15% | Multiple venue options; benchmark stands on its own as a public artifact |
| Direct rust HDC competitor emerges | 30% | Move fast on v2.1 brain-alignment; differentiate on cognitive primitives |
| Frontier lab open-sources a competing substrate | 20% | unlikely (no signals as of May 2026); agidb's OSS-first commitment matches |
| V-JEPA 2 deprecated by post-JEPA paradigm | 20% by 2028 | Trait-based encoder abstraction; swap encoders without rewriting substrate |
| TRIBE v2 replaced by TRIBE v3 mid-cycle | 60% by 2027 | BAMS protocol is version-aware; recalibration documented |
| Funding environment for deep-tech infra deteriorates | 30% | Bootstrap-friendly architecture; revenue paths from enterprise contracts |
| Founder burnout over 5 years | 40% | Realistic milestone pacing; v2.0 ship at month 9 buys credibility for slower v2.2+ |
| Major safety incident in deployed agents | 20% | Constitution article on safety; cascading unlearn + audit log makes incidents recoverable |

## Why this trajectory makes sense

Three reasons.

1. **Each version is independently valuable.** v2.0 ships as a credible substrate even if v2.1+ never happens. v2.1 ships as a credible brain-aligned substrate with a workshop paper even if v2.2+ never happens. The optionality compounds.

2. **The cognitive primitives compound.** Goals + beliefs + sensory + self-model + unlearn (v2.0) → multimodal + brain-alignment (v2.1) → pattern completion + analogical retrieval + AGM (v2.2) → causal claims + world models (v2.3) → BCI + multi-agent (v2.4) → closed-loop self-mod (v2.5). Each version's capability requires the previous version's foundation.

3. **The competitive landscape favors a 5-year horizon.** Mem0 ($24M, Series A), Letta, Zep, Cognee — all are racing on application-layer agent memory. None have committed to a 5-year substrate roadmap. By month 12, agidb is the only published cognitive substrate with brain-aligned evaluation. By year 3, the gap widens. By year 5, agidb is either the substrate or it isn't — but no other team will have run this play.

## The single non-negotiable

If at any point during the 5-year trajectory the constitution is violated to chase a feature, a customer, or a paper — the bet has been lost regardless of how good the numbers look. The substrate's value compounds because the principles don't move. Pivoting on principles ends the project; pivoting on tactics is normal.

See [constitution.md](../spec/constitution.md).
