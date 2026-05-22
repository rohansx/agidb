# agidb — Biological Mapping

> The seven cognitive floors of agidb mapped to the cognitive-psychology and
> neuroscience literature they're derived from. Why each floor exists, what
> the academic foundations are, and what's preserved/lost in the
> simplification.

## Why this document exists

agidb's seven-floor model isn't a feature list — it's an architectural commitment grounded in 50 years of memory research. The model has to be honest about which floors are well-established cognitive primitives, which are agidb's specific extensions, and which are convenient simplifications.

The mapping isn't 1:1. agidb is not a brain simulator (that's Numenta Monty's project) or a brain encoder (that's TRIBE v2's project). agidb is a database that uses cognitive-psychology frame works as productive abstractions for the kinds of memory an autonomous agent needs. This doc tells you which biological/cognitive sources each floor draws from.

In v2.1, the visual extraction pathway (V-JEPA 2) provides one additional empirical anchor: the encoder is itself part of TRIBE v2's stack predicting fMRI activity in visual cortex. So at the encoder layer, agidb v2.1 has measurable visual-cortex alignment.

## The seven floors

| Floor | Cognitive name | Primary academic source | What agidb takes |
|---|---|---|---|
| 1 | Sensory memory | Sperling 1960, Cowan 1995 | Short-lived signal buffer, surprise-gated promotion |
| 2 | Working memory | Baddeley & Hitch 1974, Cowan 2001 | Capacity-bounded active context, session-scoped recency |
| 3 | Episodic memory | Tulving 1972, 1983, 2002 | Time/place/person tagged events with bi-temporal stamps |
| 4 | Semantic memory | Quillian 1968, Tulving 1972 | Decoupled general knowledge, consolidated from episodes |
| 5 | Procedural memory | Squire 1992, Anderson 1993 (ACT-R) | Skills with execution traces and success rates |
| 6 | Goals + Beliefs | Newell 1990 (Soar), Bratman 1987 (BDI) | First-class typed primitives for agent intentionality |
| 7 | Self-model + audit | Metzinger 2003, Hofstadter 2007, V-JEPA 2 EMA | Append-only learning log + slowly-drifting self-vector |

Each floor is detailed below.

---

## Floor 1 — Sensory memory

### The biology

Sperling 1960 established the existence of a short-lived sensory store (iconic memory for vision, echoic for audio) holding raw signal for hundreds of milliseconds. Most of this signal is overwritten by subsequent input; a small fraction gets selected for further processing based on attention and salience.

Cowan 1995 generalized this to a "focus of attention" within working memory, capacity-bounded and renewable.

### What agidb takes

A short-lived sensory buffer holding recent input before promotion to episodic memory. Implemented as a fixed-capacity ring (default 1000 frames or 60 seconds). Promotion happens when surprise exceeds threshold, mimicking attention-gated transfer to longer-term storage.

### V-JEPA 2 visual cortex alignment (v2.1)

In v2.1, the visual extraction pathway runs V-JEPA 2 over 64-frame video windows. V-JEPA 2 is part of TRIBE v2's encoder stack predicting fMRI BOLD in visual cortex (V1, V2, V4, MT, IT). This gives floor 1's visual pathway an empirical anchor: the encoder itself produces representations that predict visual-cortex activation patterns with pearson r ≈ 0.2-0.4 depending on the region.

Note: this is encoder-level alignment, not full-floor alignment. agidb's floor 1 is a buffer with surprise gating; the alignment claim is about what the encoder represents, not what the buffer dynamics simulate.

### What's preserved

- The short-lived, capacity-bounded, attention-gated character of sensory memory.
- The biological function: filter noise, promote the salient.
- v2.1: empirically grounded visual representations via V-JEPA 2.

### What's lost

- The continuous-stream character (agidb samples; biology is continuous).
- The modality-specific decay rates (agidb uses unified ring; biology has different rates for iconic vs echoic).
- The role of attention beyond surprise (agidb's surprise is one signal; biological attention is many).

### Why this floor matters

Without sensory gating, every observation goes to episodic memory. Storage grows unbounded; consolidation has too much work; the agent over-remembers trivia. The sensory floor is the attentional filter.

---

## Floor 2 — Working memory

### The biology

Baddeley & Hitch 1974 proposed working memory as a system with a central executive plus modality-specific stores (phonological loop, visuospatial sketchpad). Cowan 2001 argued for a smaller capacity (~4 chunks) than Miller's classic 7±2, based on attention-focus constraints.

### What agidb takes

A capacity-bounded active context, session-scoped, recency-weighted. Implemented as a session ID + recency boost on top of episodic retrieval — not a separate floor with its own storage.

### What's preserved

- Active context relevant to current task.
- Capacity-bounded (we don't load all episodes; we load a recency window).
- Session-scoped (one task's working memory doesn't pollute another's).

### What's lost

- The modality-specific stores (no phonological loop / visuospatial sketchpad).
- The central executive (no agidb-specific "executive" — that's the agent's job).
- The continuous rehearsal that maintains working-memory items in biology.

### Why this floor exists

The recency + session-bound retrieval pattern is universal for autonomous agents. By making it floor-level rather than implementing it in agent code, we keep the recency math consistent and testable.

---

## Floor 3 — Episodic memory

### The biology

Tulving 1972 introduced episodic memory as autobiographical events with time, place, and personal-context tags. Tulving 1983 distinguished episodic from semantic in detail: episodic is "I remember when..."; semantic is "I know that...". Tulving 2002 added autonoetic consciousness (the felt sense of personal time-travel) as the defining feature.

Hippocampus + medial temporal lobe are the neural substrates; lesions to MTL produce the canonical anterograde amnesia (Scoville & Milner 1957, patient HM).

### What agidb takes

Events with bi-temporal stamps (valid time + transaction time), full provenance, and HDC signatures. Each episode is a typed `Episode` shape with a `EpisodeId`, stored in redb with its 1KB signature in `signatures.dat`.

The bi-temporal model is the key cognitive concession: episodes know both *when they happened* (valid time) and *when they were learned* (transaction time). Both queries are common ("what happened on May 15?" vs "what did I know on May 15?").

### What's preserved

- The "when, where, who, what" tagging.
- The autobiographical character (every episode is tied to the agent's experience, not abstract).
- The full-context preservation (no chunk-and-embed; we keep the original text).

### What's lost

- The felt sense of remembering (no autonoetic consciousness; that's a hard problem).
- The reconstructive character (episodic recall in biology rebuilds episodes from gist + plausible inferences; agidb returns stored episodes verbatim).
- The role of emotion in encoding strength (no amygdala simulation).

### Why this floor matters

Episodic memory is the foundation. Without it, semantic and procedural floors have nothing to consolidate. Working memory has nothing to surface. The self-model has nothing to audit.

---

## Floor 4 — Semantic memory

### The biology

Quillian 1968 modeled semantic memory as a network of concepts. Tulving 1972 made the episodic/semantic distinction. Squire 1992 distinguished declarative (episodic + semantic) from non-declarative (procedural + priming + conditioning).

In modern neuroscience, semantic memory is distributed across cortex (temporal, parietal, frontal), with the anterior temporal lobes (ATL) thought to play a hub role (Patterson, Nestor & Rogers 2007).

### What agidb takes

Decoupled general knowledge as `SemanticAtom` records produced by the consolidation worker. Atoms carry `evidence_count`, `source_episodes`, and confidence. They are typed (subject, predicate, object) with optional time validity.

The consolidation worker implements McClelland-McNaughton-O'Reilly's Complementary Learning Systems theory (1995): hippocampal episodic encoding is fast and pattern-separating; cortical semantic consolidation is slow and pattern-completing. agidb's `consolidate()` function explicitly does this — clusters similar episodes into semantic atoms.

### What's preserved

- Episodic-to-semantic consolidation as a slow, evidence-accumulating process.
- The hub character (semantic atoms aren't tied to specific episodes; they're decoupled summaries).
- Confidence as a function of evidence count.

### What's lost

- The graded conceptual structure (no superordinate / subordinate hierarchy at the storage level; that's emergent from triples).
- The ATL hub topology.
- The role of language in shaping semantic structure (no language-specific priors; we use whatever predicate vocabulary the triples ship with).

### Why this floor matters

Episodic memory grows unboundedly. Semantic memory is how agidb compresses repeated experience into reusable knowledge. Without consolidation, agidb is just a vector database of episodes. With it, agidb is a substrate that learns.

---

## Floor 5 — Procedural memory

### The biology

Squire 1992 classified procedural memory under non-declarative learning. Anderson 1993's ACT-R framework formalized procedural knowledge as production rules with utility scores updated by execution success.

In neuroscience, procedural memory recruits the basal ganglia, cerebellum, and motor cortex, with the dorsal striatum as the canonical seat.

### What agidb takes

Skills/workflows as `Procedure` records with `ExecutionTrace` logs. Each procedure has success counts, average duration, and the linked traces. Retrieval can be by name or by HDC similarity to context (find a procedure relevant to the current situation).

### What's preserved

- Procedural-vs-declarative distinction (procedures are a separate type, not encoded as episodes).
- Learning by execution (success counts increase utility for future retrieval).
- Contextual retrieval (similar contexts retrieve similar procedures).

### What's lost

- The motor-skill character of biological procedural memory (we handle abstract workflows, not motor skills).
- The implicit-learning character (agidb's procedures are explicitly written; biological procedural memory often forms without explicit rules).
- The role of repetition in strengthening procedures (we count successes, not repetitions; close but not identical).

### Why this floor matters

A long-running agent learns *how to do things* in addition to *what is true*. Without procedural memory, every task re-derives the workflow from scratch. With it, the agent gets faster over time at recurring tasks.

---

## Floor 6 — Goals and Beliefs

### The biology / cognitive theory

This floor is more rooted in cognitive architecture than neuroscience proper.

**Goals:** Newell 1990's Soar architecture has goal stacks; Newell & Simon 1972 (GPS) treats problem-solving as goal decomposition. Cognitive psychology distinguishes hierarchical goals (parent-child) from operational goals (immediate action targets). The Belief-Desire-Intention (BDI) framework of Bratman 1987 / Rao & Georgeff 1995 formalized goals as intentions.

**Beliefs:** Bratman 1987's BDI framework; Pearl 1988 on probabilistic belief; AGM 1985 (Alchourrón-Gärdenfors-Makinson) on belief revision semantics.

In neuroscience, the prefrontal cortex (PFC) houses goal representations (Miller & Cohen 2001), with dorsolateral PFC for working-memory-style goal maintenance and ventromedial PFC for value/preference. The default mode network (DMN) is implicated in autobiographical thinking and self-referential cognition.

### What agidb takes

First-class `Goal` (state machine with parent-child) and `Belief` (confidence-tracked, revisable, with audit log). Not implemented in agent code as text fields — typed substrate primitives with their own redb tables.

### What's preserved

- Goals have lifecycle (active/paused/completed/abandoned).
- Goals form hierarchies (parent-child).
- Beliefs are graded (confidence) and revisable.
- Belief revision has structure (BeliefRevision with reason and triggering evidence).

### What's lost

- The mood/affect modulation of goals (no emotional weighting).
- The desire/intention distinction (we have one type, Goal, not three).
- The full AGM formal semantics of belief revision (we approximate; v2.2 adds AGM-proper).
- The neural distinction between maintenance and monitoring.

### Why this floor matters

This is agidb's wedge. Existing systems store goals and beliefs as text. agidb stores them as typed primitives with state machines, revision audit, and HDC signatures usable in goal-biased retrieval. The cognitive primitives are what turn agidb from "another memory db" into "a cognitive substrate."

---

## Floor 7 — Self-model

### The biology / philosophy

The hardest floor to ground biologically. Self-model concepts come from:
- Metzinger 2003's "Being No One": the phenomenal self-model as a representational structure.
- Hofstadter 1979/2007: strange loops and self-reference.
- Damasio 1999: the "autobiographical self" as a layer above the proto-self.

In neuroscience, the default mode network (DMN) is implicated in self-referential thought (Buckner, Andrews-Hanna & Schacter 2008). The medial PFC, posterior cingulate cortex, and angular gyrus are central nodes.

### V-JEPA 2 + TRIBE inspiration (v2.1)

agidb's self-vector design borrows from two specific architectures:
- **V-JEPA 2's EMA target network:** prevents representation collapse by slowly drifting against gradient updates. The EMA rate (~0.99 momentum) is fast enough to track current state, slow enough to provide stability.
- **TRIBE v2's per-subject embedding layer:** captures "what makes this individual unique" via a learnable subject token that conditions all predictions.

agidb combines both: the self-vector is a slowly-drifting bundle of consolidated atoms (EMA-like), serving as a per-agent embedding (TRIBE-like). Update rate α ≈ 0.05 per consolidation epoch (slower than V-JEPA's 0.01 momentum but in similar spirit).

### What agidb takes

Two components:
1. **`learning_events` log:** append-only record of every state change in the system. Episodes stored, beliefs revised, goals completed, atoms formed, contradictions detected, unlearns performed.
2. **`self_vector`:** an 8192-bit hypervector representing "what kind of agent am I right now," updated as `self_vector ← (1-α) self_vector + α bundle(consolidated_atoms)` on each consolidation epoch, and subtracted from on unlearn (`self_vector ← self_vector - α bundle(tombstoned_signatures)`).

### What's preserved

- The audit-of-self character: the agent can ask "what did I learn this week?" and get a structured answer.
- The slow-drift character of self-representation (not instantaneous mood changes; gradual identity evolution).
- The unlearn-affects-self property: forgetting changes who the agent is.

### What's lost

- Phenomenal consciousness (we don't claim agidb has experience).
- The strange-loop self-reference (Hofstadter's notion; we have a representation but not a self-pointing structure).
- The narrative continuity that humans construct from autobiographical memory.

### Why this floor matters

A self-modifying agent needs introspection. Without a self-model, the agent acts but cannot reason about its actions. With one, the agent can explain itself, debug itself, and (in v2.5) modify itself with formal safety guarantees about what it changed.

The self-vector is the reason unlearn has to subtract from it (constitution article XVI). Without subtraction, forgotten data still contaminates the agent's self-representation. With it, unlearn is principled.

---

## Cross-floor properties

### Consolidation: floor 3 → floor 4 → floor 6

The `consolidate()` worker is the analog of sleep-dependent memory consolidation (McClelland-McNaughton-O'Reilly 1995, Diekelmann & Born 2010). It runs in the background or on-demand, scans recent episodes, clusters them by HDC similarity, and produces semantic atoms. High-evidence atoms get promoted to beliefs.

The self-vector update happens at the end of each consolidation pass (v2 addition).

### Multimodal binding: floor 1 → floor 3 (v2.1)

In v2.1, the sensory floor receives multimodal input (video + audio + text). Each modality goes through its frozen encoder, projects to an 8192-bit HV via Charikar 2002 random projection, and the modality signatures bind via VSA role-filler XOR into one episode HV. The episode is stored at floor 3 with the bound signature.

Critically, the binding is factorable: each modality component can be recovered from the bound episode via XOR with its ROLE_* hypervector. This is structurally different from attention-based fusion (TRIBE v2, dense embedding models) which is not factorable.

### Unlearn: cross-floor cascading

Unlearning a concept cascades:
1. Floor 3 episodes referencing the concept get tombstoned.
2. Floor 4 semantic atoms derived from those episodes get recomputed.
3. Floor 6 beliefs whose evidence drops below threshold get withdrawn or revised.
4. Floor 5 procedures whose trigger contexts no longer exist get marked degraded.
5. Floor 7 self-vector subtracts the tombstoned signatures.
6. Floor 7 learning log emits `LearningEvent::Unlearned` (permanent).

This is the cross-floor cascade that turns delete-from-a-table into principled cognitive forgetting.

### Goal-biased retrieval: floor 6 → floor 3+4 retrieval

When active goals exist, their HDC signatures up-weight similar episodes and semantic atoms during recall. This mirrors the role of prefrontal cortex in biasing attention toward goal-relevant memories (Miller & Cohen 2001).

---

## What agidb explicitly doesn't model

- **Emotion / affect:** no amygdala, no emotional encoding strength. Possibly v2.3+.
- **Implicit learning:** no priming, no statistical learning that occurs without explicit observation. Possibly v2.5+.
- **Sleep stages:** consolidation runs whenever, not in REM-vs-NREM cycles.
- **Forgetting curves:** decay is uniform (90-day inactivity threshold), not Ebbinghaus-shaped.
- **Reconstructive memory:** stored episodes are returned verbatim, not reconstructed from gist (biology does the opposite).
- **Mood-congruent retrieval:** no mood priming.
- **Phenomenal experience:** no claims about subjective experience.

These omissions are intentional. agidb is a database, not a cognitive simulator. The cognitive-psychology framework is productive abstraction, not literal commitment.

---

## What v2.1 brain-alignment adds

v2.1 adds one empirical anchor: the encoder stack (V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B) is the same stack TRIBE v2 uses to predict fMRI BOLD across cortex. This means:

1. **At the encoder level**, agidb v2.1 produces representations measurably aligned with predicted visual cortex (V-JEPA 2), auditory cortex (Wav2Vec-BERT), and language cortex (Llama-3.2-3B).
2. **At the episode signature level**, agidb v2.1 can be benchmarked against TRIBE-predicted cortical activation patterns via BAMS (representational similarity analysis across six functional networks).
3. **At the surprise-gating level**, agidb v2.1's threshold is calibrated against neural surprise predicted by TRIBE on associative cortex (DMN, TPJ, dlPFC).

These are empirical claims, not metaphorical ones. agidb still doesn't claim phenomenal experience or biological fidelity. It claims its representations align with TRIBE's predictions of cortical representations, measurably, on naturalistic stimuli.

See [brain-alignment.md](../architecture/brain-alignment.md) and [bams-benchmark.md](../architecture/bams-benchmark.md) for the v2.1 detail.

---

## Suggested reading

For each floor, the canonical references the framework draws from:

| Floor | Primary refs |
|---|---|
| 1 | Sperling 1960; Cowan 1995; (v2.1) Assran et al. 2025 (V-JEPA 2) |
| 2 | Baddeley & Hitch 1974; Cowan 2001 |
| 3 | Tulving 1972, 1983, 2002; Scoville & Milner 1957 |
| 4 | Quillian 1968; Patterson, Nestor & Rogers 2007; McClelland, McNaughton & O'Reilly 1995 |
| 5 | Squire 1992; Anderson 1993 (ACT-R) |
| 6 | Newell 1990 (Soar); Bratman 1987 (BDI); Pearl 1988 |
| 7 | Metzinger 2003; Hofstadter 2007; (v2.1) Assran et al. 2025 (V-JEPA 2 EMA); d'Ascoli et al. 2025 (TRIBE v2 per-subject layer) |
| Multimodal binding | Plate 1995 (HRR); Kanerva 1994/1997 (BSC); Charikar 2002 |
| Brain encoding | d'Ascoli et al. 2025 (TRIBE v1); Banville/King et al. 2026 (TRIBE v2) |
| Consolidation | McClelland-McNaughton-O'Reilly 1995; Diekelmann & Born 2010 |
| Prefrontal goal-biasing | Miller & Cohen 2001 |
| AGM belief revision | Alchourrón, Gärdenfors & Makinson 1985 |

---

## Why this framing is productive

The cognitive-psychology framing makes agidb easier to:
- **Explain to developers** ("oh, like episodic vs semantic memory") without requiring database expertise.
- **Defend in papers** with grounded academic references rather than hand-waving.
- **Extend in new versions** by adding floors or refining floor mechanics with continued research grounding.
- **Differentiate from competitors** who store flat blobs of text or vectors. Floors are structure.

In v2.1, the framing extends to brain-aligned cognitive psychology: not just "memory like a brain" metaphorically, but "memory representations measurably aligned with predicted cortical activations" empirically. That's the upgrade brain-alignment provides — it makes the biological mapping defensible at the encoder and episode-signature levels, even though the higher-level cognitive operations remain abstractions.

The framing's value is in its productive constraints, not literal biological accuracy. agidb is a cognitive substrate, not a brain.
