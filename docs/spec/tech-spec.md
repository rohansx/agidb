# sochdb — technical specification

this document specifies the rust API, the core types, error handling, performance targets, and dependencies for sochdb v0.1.

audience: contributors and integrators. assumes familiarity with rust, async, and embedded databases.

## crate organization

```
sochdb (workspace)
├── sochdb-core         # the engine: HDC kernel, redb, mmap, recall
├── sochdb-extract      # GLiNER ONNX wrapper, triple extraction
├── sochdb-cli          # the `sochdb` binary
├── sochdb-mcp          # MCP server
├── sochdb-py           # pyo3 python bindings
└── sochdb-bench        # benchmark harness against Mem0, Zep, Letta
```

users typically depend on `sochdb` (the umbrella crate that re-exports the public API of `sochdb-core` and `sochdb-extract`).

## the public API

### opening a database

```rust
use sochdb::{Sochdb, Config};

let db = Sochdb::open("./memory.soch").await?;

// or with config
let db = Sochdb::builder()
    .path("./memory.soch")
    .signature_dim(8192)
    .extractor(Extractor::GLiNER)
    .consolidation_interval(Duration::from_secs(300))
    .strict_mode(false)
    .build()
    .await?;
```

`open` creates the database if it doesn't exist. it's idempotent.

### the `Memory` trait

the core API surface, implemented by `Sochdb`:

```rust
#[async_trait]
pub trait Memory: Send + Sync {
    /// observe a new fact or event. returns an EpisodeId for provenance.
    async fn observe(&self, text: &str, opts: ObserveOpts) -> Result<EpisodeId>;

    /// observe a procedure (workflow / skill). procedural memory.
    async fn observe_procedure(&self, proc: Procedure) -> Result<EpisodeId>;

    /// retrieve memories matching a partial cue. never returns empty.
    /// returns both episodic matches and semantic atoms.
    async fn recall(&self, query: Query) -> Result<Recall>;

    /// retrieve procedures whose trigger matches the cue. procedural memory.
    async fn recall_procedure(&self, cue: &str, k: usize) -> Result<Vec<ProcedureMatch>>;

    /// fetch everything sochdb knows about a concept, optionally as-of a time.
    async fn what_about(&self, subject: &str, as_of: Option<DateTime<Utc>>)
        -> Result<EntityView>;

    /// find paths between two concepts up to N hops.
    async fn between(&self, a: &str, b: &str, max_hops: u8) -> Result<Vec<Path>>;

    /// run the consolidation worker synchronously. returns stats.
    async fn consolidate(&self) -> Result<ConsolidationReport>;

    /// flush in-memory state to disk and release locks.
    async fn close(self) -> Result<()>;
}
```

### `observe()` in detail

```rust
pub struct ObserveOpts {
    pub valid_time: Option<TimeRange>,
    pub provenance: Option<Provenance>,
    pub strict: bool,
}

pub struct Provenance {
    pub source: String,       // "user", "agent", "tool:gmail", etc.
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
    pub metadata: HashMap<String, Value>,
}
```

example:

```rust
let id = db.observe(
    "Sarah recommended Bawri in Bandra last weekend",
    ObserveOpts {
        valid_time: None,           // let the extractor figure it out
        provenance: Some(Provenance {
            source: "user".into(),
            session_id: Some("sess_abc123".into()),
            trace_id: None,
            metadata: hashmap! {},
        }),
        strict: false,
    },
).await?;
```

### `recall()` in detail

```rust
pub struct Query {
    pub cue:             String,
    pub k:               usize,
    pub as_of:           Option<DateTime<Utc>>,
    pub min_confidence:  f32,
    pub include_pending: bool,    // also return strict-mode-filtered triples
    pub tier_floor:      Tier,    // don't fall through below this tier

    // working-memory controls
    pub session_id:      Option<String>,    // scope or boost a session
    pub session_only:    bool,              // if true, restrict to session_id
    pub session_boost:   f32,               // multiplier for in-session results
    pub recency_tau:     Duration,          // half-life for recency factor
}

impl Query {
    pub fn cue(text: &str) -> Self {
        Query {
            cue: text.into(), k: 10, as_of: None,
            min_confidence: 0.0, include_pending: false,
            tier_floor: Tier::NearestNeighbor,
            session_id: None, session_only: false, session_boost: 1.0,
            recency_tau: Duration::from_secs(3600),
        }
    }

    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into()); self
    }

    pub fn session_only(mut self, b: bool) -> Self {
        self.session_only = b; self
    }

    pub fn session_boost(mut self, b: f32) -> Self {
        self.session_boost = b; self
    }
}

pub struct Recall {
    pub matches:        Vec<RecallMatch>,     // episodic
    pub semantic_atoms: Vec<SemanticMatch>,   // consolidated facts
    pub tier_used:      Tier,
    pub elapsed_ms:     u32,
}

pub struct RecallMatch {
    pub episode_id: EpisodeId,
    pub text:       String,
    pub triples:    Vec<Triple>,
    pub confidence: f32,
    pub valid_time: TimeRange,
    pub provenance: Provenance,
    pub superseded: bool,
    pub source_tier: Tier,
}

pub struct SemanticMatch {
    pub atom_id:        SemanticAtomId,
    pub statement:      String,           // canonical form
    pub concept:        ConceptId,
    pub evidence:       Vec<EpisodeId>,   // source episodes (provenance)
    pub evidence_count: u32,
    pub confidence:     f32,
    pub last_referenced: DateTime<Utc>,
}

pub enum Tier {
    Exact,             // tier A: canonical entity match
    Similarity,        // tier B: HDC signature similarity
    Gist,              // tier C: raw-text gist fallback
    NearestNeighbor,   // tier D: low-confidence nearest neighbors
}
```

example:

```rust
let recall = db.recall(Query::cue("what did sarah say about thai food?")).await?;

for m in &recall.matches {
    println!("[{:?} {:.2}] {}", m.source_tier, m.confidence, m.text);
}

println!("used tier {:?} in {}ms", recall.tier_used, recall.elapsed_ms);
```

### `what_about()` in detail

returns a comprehensive view of a concept — all triples where it appears as subject or object, all aliases, all source episodes, with optional bi-temporal filter.

```rust
pub struct EntityView {
    pub canonical_name: String,
    pub aliases: Vec<String>,
    pub entity_type: String,
    pub triples_as_subject: Vec<Triple>,
    pub triples_as_object: Vec<Triple>,
    pub episode_count: u32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub semantic_atom: Option<SemanticAtom>,  // present if consolidated
}
```

### `between()` in detail

finds paths between two concepts via shared triples. uses HDC binding to compose path signatures, so multi-hop queries cost a single signature compute plus a hamming search — no actual graph traversal.

```rust
pub struct Path {
    pub nodes: Vec<ConceptId>,
    pub edges: Vec<Triple>,
    pub total_confidence: f32,
}
```

### `observe_procedure()` and `recall_procedure()` — procedural memory

procedural memory in sochdb is a typed episode shape representing a workflow, skill, or routine the agent has learned. it answers the question *"how do i do X?"* the way episodic memory answers *"when did i do X?"* and semantic memory answers *"what is X?"*

```rust
pub struct Procedure {
    pub name:           String,           // canonical handle
    pub description:    String,           // human-readable summary
    pub trigger:        String,           // when to invoke (natural language)
    pub preconditions:  Vec<String>,      // what must be true before
    pub steps:          Vec<ProcedureStep>,
    pub postconditions: Vec<String>,      // what should be true after
    pub provenance:     Option<Provenance>,
}

pub struct ProcedureStep {
    pub description: String,
    pub tool:        Option<String>,      // tool to call, if applicable
    pub args:        Option<Value>,
}

pub struct ProcedureMatch {
    pub episode_id:    EpisodeId,
    pub procedure:     Procedure,
    pub confidence:    f32,
    pub success_count: u32,
    pub failure_count: u32,
    pub last_invoked:  Option<DateTime<Utc>>,
}
```

example:

```rust
db.observe_procedure(Procedure {
    name: "deploy_to_staging".into(),
    description: "deploy current branch to the staging environment".into(),
    trigger: "when the user wants to deploy to staging".into(),
    preconditions: vec![
        "current branch passes tests".into(),
        "user has staging credentials".into(),
    ],
    steps: vec![
        ProcedureStep {
            description: "verify tests pass".into(),
            tool: Some("run_tests".into()),
            args: None,
        },
        ProcedureStep {
            description: "run the deploy script".into(),
            tool: Some("shell".into()),
            args: Some(json!({"cmd": "./deploy.sh staging"})),
        },
    ],
    postconditions: vec!["staging URL is reachable".into()],
    provenance: None,
}).await?;
```

at recall time:

```rust
let procs = db.recall_procedure("how do i deploy to staging?", 3).await?;
for p in procs {
    println!("[{:.2}] {} ({} succ / {} fail)",
             p.confidence, p.procedure.name,
             p.success_count, p.failure_count);
}
```

**what's deferred to v0.2:** specialized retrieval (matching procedures to current situations by precondition-checking), procedure composition, procedure execution (sochdb stores procedures but doesn't run them — that's the agent framework's job), and skill abstraction (learning new procedures from observed successful sequences).

### `consolidate()` in detail

```rust
pub struct ConsolidationReport {
    pub episodes_scanned: u32,
    pub semantic_atoms_created: u32,
    pub contradictions_detected: u32,
    pub atoms_decayed: u32,
    pub bytes_reclaimed: u64,
    pub elapsed_ms: u32,
}
```

normally consolidation runs as a background tokio task on the schedule configured at open time. calling `consolidate()` explicitly forces a synchronous run — useful for tests, debugging, or scripted maintenance.

## error handling

sochdb uses `anyhow::Result` for the public API and `thiserror`-typed errors internally:

```rust
#[derive(thiserror::Error, Debug)]
pub enum SochError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("database error: {0}")]
    Database(#[from] redb::Error),

    #[error("extraction failed: {0}")]
    Extraction(String),

    #[error("signature corruption at offset {0}")]
    CorruptSignature(u64),

    #[error("invalid query: {0}")]
    InvalidQuery(String),

    #[error("concept not found: {0}")]
    UnknownConcept(String),

    #[error("internal: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, SochError>;
```

errors are always actionable. no swallowed errors. no panics in the public API.

## the HDC kernel

the heart of layer 1. minimal, no external dependencies.

```rust
// sochdb-core/src/hdc.rs

pub const D: usize = 8192;
pub const D_BYTES: usize = D / 8;   // 1024

/// a binary hypervector. fixed size 8192 bits / 1024 bytes.
#[derive(Clone, Copy)]
#[repr(C, align(64))]
pub struct HV(pub [u8; D_BYTES]);

impl HV {
    /// deterministic random HV from a hash of a name.
    pub fn from_name(name: &str) -> Self { /* ... */ }

    /// XOR-based binding. (self ⊗ other)
    pub fn bind(&self, other: &HV) -> HV {
        let mut out = [0u8; D_BYTES];
        for i in 0..D_BYTES {
            out[i] = self.0[i] ^ other.0[i];
        }
        HV(out)
    }

    /// majority-bundle of a set of HVs. (a ⊕ b ⊕ c ...)
    pub fn bundle(hvs: &[HV]) -> HV { /* per-bit majority */ }

    /// hamming distance via POPCOUNT. uses AVX-512 when available.
    pub fn hamming(&self, other: &HV) -> u32 {
        // unsafe { popcount_avx512(self.0.as_ptr(), other.0.as_ptr()) }
        // with portable fallback
        unimplemented!()
    }

    pub fn similarity(&self, other: &HV) -> f32 {
        1.0 - (self.hamming(other) as f32 / D as f32)
    }

    /// active dimensions (indices where bit is set). used for indexing.
    pub fn active_dims(&self) -> impl Iterator<Item = u32> + '_ {
        self.0.iter().enumerate().flat_map(|(byte_idx, &b)| {
            (0..8).filter(move |bit| (b >> bit) & 1 == 1)
                  .map(move |bit| (byte_idx * 8 + bit) as u32)
        })
    }
}
```

POPCOUNT uses `std::arch::x86_64::_mm512_popcnt_epi64` when `target_feature = "avx512vpopcntdq"`, with a portable fallback using `u64::count_ones()` over 128 chunks.

## performance targets

| metric | v0.1 target | mechanism |
|---|---|---|
| `observe` p50 | ≤ 100ms | GLiNER bottleneck, ~150ms uncached → cached embedding via ONNX session reuse |
| `observe` p95 | ≤ 200ms | as above + index update tail |
| `recall` p50 | ≤ 20ms | inverted-index filter + POPCOUNT scan |
| `recall` p95 | ≤ 50ms | as above + tier fall-through worst case |
| `recall` p99 | ≤ 100ms | tier D nearest-neighbor scan |
| `what_about` p95 | ≤ 30ms | indexed lookup |
| `between` (3-hop) p95 | ≤ 80ms | HDC path binding + scan |
| `consolidate` (10k episodes) | ≤ 5s | background, low priority |
| open cold | ≤ 100ms | redb header + mmap init |
| binary size | ≤ 60 MB | rust-stripped, no LLM weights |
| memory footprint (idle) | ≤ 80 MB | mmap doesn't count toward RSS |
| memory footprint (loaded 1M episodes) | ≤ 200 MB | working set; full file mmap'd |

these are measured on a benchmark laptop: Apple M2 (or Intel i7-12700H), 16 GB RAM, NVMe SSD. AVX-512 path used when available; on M-series Macs we use NEON POPCOUNT.

### benchmark reporting contract

sochdb publishes **all six metrics** on every public benchmark — never a single number. this is non-negotiable per [constitution](./constitution.md) article 10.

| metric | what it measures | why we publish it |
|---|---|---|
| **BLEU** | surface-form n-gram overlap with reference answer | conservative lower bound on correctness |
| **F1** | token overlap with reference answer | industry standard, comparable to Mem0/Zep/Letta numbers |
| **LLM-judge (binary)** | semantic correctness, judged by a held-out LLM | catches paraphrased correct answers F1 misses |
| **token cost** | total prompt + completion tokens spent per query | dollars-per-recall comparability against Mem0's ~7k baseline |
| **p95 latency** | end-to-end recall latency including any network calls | the user-facing number |
| **noisy-cue degradation** | accuracy when 20% of cue tokens are corrupted | tests graceful tier-C/D fallback (sochdb's bet) |

every published metric ships with: the harness commit hash, baseline system versions (pinned in `bench/lockfile.toml`), the judge model used for LLM-judge, and the raw per-query logs. judge models are held out — sochdb does not tune against any model used as a judge.

three benchmark suites are run on every release:

| benchmark | what it tests | source |
|---|---|---|
| **LongMemEval-S** | long-context memory accuracy on episodic recall | Wu et al., 2024 |
| **LoCoMo** | long conversation memory across 10+ sessions | Maharana et al., 2024 |
| **BEAM** | scale to millions of tokens; contradiction resolution; instruction following | Mem0, 2026 |

## dependencies

minimal, deliberate, all rust:

| crate | purpose | why this one |
|---|---|---|
| `tokio` | async runtime | de facto rust async standard |
| `redb` | embedded ACID KV | pure rust, ACID, MVCC |
| `memmap2` | mmap | safe rust mmap |
| `croaring` | roaring bitmaps | rust bindings, mature |
| `ort` | ONNX runtime | for GLiNER |
| `tokenizers` | HF tokenizers | for GLiNER input |
| `chrono` | dates and times | bi-temporal stamps |
| `serde` + `bincode` | serialization | for redb values |
| `anyhow` + `thiserror` | error handling | rust convention |
| `tracing` | structured logging | rust convention |
| `clap` | CLI parsing | for `sochdb` binary |
| `pyo3` | python bindings | for `sochdb-py` |

we explicitly avoid:
- LLM SDKs (openai, anthropic) in `sochdb-core` — only in `sochdb-extract` behind a feature flag
- C/C++ dependencies where a rust equivalent exists
- async-std and other tokio competitors

## the CLI

```bash
sochdb open ./memory.soch
sochdb observe ./memory.soch "Sarah recommended Bawri"
sochdb recall ./memory.soch "what thai place"
sochdb what-about ./memory.soch "Sarah"
sochdb consolidate ./memory.soch
sochdb stats ./memory.soch
sochdb export ./memory.soch --format jsonl > backup.jsonl
sochdb import ./memory.soch < backup.jsonl
```

the CLI is for debugging and ops. typical users embed sochdb in their app.

## the MCP server

`sochdb-mcp` exposes the API as MCP tools so any MCP-compatible agent (Claude Desktop, Cursor, Claude Code, OpenAI MCP clients) can use sochdb as a memory layer:

```jsonc
// tools exposed
{
  "name": "memory_observe",
  "description": "Store a new memory in sochdb.",
  "input_schema": {
    "type": "object",
    "properties": {
      "text": { "type": "string" },
      "valid_time": { "type": "string", "format": "date-time" }
    },
    "required": ["text"]
  }
}

{
  "name": "memory_recall",
  "description": "Recall memories matching a partial cue.",
  "input_schema": {
    "type": "object",
    "properties": {
      "cue": { "type": "string" },
      "k": { "type": "integer", "default": 10 },
      "as_of": { "type": "string", "format": "date-time" }
    },
    "required": ["cue"]
  }
}

{ "name": "memory_what_about", "..." }
{ "name": "memory_between", "..." }
{ "name": "memory_consolidate", "..." }
```

running:

```bash
sochdb-mcp --db ./memory.soch --transport stdio
```

claude desktop's mcp config:

```json
{
  "mcpServers": {
    "sochdb": {
      "command": "sochdb-mcp",
      "args": ["--db", "/Users/rohan/memory.soch"]
    }
  }
}
```

this is the primary distribution path. an agent gets memory by installing a binary, not by writing glue code.

## python bindings

`sochdb-py` via pyo3:

```python
import sochdb
import asyncio

async def main():
    db = await sochdb.open("./memory.soch")

    await db.observe("Sarah recommended Bawri in Bandra last weekend")

    recall = await db.recall("what thai place")
    for m in recall.matches:
        print(f"[{m.confidence:.2f}] {m.text}")

asyncio.run(main())
```

the python wrapper is thin — types map directly, async is preserved, errors translate to python exceptions. `pip install sochdb` ships pre-built wheels for linux/macOS/windows on x86_64 and aarch64.

## versioning and compatibility

semver:
- `0.x.y`: breaking changes allowed between minor versions; sochdb is pre-1.0
- `1.0.0`: API stability commitment; on-disk format stability

on-disk format versioning: `manifest.toml` includes `format_version`. each release supports reading the previous N format versions and ships a migration path. data is never silently rewritten — migration is explicit (`sochdb migrate <path>`).

## testing strategy

three layers of testing:

1. **unit tests** in each crate. `cargo test`. covers HDC kernel correctness, redb schema migrations, GLiNER wrapper, MCP server tool definitions.
2. **property-based tests** via `proptest`. covers HDC algebra (binding inverse, bundling membership), confidence monotonicity, supersession invariants.
3. **benchmark harness** in `sochdb-bench`. runs **LongMemEval-S, LoCoMo, and BEAM** against sochdb, Mem0, Zep/Graphiti, and Letta. publishes the full six-metric stack (BLEU + F1 + LLM-judge + token cost + p95 latency + noisy-cue degradation) with raw logs per the benchmark reporting contract above.

CI runs unit + property tests on every PR. the benchmark harness is gated to nightly to keep PR CI fast; full releases must re-run the harness with pinned baseline versions and publish the raw logs alongside the release notes.

## what's *not* in v0.1

explicit non-goals:

- distributed mode
- multi-tenancy
- a hosted cloud tier (the API is designed to support one later)
- a query language
- a UI
- multimodal storage
- transactions across multiple `observe()` calls
- compression of signatures
- encryption at rest
- replication / WAL streaming

see [ROADMAP.md](../product/roadmap.md) for what's deferred to v0.2, v0.3, v1.0.

## next reads

- [ARCHITECTURE.md](../architecture/architecture.md) — system-level architecture
- [LAYER_1_RECALL.md](../architecture/layer-1-recall.md) — HDC math
- [ROADMAP.md](../product/roadmap.md) — milestones and timeline
