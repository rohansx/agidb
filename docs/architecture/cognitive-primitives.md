# agidb — Cognitive Primitives

> The five new first-class types that make agidb a cognitive substrate rather
> than a memory database: Goals, Beliefs, Sensory frames, Self-model, and the
> Unlearn API. Extended in v2.1 with multimodal sensory and brain-aligned
> surprise gating.

## The five primitives

Every existing agent memory system stores text or vectors. agidb stores **typed cognitive primitives** with their own retrieval semantics, audit trails, and lifecycle behavior. The five primitives are:

| Primitive | Floor | What it represents |
|---|---|---|
| **Goal** | 6 | What the agent wants. State machine with parent-child hierarchy. |
| **Belief** | 6 | What the agent thinks is true. Revisable with audit. |
| **SensoryFrame** | 1 | Raw input before promotion to episodic memory. Surprise-gated. |
| **LearningEvent + SelfVector** | 7 | The self-model: audit log of state changes + slowly-drifting self-representation. |
| **UnlearnTarget** | cross-floor | Reference to something the agent should forget. Triggers cascading non-destructive removal. |

Each is a Rust type, has its own redb table, has explicit API methods, and is property-tested.

---

## Goals (floor 6)

### What a goal is

```rust
pub struct Goal {
    pub id: GoalId,
    pub parent_id: Option<GoalId>,
    pub description: String,
    pub state: GoalState,
    pub success_criteria: Vec<SuccessCriterion>,
    pub deadline: Option<DateTime<Utc>>,
    pub signature: HV,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub provenance: Provenance,
}

pub enum GoalState {
    Active,
    Paused { since: DateTime<Utc>, reason: String },
    Completed { at: DateTime<Utc>, evidence: Vec<EpisodeId> },
    Abandoned { at: DateTime<Utc>, reason: String },
}
```

### State machine

```
        Active
        ↗   ↓ ↘
    (revive) ↓ (complete) → Completed
        ↘   ↓ ↙          (terminal)
        Paused → Active
            ↓
        Abandoned (terminal)
```

Validation: Completed and Abandoned are terminal. Pause/resume preserve history. State transitions emit `LearningEvent::GoalStateChanged`.

### Parent-child hierarchy

Goals form a tree. A parent goal completes when all its children complete (configurable: any-child or all-children semantics). Each goal carries an HDC signature derived from its description + parent context. Goal-biased recall uses these signatures to up-weight relevant memories.

### API

```rust
impl Agidb {
    pub async fn set_goal(&self, goal: Goal) -> Result<GoalId>;
    pub async fn revise_goal(&self, id: GoalId, patch: GoalPatch) -> Result<()>;
    pub async fn complete_goal(&self, id: GoalId, evidence: Vec<EpisodeId>) -> Result<()>;
    pub async fn abandon_goal(&self, id: GoalId, reason: String) -> Result<()>;
    pub async fn active_goals(&self) -> Result<Vec<Goal>>;
    pub async fn goal_tree(&self, root: GoalId) -> Result<GoalTree>;
    pub async fn get_goal(&self, id: GoalId) -> Result<Option<Goal>>;
}
```

### Why first-class

Existing systems store goals as text in episode descriptions. The agent code then re-parses, re-tracks, re-evaluates. State machines live in agent code, not the database. This is brittle. agidb makes goals a typed shape so:
- Goal-biased retrieval is one HDC similarity computation.
- State transitions are auditable.
- Hierarchy is queryable.
- Success criteria are testable.

### Why this matters at scale

A long-running agent accumulates hundreds to thousands of goals over months. Without first-class goal storage, the agent has no reliable way to ask "what was I trying to do last week?" or "what subgoals support my current top-level goal?" agidb makes these queries trivial.

---

## Beliefs (floor 6)

### What a belief is

```rust
pub struct Belief {
    pub id: BeliefId,
    pub claim: String,
    pub subject: ConceptId,
    pub predicate: String,
    pub object: Value,
    pub confidence: f32,                    // [0.0, 1.0]
    pub evidence: Vec<EpisodeId>,
    pub contradictions: Vec<EpisodeId>,
    pub revision_log: Vec<BeliefRevision>,
    pub signature: HV,
    pub t_valid_start: DateTime<Utc>,
    pub t_valid_end: Option<DateTime<Utc>>,
    pub t_tx_start: DateTime<Utc>,
    pub t_tx_end: Option<DateTime<Utc>>,
    pub provenance: Provenance,
}

pub struct BeliefRevision {
    pub timestamp: DateTime<Utc>,
    pub previous_confidence: f32,
    pub new_confidence: f32,
    pub triggering_evidence: Option<EpisodeId>,
    pub reason: String,
}
```

### Revision math

When new evidence arrives:
1. If evidence supports the claim: confidence increases by Bayesian update over evidence count.
2. If evidence contradicts: confidence decreases, contradiction added to `contradictions`.
3. If confidence falls below 0.5 (configurable): belief is withdrawn (state preserved via bi-temporal closure).
4. Every change appends a `BeliefRevision` to `revision_log`.

LLMs may participate at write time for revision (constitution article IV amendment): the LLM is asked "does evidence E contradict claim C?" with structured prompt; output is parsed into a typed `RevisionDecision`. The decision plus the LLM's reasoning becomes the `reason` field. The read path stays LLM-free.

### API

```rust
impl Agidb {
    pub async fn assert_belief(&self, belief: Belief) -> Result<BeliefId>;
    pub async fn revise_belief(
        &self,
        id: BeliefId,
        new_evidence: EpisodeId
    ) -> Result<RevisionReport>;
    pub async fn what_do_i_believe(&self, about: ConceptId) -> Result<Vec<Belief>>;
    pub async fn belief_history(&self, id: BeliefId) -> Result<Vec<BeliefRevision>>;
    pub async fn withdraw_belief(&self, id: BeliefId, reason: String) -> Result<()>;
}
```

### Why first-class

Beliefs differ from facts because they have:
- **Confidence:** facts are atomic; beliefs are graded.
- **Revision history:** facts get superseded; beliefs get revised with reasons.
- **Contradictions:** beliefs track conflicting evidence; facts don't.
- **Provenance to claim level:** "I believe X because of episodes A, B, C" is queryable.

Storing beliefs as `Belief` rather than `Episode` enables:
- "Show me all my current beliefs about Sarah."
- "Why did I revise my belief about Letta's architecture?"
- "What's my most uncertain belief?"
- "What evidence currently supports / contradicts each belief?"

### Constitution article XVII

Beliefs are revisable, never overwritten. Every revision appends to the log. The agent can always answer "what did I believe last week, and what changed my mind?"

---

## Sensory frames (floor 1)

### What a sensory frame is

```rust
pub struct SensoryFrame {
    pub id: SensoryId,
    pub modality: Modality,
    pub data: SensoryData,
    pub received_at: DateTime<Utc>,
    pub surprise_score: f32,
    pub promoted_to: Option<EpisodeId>,
}

pub enum Modality {
    Text,
    Image { path: PathBuf },
    Audio { path: PathBuf, duration_ms: u32 },
    Video { path: PathBuf, frame_count: u32, fps: f32 },
    Multimodal { components: Vec<Modality> },  // v2.1+
}

pub enum SensoryData {
    InlineText(String),
    BlobRef(PathBuf),
    InlineLatent { encoder: EncoderId, latent: Vec<f32> },  // v2.1+
}
```

### Ring buffer

The sensory buffer is a fixed-capacity ring (default 1000 frames or 60 seconds). When full, oldest frames are dropped unless promoted. Promotion happens when:
1. `surprise_score > θ` (threshold; v2.0 default 0.4, v2.1 brain-calibrated)
2. OR explicit `observe()` call promotes the frame
3. OR session boundary forces a flush

### Surprise gating (v2.0)

```
surprise(frame) = 1 - similarity(frame_signature, bundle_of(recent_beliefs))
```

If the frame's signature is highly similar to the agent's current beliefs, surprise is low → don't store. If it's a novel observation, surprise is high → promote to episodic.

### Surprise gating (v2.1, brain-calibrated)

Same formula, but threshold θ_brain is empirically fit against TRIBE v2's predicted neural surprise on associative cortex (TPJ, dlPFC, DMN). See [brain-alignment.md](./brain-alignment.md).

### Multimodal sensory (v2.1)

```rust
impl Agidb {
    pub async fn observe_multimodal(
        &self,
        video: Option<VideoClip>,
        audio: Option<AudioClip>,
        text: Option<String>,
        ctx: ObserveContext,
    ) -> Result<EpisodeId>;
}
```

The pipeline:
1. Each modality runs through its frozen encoder:
   - Video → V-JEPA 2 → 1024d latent
   - Audio → Wav2Vec-BERT → 1024d latent
   - Text → Llama-3.2-3B → 2048d latent
2. Each latent projects to 8192-bit HV via Charikar 2002 random projection.
3. Brain-calibrated surprise is computed.
4. If surprise > θ_brain: promoted via VSA role-filler binding into one episode signature.
5. Stored in redb with multimodal metadata.

The resulting episode signature is factorable: each modality component can be recovered from the bound episode by XORing with the appropriate ROLE_* hypervector. This is the structural advantage over TRIBE's attention fusion and over mem0/letta/zep's dense embeddings.

### API

```rust
impl Agidb {
    // v2.0
    pub async fn observe_sensory(&self, frame: SensoryFrame) -> Result<SensoryId>;
    pub async fn working_state(&self) -> Result<SensoryBuffer>;
    pub async fn surprise_score(&self, frame: &SensoryFrame) -> Result<f32>;

    // v2.1
    pub async fn observe_multimodal(
        &self,
        video: Option<VideoClip>,
        audio: Option<AudioClip>,
        text: Option<String>,
        ctx: ObserveContext,
    ) -> Result<EpisodeId>;
    pub async fn surprise_score_brain_calibrated(
        &self,
        frame: &MultimodalFrame
    ) -> Result<f32>;
    pub async fn extract_modality_signature(
        &self,
        episode_id: EpisodeId,
        modality: Modality,
    ) -> Result<Option<HV>>;
}
```

### Why first-class

Without a sensory floor, every observation goes straight to episodic memory. Storage grows unbounded; consolidation has too much to consolidate; the agent over-remembers trivia. The sensory buffer + surprise gating is the agent's *attentional filter*. It's how the agent decides what's worth remembering.

In v2.1, multimodal sensory makes agidb the only agent memory substrate that handles video and audio as first-class inputs, not as text descriptions of media.

---

## Self-model (floor 7)

### What the self-model is

Two components:
1. **`learning_events`** — append-only log of every introspectable event the system records.
2. **`self_vector`** — a slowly-drifting 8192-bit hypervector representing "what kind of agent am I right now."

### Learning events

```rust
pub enum LearningEvent {
    EpisodeStored { id: EpisodeId, at: DateTime<Utc> },
    MultimodalEpisodeStored { id: EpisodeId, modalities: Vec<Modality>, at: DateTime<Utc> },
    GoalStateChanged { id: GoalId, from: GoalState, to: GoalState, at: DateTime<Utc> },
    BeliefAsserted { id: BeliefId, claim: String, confidence: f32, at: DateTime<Utc> },
    BeliefRevised { id: BeliefId, revision: BeliefRevision },
    BeliefWithdrawn { id: BeliefId, reason: String, at: DateTime<Utc> },
    SensoryFramePromoted { sensory_id: SensoryId, episode_id: EpisodeId, surprise: f32 },
    SemanticAtomFormed { atom_id: AtomId, source_episodes: Vec<EpisodeId>, at: DateTime<Utc> },
    ContradictionDetected { atoms: Vec<AtomId>, at: DateTime<Utc> },
    Unlearned {
        target: UnlearnTarget,
        cascade_size: usize,
        self_vector_drift: u32,
        audit_id: AuditId,
        at: DateTime<Utc>,
    },
    AttentionTraced { recall_id: RecallId, signatures_considered: usize, at: DateTime<Utc> },
    SelfVectorUpdated { drift_hamming: u32, at: DateTime<Utc> },
    ConsolidationRun { atoms_created: usize, contradictions: usize, at: DateTime<Utc> },
}
```

Closed enum (constitution article XV implication). New event types require ADR.

### Self-vector

Inspired by:
- **TRIBE v2's per-subject embedding layer:** captures "what makes this individual unique" via a learnable token.
- **V-JEPA 2's EMA target network:** prevents representation collapse by slowly drifting against gradient updates.

agidb's self-vector is a slowly-drifting 8192-bit HV updated each consolidation epoch:

```
self_vector ← (1 - α) * self_vector + α * bundle(consolidated_atoms)
```

with α ≈ 0.05. (Bundle is per-bit majority over the input set, then thresholded back to binary.)

### Critical: self-vector subtraction on unlearn

When episodes are tombstoned via the unlearn API:

```
self_vector ← self_vector - α * bundle(tombstoned_signatures)
```

(In binary HDC: bundle the tombstoned signatures, XOR with self_vector, then threshold.)

Without this step the self-model still "remembers" unlearned concepts as centroid contamination. With it, unlearn affects the self-vector. This makes the unlearn primitive *real*.

### What you can do with the self-model

```rust
impl Agidb {
    pub async fn what_did_i_learn(
        &self,
        since: DateTime<Utc>
    ) -> Result<Vec<LearningEvent>>;

    pub async fn attention_trace(
        &self,
        recall_id: RecallId
    ) -> Result<Option<AttentionTrace>>;

    pub async fn self_vector(&self) -> Result<HV>;
    pub async fn self_vector_at(&self, time: DateTime<Utc>) -> Result<HV>;

    pub async fn introspect(&self, query: IntrospectionQuery) -> Result<IntrospectionResult>;
}
```

Example questions the self-model answers:
- "What episodes did I store this week?"
- "Which goals changed state in the last 24 hours?"
- "What beliefs did I revise, and what evidence triggered it?"
- "What was I attending to during recall_id X?"
- "How much has my self-vector drifted in the last month?"

### Why this matters

A self-modifying agent needs introspection. Without a self-model, the agent can act but cannot reason about its own actions. With one, the agent can explain itself, debug itself, and (in v2.5) modify itself with formal guarantees about what it changed.

---

## Unlearn (cross-floor)

### What unlearn is

A first-class, cascading, non-destructive removal operation. Constitution articles XII and XVI.

### Targets

```rust
pub enum UnlearnTarget {
    Episode(EpisodeId),
    Belief(BeliefId),
    Concept(ConceptId),
    BySource(String),       // "GDPR request from user X"
    BySession(SessionId),   // "this entire conversation"
    Pattern(QueryPattern),  // "anything matching these criteria"
}
```

### Process

```
1. IDENTIFY CASCADE
   Find all episodes, beliefs, semantic atoms, procedures referencing
   the target. Compute the dependency graph.

2. TOMBSTONE (non-destructive)
   Mark each affected row with t_tombstoned = now.
   Signatures invalidated in mmap.
   Concept HV marked withdrawn.
   Inverted-index entries removed.

3. CASCADE
   Beliefs whose evidence drops below threshold → confidence reduced
   or belief withdrawn. Affected semantic atoms recomputed without
   removed evidence. Procedures with broken trigger chains marked degraded.

4. SELF-VECTOR SUBTRACTION (v2)
   self_vector ← self_vector - α * bundle(tombstoned_signatures)
   The self-model no longer contains contamination from the unlearned data.

5. AUDIT (permanent)
   Emit LearningEvent::Unlearned with:
   - target_ref
   - cascade_size: episodes, beliefs, atoms, procedures affected
   - self_vector_drift: hamming distance before/after
   - reason
   - dependency_graph_id

6. TOMBSTONE EXPIRY
   Default 30 days. After expiry, signatures.dat compacted, tombstoned
   rows physically removed. BUT the LearningEvent::Unlearned record
   is permanent — the *fact that data was removed* survives compaction.
```

### API

```rust
impl Agidb {
    pub async fn unlearn(
        &self,
        target: UnlearnTarget,
        reason: String
    ) -> Result<UnlearnReport>;

    pub async fn unlearn_report(&self, audit_id: AuditId) -> Result<UnlearnReport>;
    pub async fn unlearn_history(
        &self,
        since: DateTime<Utc>
    ) -> Result<Vec<UnlearnEvent>>;
    pub async fn restore_within_window(&self, audit_id: AuditId) -> Result<RestoreReport>;
}

pub struct UnlearnReport {
    pub episodes_removed: usize,
    pub beliefs_removed: usize,
    pub beliefs_revised: usize,
    pub semantic_atoms_affected: usize,
    pub procedures_affected: usize,
    pub signatures_invalidated: usize,
    pub self_vector_drift_hamming: u32,
    pub audit_log_entry: LearningEventId,
    pub tombstone_expiry: DateTime<Utc>,
}
```

### Why first-class

GDPR Article 17 (right to erasure), HIPAA, financial-data retention laws all require deletion semantics that vanilla DELETE doesn't provide:
- The deletion must cascade through derived data (beliefs derived from the unlearned episode).
- The fact of deletion must be auditable (even after the data itself is gone).
- The system's internal state (self-vector) must not retain the deleted data.

mem0, letta, zep, cognee all ship `delete()` as DELETE FROM. None handle cascading, none preserve audit, none subtract from the self-model. agidb is the first agent memory layer to take these requirements seriously from v0.1.

### Why self-vector subtraction matters

Without step 4, here's what happens: an enterprise customer asks "do you still remember information X?" The agent runs a recall query → no results returned. Customer is satisfied. But the self-vector (used in goal-biased retrieval, surprise gating, etc.) still contains a contribution from the unlearned signatures. Internal agent behavior is still subtly biased by what was supposed to be forgotten.

This is the difference between *hiding* data and *forgetting* data. agidb forgets.

---

## How the primitives compose

### Example: a complete agent loop

```rust
let db = Agidb::open("./memory.agidb").await?;

// floor 6 — goal
let goal = db.set_goal(Goal::new("find a thai place for the team dinner")).await?;

// floor 1 — sensory + extraction + promotion (v2.0 text-only)
db.observe_sensory(SensoryFrame::text("sarah recommended bawri")).await?;

// floor 1 — sensory + extraction + promotion (v2.1 multimodal)
db.observe_multimodal(
    Some(video_clip),
    Some(audio_clip),
    Some("sarah said bawri at dinner".into()),
    ObserveContext::default(),
).await?;

// floor 6 — belief
db.assert_belief(
    Belief::new("Bawri is a thai restaurant")
        .with_confidence(0.9)
        .with_evidence_from("sarah's recommendation")
).await?;

// floor 3 → 4 — consolidation
db.consolidate().await?;

// floor 1-7 — goal-biased recall
let result = db.recall("what thai place was mentioned?").await?;
// Goal-biased: "find a thai place" is active, so thai-related matches are up-weighted.
// Returns: Bawri (0.94), with provenance back to Sarah's observation.

// floor 7 — introspection
let log = db.what_did_i_learn(since_yesterday()).await?;
// Returns: [GoalSet, MultimodalEpisodeStored, BeliefAsserted, SemanticAtomFormed, ...]

// later: unlearn
db.unlearn(UnlearnTarget::BySession(dinner_session), "user requested forget".into()).await?;
// Cascading removal + self-vector subtraction + permanent audit.
```

### The big picture

The five primitives turn agidb from a memory database into a cognitive substrate. Each primitive could exist standalone, but their composition is what makes the substrate AGI-grade:

- **Goals** make retrieval intentional. The agent attends to what it wants.
- **Beliefs** make state revisable. The agent updates its worldview when evidence changes.
- **Sensory frames** make perception selective. The agent filters noise.
- **Self-model** makes the agent introspectable. The agent can reason about itself.
- **Unlearn** makes forgetting principled. The agent can be trusted to forget when required.

Together, they implement the minimum credible substrate for autonomous AI agents.

In v2.1, multimodal sensory adds video+audio+text as first-class inputs, brain-calibrated surprise empirically grounds the sensory threshold, and the BAMS benchmark validates the substrate against human cortical ground truth. The cognitive primitives become brain-aligned cognitive primitives. That's the v2.1 wedge.
