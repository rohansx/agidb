# agidb — Technical Specification

> The full Rust API. Types, traits, methods, error model, performance
> targets. The reference document for anyone implementing against agidb,
> embedding it, or contributing to the engine. Covers v2.0 + v2.1.

**Status:** v2.0 substrate API stabilizing at month 9. v2.1 multimodal API stabilizing at month 12. Pre-1.0 — APIs are not yet semver-stable.

## Overview

```rust
use agidb::{Agidb, Query, Goal, Belief, ObserveContext};
use agidb::sensory::{VideoClip, AudioClip};  // v2.1

#[tokio::main]
async fn main() -> agidb::Result<()> {
    let db = Agidb::open("./memory.agidb").await?;

    // v2.0 — text observation
    db.observe("Sarah recommended Bawri", ObserveContext::default()).await?;

    // v2.1 — multimodal observation
    db.observe_multimodal(
        Some(VideoClip::from_file("dinner.mp4")?),
        Some(AudioClip::from_file("dinner.wav")?),
        Some("sarah recommended Bawri".into()),
        ObserveContext::default(),
    ).await?;

    // v2.0 — first-class goals
    let goal = db.set_goal(Goal::new("find a thai place for the team dinner")).await?;

    // v2.0 — first-class beliefs
    db.assert_belief(
        Belief::new("Sarah likes thai food").with_confidence(0.8)
    ).await?;

    // v2.0 — unified recall (goal-biased automatically)
    let recall = db.recall(Query::cue("what thai place did sarah mention?")).await?;

    // v2.1 — brain-aligned memory similarity score
    let bams = db.bams_self_score().await?;

    Ok(())
}
```

## Public crate structure

```
agidb (umbrella) — re-exports the public API
├── agidb-core         the engine: HDC, redb, mmap, recall, consolidation,
│                      goals, beliefs, sensory, self-model, unlearn
├── agidb-extract      GLiNER ONNX wrapper, triple + belief extraction
├── agidb-sensory      v2.1: V-JEPA 2 + Wav2Vec-BERT + Llama-3.2-3B encoders,
│                            HDC projection, multimodal binding
├── agidb-ns           neurosymbolic translation layer
├── agidb-skills       procedural execution traces, skill runtime
├── agidb-cli          the `agidb` binary
├── agidb-mcp          MCP server
├── agidb-py           pyo3 Python bindings
├── agidb-bench        benchmark harness (LongMemEval/LoCoMo/BEAM + cognitive)
└── agidb-bams         v2.1: BAMS benchmark suite, six-network RSA, baselines
```

MSRV: Rust 1.89. License: Apache-2.0 (core), CC BY-NC for BAMS artifacts (TRIBE v2 weights).

## Core types

### Identifiers

```rust
pub struct EpisodeId(pub u64);
pub struct ConceptId(pub u64);
pub struct AtomId(pub u64);
pub struct GoalId(pub u64);
pub struct BeliefId(pub u64);
pub struct ProcedureId(pub u64);
pub struct SensoryId(pub u64);
pub struct LearningEventId(pub u64);
pub struct AuditId(pub u64);
pub struct RecallId(pub u64);
pub struct SelfVectorSnapshotId(pub u64);  // v2.1
pub struct SessionId(pub Uuid);
```

### Hypervector

```rust
#[repr(align(64))]
pub struct HV {
    pub bits: [u64; 128],   // 8192 bits = 1024 bytes
}

impl HV {
    pub fn zero() -> Self;
    pub fn from_seed(seed: &[u8; 32]) -> Self;
    pub fn from_name(name: &str) -> Self;
    pub fn random<R: Rng>(rng: &mut R) -> Self;
    pub fn bind(&self, other: &HV) -> HV;       // XOR
    pub fn hamming(&self, other: &HV) -> u32;    // POPCOUNT
    pub fn similarity(&self, other: &HV) -> f32; // 1 - hamming/8192
    pub fn active_dims(&self) -> u32;
    pub fn set_bit(&mut self, idx: u32);
    pub fn clear_bit(&mut self, idx: u32);
    pub fn get_bit(&self, idx: u32) -> bool;
    pub fn set_bits(&self) -> impl Iterator<Item = u32>;
    pub fn subtract(&self, other: &HV, alpha: f32) -> HV;  // v2: for self-vector
}

pub fn bundle(hvs: &[HV]) -> HV;   // per-bit majority
pub fn bind_pair(a: &HV, b: &HV) -> HV;
```

### Domain types (inherited)

```rust
pub struct Episode {
    pub id: EpisodeId,
    pub text: String,
    pub triples: Vec<Triple>,
    pub signature: HV,
    pub gist_signature: HV,
    pub provenance: Provenance,
    pub confidence: f32,
    pub t_valid_start: DateTime<Utc>,
    pub t_valid_end: Option<DateTime<Utc>>,
    pub t_tx_start: DateTime<Utc>,
    pub t_tx_end: Option<DateTime<Utc>>,
    pub superseded_by: Option<EpisodeId>,
    pub tombstoned_at: Option<DateTime<Utc>>,
    pub session_id: Option<SessionId>,
    // v2.1
    pub modalities: Vec<Modality>,
    pub modality_signatures: Option<MultimodalSignatures>,
}

pub struct Triple {
    pub subject: ConceptId,
    pub predicate: String,
    pub object: Value,
    pub confidence: f32,
    pub source_episode: Option<EpisodeId>,
}

pub enum Value {
    Concept(ConceptId),
    Text(String),
    Number(f64),
    Date(DateTime<Utc>),
}

pub struct Concept {
    pub id: ConceptId,
    pub canonical_name: String,
    pub aliases: Vec<String>,
    pub concept_type: ConceptType,
    pub signature: HV,
    pub created_at: DateTime<Utc>,
    pub withdrawn_at: Option<DateTime<Utc>>,
}

pub struct SemanticAtom {
    pub id: AtomId,
    pub subject: ConceptId,
    pub predicate: String,
    pub object: Value,
    pub signature: HV,
    pub evidence_count: u32,
    pub source_episodes: Vec<EpisodeId>,
    pub t_valid_start: DateTime<Utc>,
    pub t_valid_end: Option<DateTime<Utc>>,
    pub confidence: f32,
    pub last_referenced: DateTime<Utc>,
}

pub struct Provenance {
    pub source: String,
    pub session_id: Option<SessionId>,
    pub trace_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}
```

### v2.0 cognitive types

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

pub struct Belief {
    pub id: BeliefId,
    pub claim: String,
    pub subject: ConceptId,
    pub predicate: String,
    pub object: Value,
    pub confidence: f32,
    pub evidence: Vec<EpisodeId>,
    pub contradictions: Vec<EpisodeId>,
    pub revision_log: Vec<BeliefRevision>,
    pub signature: HV,
    pub t_valid_start: DateTime<Utc>,
    pub t_valid_end: Option<DateTime<Utc>>,
    pub t_tx_start: DateTime<Utc>,
    pub t_tx_end: Option<DateTime<Utc>>,
    pub provenance: Provenance,
    pub withdrawn_at: Option<DateTime<Utc>>,
}

pub struct BeliefRevision {
    pub timestamp: DateTime<Utc>,
    pub previous_confidence: f32,
    pub new_confidence: f32,
    pub triggering_evidence: Option<EpisodeId>,
    pub reason: String,
    pub llm_used: bool,
    pub llm_model: Option<String>,
}

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
    Multimodal { components: Vec<Modality> },  // v2.1
}

pub enum SensoryData {
    InlineText(String),
    BlobRef(PathBuf),
    InlineLatent { encoder: EncoderId, latent: Vec<f32> },  // v2.1
}

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

pub enum UnlearnTarget {
    Episode(EpisodeId),
    Belief(BeliefId),
    Concept(ConceptId),
    BySource(String),
    BySession(SessionId),
    Pattern(QueryPattern),
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

pub struct AttentionTrace {
    pub id: RecallId,
    pub query: Query,
    pub candidates: Vec<AttentionCandidate>,
    pub goal_signature: Option<HV>,
    pub recency_window: Duration,
    pub timestamp: DateTime<Utc>,
}

pub struct AttentionCandidate {
    pub episode_id: EpisodeId,
    pub similarity: f32,
    pub goal_bias: f32,
    pub recency_boost: f32,
    pub final_confidence: f32,
    pub retained: bool,
    pub rejection_reason: Option<String>,
}
```

### v2.1 multimodal types

```rust
pub struct MultimodalSignatures {
    pub video: Option<HV>,
    pub audio: Option<HV>,
    pub text: Option<HV>,
}

pub struct EncoderConfig {
    pub role: EncoderRole,
    pub version: String,
    pub weight_sha: String,
    pub projection_seed: u64,
    pub d_input: usize,
    pub d_output: usize,
    pub huggingface_url: String,
    pub registered_at: DateTime<Utc>,
}

pub enum EncoderRole {
    VJepa2,
    Wav2VecBert,
    LlamaText,
    Gliner,
}

pub struct BrainCalibration {
    pub theta_brain: f32,
    pub fitted_at: DateTime<Utc>,
    pub calibration_dataset: String,    // e.g. "courtois-neuromod-subject-1"
    pub tribe_version: String,           // e.g. "v2-march-2026"
    pub tribe_weights_sha: String,
    pub neural_threshold_sigma: f32,
    pub pearson_correlation: f32,
}

pub struct BamsScore {
    pub overall: f32,
    pub per_network: HashMap<CorticalNetwork, f32>,
    pub per_movie: HashMap<String, HashMap<CorticalNetwork, f32>>,
    pub tribe_version: String,
    pub agidb_version: String,
    pub computed_at: DateTime<Utc>,
}

pub enum CorticalNetwork {
    DefaultMode,
    Visual,
    Auditory,
    Language,
    DorsalAttention,
    Frontoparietal,
}

pub struct VideoClip {
    pub frames: Vec<Frame>,
    pub fps: f32,
    pub duration_ms: u32,
}

pub struct AudioClip {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub duration_ms: u32,
}

pub struct MultimodalFrame {
    pub video: Option<VideoClip>,
    pub audio: Option<AudioClip>,
    pub text: Option<String>,
    pub received_at: DateTime<Utc>,
}

pub enum SelfVectorTrigger {
    Consolidation,
    Unlearn,
    Manual,
}

pub struct SelfVectorSnapshot {
    pub id: SelfVectorSnapshotId,
    pub taken_at: DateTime<Utc>,
    pub signature: HV,
    pub drift_from_previous: u32,
    pub trigger: SelfVectorTrigger,
}
```

### Query and recall types

```rust
pub struct Query {
    pub cue_text: Option<String>,
    pub entity_name: Option<String>,
    pub extracted_triples: Vec<Triple>,
    pub session_id: Option<SessionId>,
    pub valid_as_of: Option<DateTime<Utc>>,
    pub transaction_as_of: Option<DateTime<Utc>>,
    pub tier_floor: Tier,
    pub k: usize,
    pub min_confidence: f32,
    pub trace_attention: bool,
    pub goal_bias_weight: f32,
    pub recency_window: Duration,
}

impl Query {
    pub fn cue(text: impl Into<String>) -> Self;
    pub fn entity(name: impl Into<String>) -> Self;
    pub fn with_session(self, id: SessionId) -> Self;
    pub fn as_of(self, valid: DateTime<Utc>) -> Self;
    pub fn transaction_at(self, tx: DateTime<Utc>) -> Self;
    pub fn tier_floor(self, tier: Tier) -> Self;
    pub fn k(self, k: usize) -> Self;
    pub fn min_confidence(self, c: f32) -> Self;
    pub fn trace_attention(self, trace: bool) -> Self;
    pub fn with_goal_bias(self, weight: f32) -> Self;
}

pub enum Tier {
    Exact,
    Similarity,
    Gist,
    NearestNeighbor,
}

pub struct Recall {
    pub matches: Vec<RecallMatch>,
    pub semantic_atoms: Vec<SemanticMatch>,
    pub beliefs: Vec<BeliefMatch>,
    pub active_goals: Vec<GoalId>,
    pub tier_used: Tier,
    pub elapsed_ms: u32,
    pub attention_trace: Option<AttentionTrace>,
}

pub struct RecallMatch {
    pub episode_id: EpisodeId,
    pub text: String,
    pub confidence: f32,
    pub tier: Tier,
    pub low_confidence: bool,
    pub goal_biased: bool,
    pub provenance: Provenance,
    pub modalities: Vec<Modality>,  // v2.1
}

pub struct SemanticMatch {
    pub atom_id: AtomId,
    pub atom: SemanticAtom,
    pub confidence: f32,
}

pub struct BeliefMatch {
    pub belief_id: BeliefId,
    pub belief: Belief,
    pub confidence: f32,
}
```

## The main API

```rust
pub struct Agidb { /* private */ }

impl Agidb {
    // Lifecycle
    pub async fn open(path: impl AsRef<Path>) -> Result<Self>;
    pub async fn create(path: impl AsRef<Path>, config: AgidbConfig) -> Result<Self>;
    pub async fn close(self) -> Result<()>;

    // Floor 1 — sensory
    pub async fn observe_sensory(&self, frame: SensoryFrame) -> Result<SensoryId>;
    pub async fn working_state(&self) -> Result<SensoryBuffer>;
    pub async fn surprise_score(&self, frame: &SensoryFrame) -> Result<f32>;

    // Floor 3 — episodic (v2.0 text)
    pub async fn observe(&self, text: impl Into<String>, ctx: ObserveContext) -> Result<EpisodeId>;
    pub async fn get_episode(&self, id: EpisodeId) -> Result<Option<Episode>>;
    pub async fn supersede(&self, old: EpisodeId, new: EpisodeId) -> Result<()>;
    pub async fn between(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Episode>>;

    // Floor 1 + 3 — multimodal (v2.1)
    pub async fn observe_multimodal(
        &self,
        video: Option<VideoClip>,
        audio: Option<AudioClip>,
        text: Option<String>,
        ctx: ObserveContext,
    ) -> Result<EpisodeId>;
    pub async fn surprise_score_brain_calibrated(&self, frame: &MultimodalFrame) -> Result<f32>;
    pub async fn extract_modality_signature(
        &self,
        episode_id: EpisodeId,
        modality: Modality,
    ) -> Result<Option<HV>>;

    // Floor 1-7 — unified recall (goal-biased automatically)
    pub async fn recall(&self, query: impl Into<Query>) -> Result<Recall>;

    // Floor 4 — semantic
    pub async fn consolidate(&self) -> Result<ConsolidationReport>;
    pub async fn what_about(&self, concept: ConceptId) -> Result<Vec<SemanticMatch>>;

    // Floor 5 — procedural
    pub async fn observe_procedure(&self, p: Procedure) -> Result<ProcedureId>;
    pub async fn record_execution(&self, p: ProcedureId, trace: ExecutionTrace) -> Result<()>;
    pub async fn procedure_stats(&self, p: ProcedureId) -> Result<ProcedureStats>;
    pub async fn recall_procedure(&self, query: Query) -> Result<Vec<Procedure>>;

    // Floor 6 — goals
    pub async fn set_goal(&self, goal: Goal) -> Result<GoalId>;
    pub async fn revise_goal(&self, id: GoalId, patch: GoalPatch) -> Result<()>;
    pub async fn complete_goal(&self, id: GoalId, evidence: Vec<EpisodeId>) -> Result<()>;
    pub async fn abandon_goal(&self, id: GoalId, reason: String) -> Result<()>;
    pub async fn active_goals(&self) -> Result<Vec<Goal>>;
    pub async fn goal_tree(&self, root: GoalId) -> Result<GoalTree>;
    pub async fn get_goal(&self, id: GoalId) -> Result<Option<Goal>>;

    // Floor 6 — beliefs
    pub async fn assert_belief(&self, belief: Belief) -> Result<BeliefId>;
    pub async fn revise_belief(&self, id: BeliefId, new_evidence: EpisodeId) -> Result<RevisionReport>;
    pub async fn what_do_i_believe(&self, about: ConceptId) -> Result<Vec<Belief>>;
    pub async fn belief_history(&self, id: BeliefId) -> Result<Vec<BeliefRevision>>;
    pub async fn withdraw_belief(&self, id: BeliefId, reason: String) -> Result<()>;

    // Floor 7 — self-model
    pub async fn what_did_i_learn(&self, since: DateTime<Utc>) -> Result<Vec<LearningEvent>>;
    pub async fn attention_trace(&self, recall_id: RecallId) -> Result<Option<AttentionTrace>>;
    pub async fn self_vector(&self) -> Result<HV>;
    pub async fn self_vector_at(&self, time: DateTime<Utc>) -> Result<HV>;
    pub async fn self_vector_history(&self, since: DateTime<Utc>) -> Result<Vec<SelfVectorSnapshot>>;
    pub async fn introspect(&self, q: IntrospectionQuery) -> Result<IntrospectionResult>;

    // Cross-floor — unlearn
    pub async fn unlearn(&self, target: UnlearnTarget, reason: String) -> Result<UnlearnReport>;
    pub async fn unlearn_report(&self, audit_id: AuditId) -> Result<UnlearnReport>;
    pub async fn unlearn_history(&self, since: DateTime<Utc>) -> Result<Vec<UnlearnEvent>>;
    pub async fn restore_within_window(&self, audit_id: AuditId) -> Result<RestoreReport>;

    // Neurosymbolic
    pub async fn neurosymbolic_query(&self, q: NeurosymbolicQuery) -> Result<Recall>;
    pub async fn signature_to_triples(&self, sig: &HV) -> Result<Vec<Triple>>;
    pub async fn triples_to_signature(&self, triples: &[Triple]) -> Result<HV>;

    // v2.1 — brain alignment
    pub async fn brain_calibration(&self) -> Result<BrainCalibration>;
    pub async fn recalibrate(&self, dataset: CalibrationDataset) -> Result<BrainCalibration>;
    pub async fn bams_self_score(&self) -> Result<BamsScore>;
    pub async fn encoder_versions(&self) -> Result<HashMap<EncoderRole, EncoderConfig>>;

    // Maintenance
    pub async fn compact(&self) -> Result<CompactReport>;
    pub async fn export_jsonl(&self, path: impl AsRef<Path>) -> Result<()>;
    pub async fn import_jsonl(&self, path: impl AsRef<Path>) -> Result<ImportReport>;
    pub async fn migrate_encoders(&self, new_config: EncoderConfig) -> Result<()>;  // v2.1
}
```

## Encoder trait abstraction (v2.1)

```rust
pub trait Encoder: Send + Sync {
    type Input;
    fn role(&self) -> EncoderRole;
    fn config(&self) -> &EncoderConfig;
    fn encode(&self, input: &Self::Input) -> Result<Vec<f32>>;
}

pub trait MultimodalEncoder: Encoder {
    fn encode_and_project(&self, input: &Self::Input) -> Result<HV>;
}

pub struct VJepa2Encoder { /* private */ }
pub struct Wav2VecBertEncoder { /* private */ }
pub struct LlamaTextEncoder { /* private */ }

impl Encoder for VJepa2Encoder {
    type Input = VideoClip;
    fn role(&self) -> EncoderRole { EncoderRole::VJepa2 }
    fn config(&self) -> &EncoderConfig;
    fn encode(&self, clip: &VideoClip) -> Result<Vec<f32>>;
}

impl MultimodalEncoder for VJepa2Encoder {
    fn encode_and_project(&self, clip: &VideoClip) -> Result<HV>;
}
```

Two reference implementations per encoder: ONNX-backed (default) and Candle-backed (optional pure-Rust path).

## Error model

```rust
#[derive(Debug, thiserror::Error)]
pub enum AgidbError {
    #[error("storage error: {0}")] Storage(#[from] redb::Error),
    #[error("io error: {0}")] Io(#[from] std::io::Error),
    #[error("extraction error: {0}")] Extraction(String),
    #[error("encoder error: {0}")] Encoder(String),
    #[error("encoder version mismatch: db={0}, binary={1}")]
    EncoderVersionMismatch(String, String),
    #[error("not found: {0}")] NotFound(String),
    #[error("invalid input: {0}")] InvalidInput(String),
    #[error("schema mismatch: db={0}, binary={1}")] SchemaMismatch(String, String),
    #[error("LLM error: {0}")] Llm(String),
    #[error("brain calibration error: {0}")] Calibration(String),
    #[error("BAMS error: {0}")] Bams(String),
    #[error("other: {0}")] Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, AgidbError>;
```

## Performance targets

### v2.0 substrate

| Metric | Target | Inherited or new |
|---|---|---|
| `recall` p50 / p95 / p99 | ≤ 20ms / ≤ 50ms / ≤ 100ms | inherited |
| `observe` (text) p50 / p95 | ≤ 100ms / ≤ 200ms | inherited (GLiNER-bound) |
| 8192-bit hamming scan over 100k signatures | ≤ 5ms portable / ≤ 1.5ms AVX-512 | inherited |
| `consolidate` (10k episodes) | ≤ 5s | inherited |
| `set_goal` / `revise_goal` / `assert_belief` | ≤ 5ms | new |
| `unlearn` cascade (1000-episode concept) | ≤ 100ms | new |
| `what_did_i_learn` (last 7 days) | ≤ 50ms | new |
| `self_vector` snapshot | ≤ 5ms | new |
| Binary size | ≤ 80 MB | inherited |
| Memory footprint (1M episodes loaded) | ≤ 250 MB | inherited |

### v2.1 brain-aligned additions

| Metric | Target (CPU) | Target (GPU) | Notes |
|---|---|---|---|
| `observe_multimodal` (30s video+audio clip) p50 | ≤ 2s | ≤ 500ms | V-JEPA dominates |
| V-JEPA 2 inference (64 frames, 256×256) p50 | ≤ 1.5s | ≤ 200ms | M2 ANE / RTX 4090 |
| Wav2Vec-BERT inference (60s audio) p50 | ≤ 400ms | ≤ 80ms | |
| Llama-3.2-3B encoder (1024 tokens) p50 | ≤ 200ms | ≤ 30ms | |
| Charikar 2002 projection (1024d → 8192-bit) | ≤ 1ms | n/a | SIMD-friendly |
| Multimodal binding (3 modalities → 1 HV) | ≤ 200µs | n/a | |
| Brain-calibrated surprise score | ≤ 500µs | n/a | |
| `extract_modality_signature` | ≤ 1ms | n/a | XOR + nearest-neighbor cleanup |
| BAMS single-movie evaluation | ≤ 30s | ≤ 5s | per-system per-movie |
| BAMS full suite (6 movies × 7 systems × 6 networks) | ≤ 8h | ≤ 1h | parallelizable |
| Binary size (with encoders bundled) | ≤ 4 GB | | |
| Binary size (weights on-demand download) | ≤ 100 MB | | preferred default |

## Configuration

```rust
pub struct AgidbConfig {
    pub max_episode_signatures: usize,            // default 10_000_000
    pub consolidation_interval: Duration,         // default 5min
    pub consolidation_min_evidence: u32,          // default 3
    pub belief_promotion_threshold: u32,          // default 5
    pub similarity_threshold_tier_b: f32,         // default 0.6
    pub similarity_threshold_tier_c: f32,         // default 0.3
    pub similarity_threshold_tier_d: f32,         // default 0.0
    pub default_goal_bias_weight: f32,            // default 0.3
    pub sensory_buffer_capacity: usize,           // default 1000
    pub sensory_buffer_duration: Duration,        // default 60s
    pub surprise_threshold: f32,                  // default 0.4 (v2.0) or brain-calibrated (v2.1)
    pub tombstone_retention: Duration,            // default 30 days
    pub self_vector_alpha: f32,                   // default 0.05
    pub enable_signed_audit: bool,                // default false; v0.3+
    pub backend: EncoderBackend,                  // default Onnx; v2.1
    pub gpu_acceleration: GpuConfig,              // v2.1
    pub brain_calibration: Option<BrainCalibration>, // v2.1; None until calibrated
}

pub enum EncoderBackend {
    Onnx,
    Candle,
}

pub struct GpuConfig {
    pub use_metal: bool,   // Apple
    pub use_cuda: bool,    // NVIDIA
    pub fallback_to_cpu: bool,
}
```

## BAMS API (agidb-bams crate, v2.1)

```rust
pub struct BamsBenchmark { /* private */ }

impl BamsBenchmark {
    pub async fn new(config: BamsConfig) -> Result<Self>;
    pub async fn run_full_suite(&self) -> Result<BamsReport>;
    pub async fn run_single_movie(&self, movie: &str, system: &dyn AgentMemorySystem) -> Result<BamsScore>;
    pub async fn compute_baselines(&self) -> Result<HashMap<String, BamsScore>>;
}

pub trait AgentMemorySystem: Send + Sync {
    fn name(&self) -> &str;
    async fn replay_stimulus(&mut self, stream: &StimulusStream) -> Result<Vec<HV>>;
}

pub struct BamsConfig {
    pub movies: Vec<String>,
    pub networks: Vec<CorticalNetwork>,
    pub tribe_weights_path: PathBuf,
    pub stimulus_dataset_root: PathBuf,
    pub output_path: PathBuf,
}

pub struct BamsReport {
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub systems: HashMap<String, BamsScore>,
    pub reproduction: ReproductionInfo,
}
```

## Python bindings (agidb-py)

`pip install agidb`. All async methods exposed as async Python via pyo3-asyncio.

```python
import asyncio
import agidb

async def main():
    db = await agidb.Agidb.open("./memory.agidb")

    # v2.0 — text observe
    await db.observe("Sarah recommended Bawri", ctx=agidb.ObserveContext())

    # v2.1 — multimodal observe
    await db.observe_multimodal(
        video=agidb.VideoClip.from_file("dinner.mp4"),
        audio=agidb.AudioClip.from_file("dinner.wav"),
        text="sarah recommended Bawri",
    )

    # Goals + beliefs
    goal = await db.set_goal(agidb.Goal(description="find a thai place"))
    await db.assert_belief(agidb.Belief(
        claim="Sarah likes thai food", confidence=0.8
    ))

    # Recall
    result = await db.recall(agidb.Query.cue("what thai place did sarah mention?"))
    for m in result.matches:
        print(f"[{m.confidence:.2f}] {m.text}")

    # v2.1 — BAMS self-score
    bams = await db.bams_self_score()
    print(f"BAMS overall: {bams.overall:.3f}")

asyncio.run(main())
```

## MCP server (agidb-mcp)

Exposes the public API as MCP tools. Connect from Claude Desktop:

```json
{
  "mcpServers": {
    "agidb": {
      "command": "agidb-mcp",
      "args": ["--db", "/path/to/memory.agidb"]
    }
  }
}
```

Tools exposed: `observe`, `observe_multimodal` (v2.1), `recall`, `set_goal`, `revise_goal`, `assert_belief`, `revise_belief`, `unlearn`, `consolidate`, `what_did_i_learn`, `what_do_i_believe`, `active_goals`, `between`, `bams_self_score` (v2.1).

## Stability commitments

| API surface | Stability |
|---|---|
| HDC kernel (HV, bind, bundle, hamming) | stable since sochdb v1; no breaks expected |
| Storage layout | format_version pinned in manifest; migrations explicit |
| Encoder versions | pinned in manifest; mismatch errors |
| Public Agidb API | stabilizing, semver from 1.0 (post-v2.0 launch) |
| Internal traits / type bounds | may evolve; pre-1.0 |
| BAMS protocol | v2.1 is "BAMS v1"; future BAMS v2 may differ |
| Brain calibration | v2.1 against TRIBE v2 march 2026; recalibration required for TRIBE v3 |

## What this spec doesn't cover

- Detailed consolidation algorithm (see architecture.md)
- Detailed BAMS protocol (see bams-benchmark.md)
- Detailed brain-alignment derivation (see brain-alignment.md)
- Per-floor cognitive semantics (see cognitive-primitives.md)
- HDC math justification (see layer-1-recall.md)
- Encoder selection rationale (see layer-2-extraction.md)
- Storage schema rationale (see layer-3-storage.md)
- 5-year evolution plan (see agi-trajectory.md)

This document is the API surface reference. The other docs are the why.
