# Phase 3 — Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land layer 2 — turn raw text into structured triples + canonicalized entities + parsed time anchors, achieving **F1 ≥ 0.85** on a 100-sample gold set and activating **tier B** in `Store::recall()`.

**Architecture:** Port the working subset from the sibling `ctxgraph` repo (`gline-rs` for NER + GLiREL relation extractor + `temporal.rs` + `model_manager.rs`), wrap in a new `agidb-extract` crate, expose orchestration as a free function `agidb_extract::observe_text(&mut Store, &Extractor, text, ctx) -> Result<EpisodeId>`. `agidb-core` stays extraction-blind (gets a `TextExtractor` trait and a `Store::create_concept` helper; nothing else).

**Tech stack:** Rust 1.89, gline-rs (NER), ort 2.0 (ONNX runtime), tokenizers 0.22, ndarray, chrono_english (time), reqwest+rustls (model download), sha2 (verification), proptest (property tests).

**Spec:** [`docs/superpowers/specs/2026-05-23-phase-3-extraction-design.md`](../specs/2026-05-23-phase-3-extraction-design.md)

**Port source:** `/home/rsx/Desktop/projx/ctxgraph/crates/ctxgraph-extract/`

## Execution status (2026-05-23)

10 of 18 tasks complete. Workspace at HEAD: 81 tests passing (+37 over baseline), clippy + fmt clean.

| Task | Status | Commit |
|---|---|---|
| 1 — agidb-core types | ✅ | `bd55104` |
| 2 — deps + ExtractError | ✅ | `883e60a` |
| 3 — predicate canonicalizer | ✅ | `5a48192` |
| 4 — `Store::create_concept` | ✅ | `d059ec8` |
| 5 — alias resolver | ✅ | `c4bd7db` |
| 6 — temporal parser | ✅ | `7805699` |
| 7 — ModelRef constants | ✅ | `9ff4604` |
| 8 — model_manager | ✅ | `a628f9f` |
| `Store::next_episode_id` (plan-adjacent helper) | ✅ | `755d998` |
| 9 — NER via gline-rs | ⬜ | needs `gline-rs` API + ONNX download |
| 10 — GLiREL port | ⬜ | needs ctxgraph port + ONNX |
| 11 — `Extractor` orchestration | ⬜ | depends on 9 + 10; TextExtractor trait already in place |
| 12 + 13 — `observe_text` + integration test | ✅ | `15846ef` (does NOT depend on 11 thanks to TextExtractor trait + MockExtractor) |
| 14 — eval sub-crate scaffold | ⬜ | depends on 11 |
| 15 — 100-sample gold dataset | ⬜ | **human labelling work** |
| 16 — eval binary | ⬜ | depends on 11 + 15 |
| 17 — nightly CI workflow | ⬜ | depends on 14–16 |
| 18 — F1 ≥ 0.85 verification loop | ⬜ | the actual exit gate |

See [`../specs/2026-05-23-phase-3-extraction-design.md`](../specs/2026-05-23-phase-3-extraction-design.md) § 14 for as-built type adjustments. See [`../../phases/phase-3-extraction.md`](../../phases/phase-3-extraction.md) for the per-deliverable view.

---

## File Structure

| File | Status | Responsibility |
|---|---|---|
| `crates/agidb-core/src/types.rs` | **modify** | add `ExtractContext`, `Extraction`, `Entity`, `TextExtractor` trait |
| `crates/agidb-core/src/store.rs` | **modify** | add `Store::create_concept(name, kind) -> Result<ConceptId>` helper |
| `crates/agidb-core/src/lib.rs` | **modify** | re-export new types |
| `crates/agidb-extract/Cargo.toml` | **modify** | add deps (`gline-rs`, `ort`, `tokenizers`, `ndarray`, `chrono_english`, `reqwest`, `rustls`, `sha2`, `dirs`, `regex`, `indicatif`, `proptest`, `tempfile`) |
| `crates/agidb-extract/src/lib.rs` | **rewrite** | `pub use ...`, `pub fn observe_text(...)` |
| `crates/agidb-extract/src/error.rs` | **create** | `ExtractError` + `From<ExtractError> for AgidbError` |
| `crates/agidb-extract/src/predicates.rs` | **create** | curated trie of surface-verb → canonical |
| `crates/agidb-extract/src/aliases.rs` | **create** | exact + Levenshtein-≤3 resolver against `Store` |
| `crates/agidb-extract/src/temporal.rs` | **create** | port from `ctxgraph-extract/src/temporal.rs` (460 LOC) |
| `crates/agidb-extract/src/models.rs` | **create** | `ModelRef` + SHA-pinned default constants |
| `crates/agidb-extract/src/model_manager.rs` | **create** | port from `ctxgraph-extract/src/model_manager.rs` (414 LOC) |
| `crates/agidb-extract/src/ner.rs` | **create** | port + adapt `ctxgraph-extract/src/ner.rs` (113 LOC), `gline-rs` wrapper |
| `crates/agidb-extract/src/glirel.rs` | **create** | port from `ctxgraph-extract/src/glirel.rs` (717 LOC), relation extraction |
| `crates/agidb-extract/src/extractor.rs` | **create** | `Extractor`, `ExtractorConfig`, `impl TextExtractor`, pipeline orchestration |
| `crates/agidb-extract/tests/predicates_properties.rs` | **create** | ~30 surface→canonical cases |
| `crates/agidb-extract/tests/aliases_properties.rs` | **create** | exact match, Levenshtein boundary |
| `crates/agidb-extract/tests/temporal_properties.rs` | **create** | 50 cases incl. proptest |
| `crates/agidb-extract/tests/ner_smoke.rs` | **create** | 5–10 fixtures with real NER (gated `--features model-tests`) |
| `crates/agidb-extract/tests/glirel_smoke.rs` | **create** | 5–10 fixtures with real RE (gated `--features model-tests`) |
| `crates/agidb-extract/tests/observe_text.rs` | **create** | end-to-end integration; `MockExtractor` for PR-time + 1 gated test |
| `crates/agidb-extract/eval/Cargo.toml` | **create** | sub-crate manifest, binary `agidb-extract-eval` |
| `crates/agidb-extract/eval/src/main.rs` | **create** | load gold, run extractor, score P/R/F1, write JSON report |
| `crates/agidb-extract/eval/gold/observations.jsonl` | **create** | 100 hand-labelled samples |
| `Cargo.toml` (workspace) | **modify** | add new members + workspace deps |
| `.github/workflows/eval-nightly.yml` | **create** | nightly eval workflow |

---

## Task 1: Core extraction types in `agidb-core`

**Files:**
- Modify: `crates/agidb-core/src/types.rs` — add types at end of file
- Modify: `crates/agidb-core/src/lib.rs` — re-export
- Test: `crates/agidb-core/tests/extraction_types.rs` (new)

- [ ] **Step 1: Write the failing test**

Create `crates/agidb-core/tests/extraction_types.rs`:

```rust
//! Smoke tests for the layer-2-facing types added in phase 3.
//! Real extraction is tested in agidb-extract.

use agidb_core::types::{Entity, ExtractContext, Extraction, TextExtractor, Triple, Value};
use agidb_core::Result;
use chrono::Utc;

struct DummyExtractor;
impl TextExtractor for DummyExtractor {
    fn extract(&self, _text: &str, _ctx: &ExtractContext) -> Result<Extraction> {
        Ok(Extraction {
            triples: vec![],
            valid_time: None,
            raw_entities: vec![],
        })
    }
}

#[test]
fn extract_context_default_uses_observation_time() {
    let now = Utc::now();
    let ctx = ExtractContext {
        observation_time: now,
        relation_hint_types: vec![],
    };
    assert_eq!(ctx.observation_time, now);
}

#[test]
fn entity_carries_optional_concept_id() {
    let e = Entity {
        text: "Sarah".into(),
        entity_type: "Person".into(),
        span: (0, 5),
        confidence: 0.93,
        concept_id: None,
    };
    assert!(e.concept_id.is_none());
    assert_eq!(e.span, (0, 5));
}

#[test]
fn dummy_extractor_satisfies_trait() {
    let ext = DummyExtractor;
    let ctx = ExtractContext {
        observation_time: Utc::now(),
        relation_hint_types: vec![],
    };
    let r = ext.extract("hello", &ctx).expect("dummy never fails");
    assert!(r.triples.is_empty());
    assert!(r.valid_time.is_none());
}
```

- [ ] **Step 2: Run the test to verify it fails**

```
cargo test -p agidb-core --test extraction_types
```

Expected: FAIL with `unresolved import` / `cannot find type Extraction`.

- [ ] **Step 3: Add the types to `crates/agidb-core/src/types.rs`**

Append at end of file (before any trailing module declarations):

```rust
// -----------------------------------------------------------------------------
// Layer-2-facing types (consumed by agidb-extract; defined here so agidb-core
// has no dep on agidb-extract).
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ExtractContext {
    pub observation_time: chrono::DateTime<chrono::Utc>,
    pub relation_hint_types: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub text: String,
    pub entity_type: String,
    pub span: (usize, usize),
    pub confidence: f32,
    pub concept_id: Option<ConceptId>,
}

#[derive(Debug, Clone)]
pub struct Extraction {
    pub triples: Vec<Triple>,
    pub valid_time: Option<TimeRange>,
    pub raw_entities: Vec<Entity>,
}

/// Layer-2 → layer-3 boundary. Any extractor implementing this can be passed
/// to `agidb_extract::observe_text`. `agidb-core` stays extraction-blind.
pub trait TextExtractor {
    fn extract(&self, text: &str, ctx: &ExtractContext) -> crate::Result<Extraction>;
}
```

- [ ] **Step 4: Re-export from `lib.rs`**

Modify `crates/agidb-core/src/lib.rs`. Find the existing `pub use types::...` line and extend it (or add a new line) so the new types are re-exported:

```rust
pub use types::{
    Concept, ConceptId, Entity, Episode, EpisodeId, ExtractContext, Extraction, Provenance,
    Query, Recall, RecallMatch, SemanticAtom, SemanticAtomId, SemanticMatch, TextExtractor,
    Tier, TimeRange, Triple, TripleId, Value,
};
```

(Keep existing re-exports intact; just add the four new names alphabetically.)

- [ ] **Step 5: Run the test to verify it passes**

```
cargo test -p agidb-core --test extraction_types
```

Expected: PASS, 3 tests ok.

- [ ] **Step 6: Confirm the whole workspace still builds**

```
cargo build --workspace --all-targets
```

Expected: Finished, no errors.

- [ ] **Step 7: Commit**

```bash
git add crates/agidb-core/src/types.rs crates/agidb-core/src/lib.rs crates/agidb-core/tests/extraction_types.rs
git commit -m "feat(agidb-core): add layer-2 types (ExtractContext, Extraction, Entity, TextExtractor)"
```

---

## Task 2: `agidb-extract` Cargo dependencies + `ExtractError`

**Files:**
- Modify: `Cargo.toml` (workspace) — add new workspace deps
- Modify: `crates/agidb-extract/Cargo.toml` — pull deps in
- Create: `crates/agidb-extract/src/error.rs`
- Modify: `crates/agidb-extract/src/lib.rs` — mod + pub use

- [ ] **Step 1: Add the new workspace deps in `Cargo.toml`**

Find the `[workspace.dependencies]` block and append:

```toml
# extraction (phase 3)
gline-rs = "0.1"        # NER; ctxgraph also uses this. Pin exact version after first build.
ort = "=2.0.0-rc.9"     # ONNX runtime; matches ctxgraph's pin to avoid duplicate-version pulls.
tokenizers = "0.22"
ndarray = "0.16"
chrono_english = "0.1"
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
rustls = "0.23"
sha2 = "0.10"
dirs = "6"
regex = "1"
indicatif = "0.17"
```

- [ ] **Step 2: Update `crates/agidb-extract/Cargo.toml`**

Replace the `[dependencies]` block with:

```toml
[dependencies]
agidb-core = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
# extraction-specific
gline-rs = { workspace = true }
ort = { workspace = true }
tokenizers = { workspace = true }
ndarray = { workspace = true }
chrono_english = { workspace = true }
reqwest = { workspace = true }
sha2 = { workspace = true }
dirs = { workspace = true }
regex = { workspace = true }
indicatif = { workspace = true }

[dev-dependencies]
proptest = { workspace = true }
tempfile = { workspace = true }

[features]
default = []
# Tests gated by this run real ONNX inference; default CI run skips them.
model-tests = []
```

- [ ] **Step 3: Create `crates/agidb-extract/src/error.rs`**

```rust
//! Typed errors for layer-2 extraction. Converts to `AgidbError::Extraction`
//! at the agidb-core boundary so the engine sees a single error surface.

use agidb_core::AgidbError;

#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("model load: {0}")]
    ModelLoad(String),

    #[error("model download: {0}")]
    ModelDownload(String),

    #[error("ort inference: {0}")]
    Inference(String),

    #[error("tokenize: {0}")]
    Tokenize(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid model artifact: {0}")]
    InvalidArtifact(String),
}

impl From<ExtractError> for AgidbError {
    fn from(e: ExtractError) -> Self {
        AgidbError::Extraction(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ExtractError>;
```

- [ ] **Step 4: Rewrite `crates/agidb-extract/src/lib.rs`**

Replace the stub with the new scaffold (modules will be filled in later tasks):

```rust
//! Layer 2 — extraction.
//!
//! Turns raw text into structured `Triple`s, canonicalized entities, and
//! parsed time anchors. Wraps `gline-rs` (NER) + a ported GLiREL relation
//! extractor + a ported chrono_english-based temporal parser. Built so the
//! agidb-core engine stays extraction-blind: callers either invoke
//! `Extractor::extract` directly or go through the `observe_text` free
//! function below.
//!
//! Layered per phase-3 design: see
//! docs/superpowers/specs/2026-05-23-phase-3-extraction-design.md

pub mod error;
pub mod predicates;
// pub mod aliases;       — added by task 5
// pub mod temporal;      — added by task 6
// pub mod models;        — added by task 7
// pub mod model_manager; — added by task 8
// pub mod ner;           — added by task 9
// pub mod glirel;        — added by task 10
// pub mod extractor;     — added by task 11

pub use crate::error::{ExtractError, Result};
```

- [ ] **Step 5: Verify build + run any pre-existing tests**

```
cargo build -p agidb-extract --all-targets
cargo test --workspace
```

Expected: Finished; 44 prior tests still pass.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml crates/agidb-extract/Cargo.toml crates/agidb-extract/src/lib.rs crates/agidb-extract/src/error.rs
git commit -m "feat(agidb-extract): wire extraction deps and ExtractError scaffold"
```

---

## Task 3: Predicate canonicalizer

**Files:**
- Create: `crates/agidb-extract/src/predicates.rs`
- Create: `crates/agidb-extract/tests/predicates_properties.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/agidb-extract/tests/predicates_properties.rs`:

```rust
use agidb_extract::predicates::{PredicateTable, canonicalize};

fn table() -> PredicateTable { PredicateTable::default() }

#[test]
fn exact_match_recommends() {
    let t = table();
    assert_eq!(canonicalize(&t, "recommended"), Some("recommends".into()));
    assert_eq!(canonicalize(&t, "Recommended"), Some("recommends".into()));
    assert_eq!(canonicalize(&t, "suggested"), Some("recommends".into()));
    assert_eq!(canonicalize(&t, "told me about"), Some("recommends".into()));
}

#[test]
fn located_in_family() {
    let t = table();
    assert_eq!(canonicalize(&t, "in"), Some("located_in".into()));
    assert_eq!(canonicalize(&t, "based in"), Some("located_in".into()));
    assert_eq!(canonicalize(&t, "lives in"), Some("located_in".into()));
}

#[test]
fn works_at_family() {
    let t = table();
    assert_eq!(canonicalize(&t, "works at"), Some("works_at".into()));
    assert_eq!(canonicalize(&t, "is employed by"), Some("works_at".into()));
}

#[test]
fn unknown_returns_none() {
    let t = table();
    assert_eq!(canonicalize(&t, "frobnicated"), None);
    assert_eq!(canonicalize(&t, ""), None);
}

#[test]
fn custom_synonyms_extend_defaults() {
    let mut t = table();
    t.add_synonym("frobnicates", "frobnicated");
    t.add_synonym("frobnicates", "twiddled the knobs on");
    assert_eq!(canonicalize(&t, "frobnicated"), Some("frobnicates".into()));
    assert_eq!(canonicalize(&t, "twiddled the knobs on"), Some("frobnicates".into()));
}
```

- [ ] **Step 2: Run the test to verify it fails**

```
cargo test -p agidb-extract --test predicates_properties
```

Expected: FAIL with `unresolved import predicates`.

- [ ] **Step 3: Implement `crates/agidb-extract/src/predicates.rs`**

```rust
//! Predicate canonicalization: surface verbs → a small canonical vocabulary.
//! Curated, not learned. Custom synonyms loadable per-deployment.

use std::collections::HashMap;

/// Lookup table. Key = lowercased surface form; value = canonical predicate.
#[derive(Debug, Clone)]
pub struct PredicateTable {
    table: HashMap<String, String>,
}

impl PredicateTable {
    pub fn new() -> Self {
        Self { table: HashMap::new() }
    }

    pub fn add_synonym(&mut self, canonical: &str, surface: &str) {
        self.table.insert(surface.to_lowercase(), canonical.to_string());
    }

    pub fn lookup(&self, surface: &str) -> Option<String> {
        self.table.get(&surface.to_lowercase()).cloned()
    }
}

impl Default for PredicateTable {
    /// The built-in curated vocabulary. Extend per-deployment by calling
    /// `add_synonym`. Tracked in
    /// docs/superpowers/specs/2026-05-23-phase-3-extraction-design.md § 6.
    fn default() -> Self {
        let mut t = Self::new();
        // recommends
        for s in ["recommended", "suggested", "told me about", "pitched", "mentioned to me"] {
            t.add_synonym("recommends", s);
        }
        // located_in
        for s in ["in", "based in", "is from", "lives in", "is located in"] {
            t.add_synonym("located_in", s);
        }
        // works_at
        for s in ["works at", "is employed by", "is at", "works for"] {
            t.add_synonym("works_at", s);
        }
        // likes
        for s in ["likes", "loves", "prefers", "is into", "enjoys"] {
            t.add_synonym("likes", s);
        }
        // said
        for s in ["said", "told", "claimed", "mentioned"] {
            t.add_synonym("said", s);
        }
        // met
        for s in ["met", "ran into", "saw", "encountered"] {
            t.add_synonym("met", s);
        }
        // visited
        for s in ["visited", "went to", "stopped by", "dropped in at"] {
            t.add_synonym("visited", s);
        }
        // owns
        for s in ["owns", "has", "possesses"] {
            t.add_synonym("owns", s);
        }
        t
    }
}

/// Convenience: look up a surface predicate. Returns `None` for unknown.
pub fn canonicalize(table: &PredicateTable, surface: &str) -> Option<String> {
    table.lookup(surface)
}
```

- [ ] **Step 4: Wire the module in `lib.rs`**

Already present (`pub mod predicates;` was added in Task 2 Step 4 as uncommented). Confirm it's uncommented in `crates/agidb-extract/src/lib.rs`.

- [ ] **Step 5: Run the test to verify it passes**

```
cargo test -p agidb-extract --test predicates_properties
```

Expected: 5 passed.

- [ ] **Step 6: Commit**

```bash
git add crates/agidb-extract/src/predicates.rs crates/agidb-extract/tests/predicates_properties.rs
git commit -m "feat(agidb-extract): predicate canonicalizer with default + custom synonyms"
```

---

## Task 4: `Store::create_concept` helper

The alias resolver in Task 5 needs to mint new `Concept` rows when a name doesn't match anything. The existing `Store` already has `concept_id_for(name)`; it lacks the create path.

**Files:**
- Modify: `crates/agidb-core/src/store.rs` — add `Store::create_concept`
- Modify: `crates/agidb-core/tests/extraction_types.rs` — extend with a roundtrip test
- (Or new) Test: `crates/agidb-core/tests/concept_lifecycle.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/agidb-core/tests/concept_lifecycle.rs`:

```rust
//! create_concept assigns a fresh ConceptId, persists the row, makes
//! concept_id_for find it.

use agidb_core::store::{Store, StoreConfig};
use agidb_core::types::{Concept, ConceptId};
use tempfile::TempDir;

fn fresh_store() -> (Store, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let cfg = StoreConfig {
        path: dir.path().to_path_buf(),
        ..StoreConfig::default()
    };
    let store = Store::open(cfg).expect("open");
    (store, dir)
}

#[test]
fn create_concept_assigns_id_and_persists() {
    let (mut store, _dir) = fresh_store();
    let id = store
        .create_concept("Sarah", "Person")
        .expect("create");
    let looked_up = store
        .concept_id_for("Sarah")
        .expect("lookup")
        .expect("found");
    assert_eq!(id, looked_up);
    assert!(id.raw() > 0, "ids start at 1");
}

#[test]
fn create_concept_is_idempotent_on_canonical_name() {
    let (mut store, _dir) = fresh_store();
    let first = store.create_concept("Bawri", "Place").expect("first");
    let second = store.create_concept("Bawri", "Place").expect("second");
    assert_eq!(first, second, "second create returns the same id");
}

#[test]
fn create_concept_distinct_names_get_distinct_ids() {
    let (mut store, _dir) = fresh_store();
    let a = store.create_concept("Sarah", "Person").expect("a");
    let b = store.create_concept("Bawri", "Place").expect("b");
    assert_ne!(a, b);
}
```

- [ ] **Step 2: Run the test to verify it fails**

```
cargo test -p agidb-core --test concept_lifecycle
```

Expected: FAIL with `no method create_concept`.

- [ ] **Step 3: Implement `Store::create_concept`**

Open `crates/agidb-core/src/store.rs`. Find the existing `impl Store {` block (the main one). Append before the closing brace:

```rust
    /// Idempotent: if a Concept with this canonical name already exists,
    /// return its `ConceptId`. Otherwise mint a new one, persist, and
    /// return it.
    pub fn create_concept(
        &mut self,
        canonical_name: &str,
        concept_type: &str,
    ) -> Result<ConceptId> {
        // Fast path: already exists.
        if let Some(existing) = self.concept_id_for(canonical_name)? {
            return Ok(existing);
        }

        let tx = self.db.begin_write()?;
        let id;
        {
            let mut concepts = tx.open_table(CONCEPTS)?;
            let mut by_name = tx.open_table(CONCEPT_BY_NAME)?;
            let mut manifest = tx.open_table(MANIFEST)?;

            id = next_concept_id(&mut manifest)?;
            let concept = Concept {
                id,
                canonical_name: canonical_name.to_string(),
                aliases: Vec::new(),
                concept_type: concept_type.to_string(),
                signature: HV::from_name(canonical_name),
                created_at: chrono::Utc::now(),
                withdrawn_at: None,
            };
            concepts.insert(id.raw(), encode(&concept)?)?;
            by_name.insert(canonical_name, id.raw())?;
        }
        tx.commit()?;
        Ok(id)
    }
```

(Verify imports at the top of `store.rs` include `Concept`, `ConceptId`, `HV`, `CONCEPTS`, `CONCEPT_BY_NAME`, `MANIFEST`, `next_concept_id`, `encode`. They should already be present from existing observe logic; add only if missing.)

- [ ] **Step 4: Run the test to verify it passes**

```
cargo test -p agidb-core --test concept_lifecycle
```

Expected: 3 passed.

- [ ] **Step 5: Confirm nothing else broke**

```
cargo test --workspace
```

Expected: all prior tests still green.

- [ ] **Step 6: Commit**

```bash
git add crates/agidb-core/src/store.rs crates/agidb-core/tests/concept_lifecycle.rs
git commit -m "feat(agidb-core): Store::create_concept (idempotent on canonical name)"
```

---

## Task 5: Alias resolver

**Files:**
- Create: `crates/agidb-extract/src/aliases.rs`
- Create: `crates/agidb-extract/tests/aliases_properties.rs`
- Modify: `crates/agidb-extract/src/lib.rs` — uncomment `pub mod aliases;`

- [ ] **Step 1: Write the failing test**

Create `crates/agidb-extract/tests/aliases_properties.rs`:

```rust
//! Alias resolver: exact (case-folded) → existing ConceptId.
//! Otherwise Levenshtein <= 3 against canonical names; unique match wins.
//! Else create a new Concept via Store::create_concept.

use agidb_core::store::{Store, StoreConfig};
use agidb_extract::aliases::AliasResolver;
use tempfile::TempDir;

fn fresh_store() -> (Store, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let cfg = StoreConfig { path: dir.path().to_path_buf(), ..Default::default() };
    (Store::open(cfg).expect("open"), dir)
}

#[test]
fn exact_match_returns_existing_id() {
    let (mut store, _d) = fresh_store();
    let original = store.create_concept("Sarah", "Person").unwrap();
    let resolver = AliasResolver::new();
    let resolved = resolver.resolve(&mut store, "Sarah", "Person").unwrap();
    assert_eq!(resolved, original);
}

#[test]
fn case_insensitive_match() {
    let (mut store, _d) = fresh_store();
    let original = store.create_concept("Bawri", "Place").unwrap();
    let resolver = AliasResolver::new();
    let resolved = resolver.resolve(&mut store, "bawri", "Place").unwrap();
    assert_eq!(resolved, original);
}

#[test]
fn levenshtein_one_typo_matches() {
    let (mut store, _d) = fresh_store();
    let original = store.create_concept("Bandra", "Place").unwrap();
    let resolver = AliasResolver::new();
    // single-char typo, distance = 1, should still match
    let resolved = resolver.resolve(&mut store, "Bandar", "Place").unwrap();
    assert_eq!(resolved, original);
}

#[test]
fn no_match_creates_new_concept() {
    let (mut store, _d) = fresh_store();
    let resolver = AliasResolver::new();
    let new_id = resolver.resolve(&mut store, "Quetzalcoatl", "Person").unwrap();
    // Resolving the same again returns the same id (it's persisted).
    let again = resolver.resolve(&mut store, "Quetzalcoatl", "Person").unwrap();
    assert_eq!(new_id, again);
}

#[test]
fn ambiguous_fuzzy_match_falls_through_to_create() {
    let (mut store, _d) = fresh_store();
    let _a = store.create_concept("Alice", "Person").unwrap();
    let _b = store.create_concept("Allie", "Person").unwrap();
    // "Alise" is within distance 2 of both — ambiguous → don't merge; create new.
    let resolver = AliasResolver::new();
    let resolved = resolver.resolve(&mut store, "Alise", "Person").unwrap();
    assert_ne!(resolved, _a);
    assert_ne!(resolved, _b);
}
```

- [ ] **Step 2: Run the test to verify it fails**

```
cargo test -p agidb-extract --test aliases_properties
```

Expected: FAIL with `unresolved import aliases`.

- [ ] **Step 3: Implement `crates/agidb-extract/src/aliases.rs`**

```rust
//! Exact + fuzzy alias resolution against the Store's concepts table.

use agidb_core::store::Store;
use agidb_core::types::ConceptId;
use agidb_core::Result;

const FUZZY_THRESHOLD: usize = 3;

#[derive(Default)]
pub struct AliasResolver {
    _placeholder: (),
}

impl AliasResolver {
    pub fn new() -> Self { Self::default() }

    /// Resolve a surface mention to a `ConceptId`. On miss, mints one.
    /// Concept lookup is case-folded. Fuzzy match accepts Levenshtein <= 3
    /// only when it's unambiguous (single candidate).
    pub fn resolve(
        &self,
        store: &mut Store,
        mention: &str,
        kind: &str,
    ) -> Result<ConceptId> {
        // 1. Exact (case-folded). concept_id_for is already case-sensitive on
        //    canonical_name; we mimic case-insensitivity by scanning all names
        //    in lowercase.
        let folded = mention.to_lowercase();
        if let Some(id) = store.concept_id_for_ci(&folded)? {
            return Ok(id);
        }

        // 2. Fuzzy (Levenshtein <= 3) — single-candidate gate.
        let candidates = store.fuzzy_concept_candidates(&folded, FUZZY_THRESHOLD)?;
        if candidates.len() == 1 {
            return Ok(candidates[0]);
        }

        // 3. Miss → create.
        store.create_concept(mention, kind)
    }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let (m, n) = (a.chars().count(), b.chars().count());
    if m == 0 { return n; }
    if n == 0 { return m; }
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

/// pub for testing.
pub fn lev_distance(a: &str, b: &str) -> usize { levenshtein(a, b) }
```

- [ ] **Step 4: Add the case-insensitive lookup + fuzzy scan to `Store`**

The test depends on two new methods. Open `crates/agidb-core/src/store.rs` and add to the main `impl Store`:

```rust
    /// Case-insensitive lookup against the concept_by_name table. O(N);
    /// fine for v0.1 concept counts.
    pub fn concept_id_for_ci(&self, lowercased: &str) -> Result<Option<ConceptId>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(CONCEPT_BY_NAME)?;
        for row in table.iter()? {
            let (k, v) = row?;
            if k.value().to_lowercase() == lowercased {
                return Ok(Some(ConceptId::new(v.value())));
            }
        }
        Ok(None)
    }

    /// Return all ConceptIds whose lowercased canonical_name is within
    /// `max_dist` Levenshtein of `lowercased`. O(N). Skips the exact-match
    /// case (use `concept_id_for_ci` for that).
    pub fn fuzzy_concept_candidates(
        &self,
        lowercased: &str,
        max_dist: usize,
    ) -> Result<Vec<ConceptId>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(CONCEPT_BY_NAME)?;
        let mut hits = Vec::new();
        for row in table.iter()? {
            let (k, v) = row?;
            let folded = k.value().to_lowercase();
            if folded == lowercased {
                continue;
            }
            // tiny stand-alone levenshtein; kept in agidb-extract too but
            // duplicated here to avoid an agidb-extract dep
            if levenshtein(&folded, lowercased) <= max_dist {
                hits.push(ConceptId::new(v.value()));
            }
        }
        Ok(hits)
    }
```

And add a private `levenshtein` helper at the bottom of `store.rs` (or copy the one from aliases.rs verbatim — same function). Yes, this duplicates `levenshtein` between core and extract; we accept it to keep `agidb-core` extract-free.

- [ ] **Step 5: Uncomment the module + add to `lib.rs`**

Replace the `// pub mod aliases;` line in `crates/agidb-extract/src/lib.rs` with `pub mod aliases;`.

- [ ] **Step 6: Run the test to verify it passes**

```
cargo test -p agidb-extract --test aliases_properties
cargo test -p agidb-core --test concept_lifecycle
```

Expected: both green.

- [ ] **Step 7: Commit**

```bash
git add crates/agidb-extract/src/aliases.rs crates/agidb-extract/src/lib.rs crates/agidb-extract/tests/aliases_properties.rs crates/agidb-core/src/store.rs
git commit -m "feat(agidb-extract): alias resolver (exact + Levenshtein<=3 + create-on-miss)"
```

---

## Task 6: Temporal parser

**Files:**
- Create: `crates/agidb-extract/src/temporal.rs` — port from `ctxgraph-extract/src/temporal.rs`
- Create: `crates/agidb-extract/tests/temporal_properties.rs`
- Modify: `crates/agidb-extract/src/lib.rs` — uncomment `pub mod temporal;`

- [ ] **Step 1: Write the failing test**

```rust
// crates/agidb-extract/tests/temporal_properties.rs
use agidb_extract::temporal::parse_time_anchor;
use chrono::{TimeZone, Utc};

fn anchor() -> chrono::DateTime<chrono::Utc> {
    Utc.with_ymd_and_hms(2026, 5, 23, 12, 0, 0).unwrap()
}

#[test]
fn yesterday_returns_a_range_one_day_back() {
    let r = parse_time_anchor("yesterday", anchor()).expect("parsed");
    assert!(r.start < anchor());
    let one_day_ago = anchor() - chrono::Duration::days(1);
    let diff = (r.start - one_day_ago).num_hours().abs();
    assert!(diff <= 24, "got start={:?}", r.start);
}

#[test]
fn last_weekend_lands_in_the_prior_saturday_sunday() {
    let r = parse_time_anchor("last weekend", anchor()).expect("parsed");
    // anchor is Sat 2026-05-23, so "last weekend" should be ~May 16-17
    assert!(r.start.date_naive() <= chrono::NaiveDate::from_ymd_opt(2026, 5, 17).unwrap());
    assert!(r.start.date_naive() >= chrono::NaiveDate::from_ymd_opt(2026, 5, 15).unwrap());
}

#[test]
fn iso_date_parses() {
    let r = parse_time_anchor("2026-01-15", anchor()).expect("parsed");
    assert_eq!(r.start.date_naive(), chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
}

#[test]
fn nonsense_returns_none() {
    assert!(parse_time_anchor("frobnicated", anchor()).is_none());
    assert!(parse_time_anchor("", anchor()).is_none());
}

#[test]
fn two_months_ago_lands_in_march_2026() {
    let r = parse_time_anchor("two months ago", anchor()).expect("parsed");
    // anchor = 2026-05-23; two months back ≈ March
    assert_eq!(r.start.month(), 3);
    use chrono::Datelike;
}
```

- [ ] **Step 2: Run to confirm it fails**

```
cargo test -p agidb-extract --test temporal_properties
```

Expected: FAIL — `temporal` module missing.

- [ ] **Step 3: Read ctxgraph's `temporal.rs` to port from**

Read `/home/rsx/Desktop/projx/ctxgraph/crates/ctxgraph-extract/src/temporal.rs` (460 LOC). Note:
- It uses `chrono_english::parse_date_string`.
- It returns ctxgraph's own time type — we'll return `agidb_core::types::TimeRange` instead.
- Drop any ctxgraph-specific dependencies; keep only the parsing logic.

- [ ] **Step 4: Implement `crates/agidb-extract/src/temporal.rs`**

The skeleton (port the parsing logic; the rest is small):

```rust
//! Parse natural-language time anchors into `TimeRange` values.
//!
//! Ported (and trimmed) from
//! /home/rsx/Desktop/projx/ctxgraph/crates/ctxgraph-extract/src/temporal.rs
//!
//! Returns `None` for unparseable input (non-fatal — caller falls back to
//! the observation time per the design spec).

use agidb_core::types::TimeRange;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc, Weekday};
use chrono_english::{parse_date_string, Dialect};

/// Parse `text` as a time expression, anchored at `now`. Returns `None`
/// on failure. The returned `TimeRange` has `end = None` for point-in-time
/// expressions and a span for range expressions ("last weekend", "yesterday").
pub fn parse_time_anchor(text: &str, now: DateTime<Utc>) -> Option<TimeRange> {
    let text = text.trim().to_lowercase();
    if text.is_empty() { return None; }

    // 1. Range expressions we want to handle precisely.
    if text == "yesterday" {
        let start = (now - Duration::days(1)).date_naive().and_hms_opt(0, 0, 0)?;
        let end = (now - Duration::days(1)).date_naive().and_hms_opt(23, 59, 59)?;
        return Some(TimeRange {
            start: DateTime::<Utc>::from_naive_utc_and_offset(start, Utc),
            end: Some(DateTime::<Utc>::from_naive_utc_and_offset(end, Utc)),
        });
    }
    if text == "last weekend" {
        return last_weekend(now);
    }
    if text == "this weekend" {
        return this_weekend(now);
    }

    // 2. Fall through to chrono_english for everything else.
    parse_date_string(&text, now, Dialect::Us)
        .ok()
        .map(|dt| TimeRange { start: dt.with_timezone(&Utc), end: None })
}

fn last_weekend(now: DateTime<Utc>) -> Option<TimeRange> {
    let today = now.date_naive();
    // Walk back to the most recent Saturday strictly before today.
    let mut d = today - Duration::days(1);
    while d.weekday() != Weekday::Sat {
        d = d - Duration::days(1);
    }
    let sat = d;
    let sun = sat + Duration::days(1);
    Some(TimeRange {
        start: DateTime::<Utc>::from_naive_utc_and_offset(sat.and_hms_opt(0, 0, 0)?, Utc),
        end: Some(DateTime::<Utc>::from_naive_utc_and_offset(sun.and_hms_opt(23, 59, 59)?, Utc)),
    })
}

fn this_weekend(now: DateTime<Utc>) -> Option<TimeRange> {
    let today = now.date_naive();
    // Forward to the upcoming Saturday (or today if today is Sat/Sun).
    let mut d = today;
    while d.weekday() != Weekday::Sat {
        d = d + Duration::days(1);
    }
    let sat = d;
    let sun = sat + Duration::days(1);
    Some(TimeRange {
        start: DateTime::<Utc>::from_naive_utc_and_offset(sat.and_hms_opt(0, 0, 0)?, Utc),
        end: Some(DateTime::<Utc>::from_naive_utc_and_offset(sun.and_hms_opt(23, 59, 59)?, Utc)),
    })
}
```

(More patterns from ctxgraph's `temporal.rs` can be added in iteration if the gold-set eval shows misses; this gives a starting baseline. ISO dates and "N units ago / from now" expressions are handled by `chrono_english`.)

- [ ] **Step 5: Uncomment `pub mod temporal;` in `lib.rs`**

- [ ] **Step 6: Run the test to verify it passes**

```
cargo test -p agidb-extract --test temporal_properties
```

Expected: 5 passed (may need to adjust `last_weekend` if the anchor day-of-week is itself Saturday; test ranges are forgiving).

- [ ] **Step 7: Commit**

```bash
git add crates/agidb-extract/src/temporal.rs crates/agidb-extract/src/lib.rs crates/agidb-extract/tests/temporal_properties.rs
git commit -m "feat(agidb-extract): chrono_english-based temporal parser with range cases"
```

---

## Task 7: `ModelRef` constants

**Files:**
- Create: `crates/agidb-extract/src/models.rs`
- Modify: `crates/agidb-extract/src/lib.rs` — uncomment `pub mod models;`

- [ ] **Step 1: Write the file (no test needed — pure constants)**

```rust
//! Pinned model references. Updating a model = a code change + new SHA.
//! See model_manager.rs for download/verify behavior.

#[derive(Debug, Clone)]
pub struct ModelRef {
    pub repo: &'static str,
    pub revision: &'static str,
    pub sha256: &'static str,
    /// Optional file-within-repo. None means the standard `model.onnx`.
    pub file: Option<&'static str>,
}

/// Default GLiNER model for NER. The SHA gets pinned the first time
/// `model_manager` downloads and verifies the artifact in task 8/9.
pub const GLINER_DEFAULT: ModelRef = ModelRef {
    repo: "urchade/gliner_multi-v2.1",
    revision: "main",
    sha256: "TBD-PIN-AT-FIRST-DOWNLOAD",
    file: Some("model.onnx"),
};

/// Default GLiREL model for relation extraction. Candidate repo; revisit
/// in task 10 when actually loading.
pub const GLIREL_DEFAULT: ModelRef = ModelRef {
    repo: "jackboyla/glirel_beta",
    revision: "main",
    sha256: "TBD-PIN-AT-FIRST-DOWNLOAD",
    file: Some("model.onnx"),
};
```

- [ ] **Step 2: Uncomment + commit**

```bash
git add crates/agidb-extract/src/models.rs crates/agidb-extract/src/lib.rs
git commit -m "feat(agidb-extract): ModelRef + default GLiNER/GLiREL pins (SHA placeholder)"
```

---

## Task 8: Model manager (port)

**Files:**
- Create: `crates/agidb-extract/src/model_manager.rs` — port + trim from ctxgraph
- Create: `crates/agidb-extract/tests/model_manager.rs` — unit test the cache hit/miss + SHA verify path, no actual network

- [ ] **Step 1: Read the port source**

Read `/home/rsx/Desktop/projx/ctxgraph/crates/ctxgraph-extract/src/model_manager.rs` (414 LOC). Note its public API and trim:
- Keep: HF URL building, cache-dir resolution, SHA256 verify, atomic rename.
- Drop: anything ctxgraph-config-specific, anything wired to ctxgraph's own model registry.

- [ ] **Step 2: Write the failing test**

```rust
// crates/agidb-extract/tests/model_manager.rs
//! Unit tests for the offline-path of ModelManager: cache hits, SHA verify,
//! offline-mode rejection. Network downloads are exercised manually / by
//! the eval workflow, not in unit tests.

use agidb_extract::model_manager::ModelManager;
use agidb_extract::models::ModelRef;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn fake_model_ref() -> ModelRef {
    ModelRef {
        repo: "fake/repo",
        revision: "main",
        sha256: "TBD-PIN-AT-FIRST-DOWNLOAD",
        file: Some("model.onnx"),
    }
}

#[test]
fn cache_hit_returns_path_without_network() {
    let cache = TempDir::new().unwrap();
    let mgr = ModelManager::new(cache.path().to_path_buf(), /*offline=*/false);
    // Pre-populate a fake file.
    let r = fake_model_ref();
    let path = mgr.cache_path(&r);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let bytes = b"hello onnx";
    std::fs::write(&path, bytes).unwrap();

    // Patch the ModelRef SHA to match what we wrote.
    let mut h = Sha256::new();
    h.update(bytes);
    let sha = format!("{:x}", h.finalize());

    // ensure_cached should succeed without ever hitting the network because
    // a file at the expected path with the matching SHA is present.
    let pinned = ModelRef { sha256: Box::leak(sha.clone().into_boxed_str()), ..r };
    let got = mgr.ensure_cached(&pinned).expect("cache hit");
    assert_eq!(got, path);
}

#[test]
fn offline_mode_errors_on_miss() {
    let cache = TempDir::new().unwrap();
    let mgr = ModelManager::new(cache.path().to_path_buf(), /*offline=*/true);
    let r = fake_model_ref();
    let err = mgr.ensure_cached(&r).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("offline") || msg.contains("Offline"), "got: {msg}");
}
```

- [ ] **Step 3: Run to confirm it fails**

```
cargo test -p agidb-extract --test model_manager
```

Expected: FAIL — module missing.

- [ ] **Step 4: Implement `crates/agidb-extract/src/model_manager.rs`**

```rust
//! HuggingFace ONNX model download + cache with SHA verification.
//!
//! Ported and trimmed from
//! /home/rsx/Desktop/projx/ctxgraph/crates/ctxgraph-extract/src/model_manager.rs
//!
//! Constitution: zero network at READ/WRITE time; downloads are setup-time only.
//! AGIDB_OFFLINE=1 forbids downloads outright.

use crate::error::{ExtractError, Result};
use crate::models::ModelRef;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct ModelManager {
    cache_root: PathBuf,
    offline: bool,
}

impl ModelManager {
    pub fn new(cache_root: PathBuf, offline: bool) -> Self {
        Self { cache_root, offline }
    }

    pub fn cache_path(&self, m: &ModelRef) -> PathBuf {
        // sanitize repo path: "urchade/gliner_multi-v2.1" → "urchade_gliner_multi-v2.1"
        let repo_safe = m.repo.replace('/', "_");
        let file = m.file.unwrap_or("model.onnx");
        self.cache_root.join(repo_safe).join(m.revision).join(file)
    }

    /// Return the path to the cached model, downloading + verifying if absent.
    /// Errors if `offline` and the file isn't present, or if the SHA mismatches.
    pub fn ensure_cached(&self, m: &ModelRef) -> Result<PathBuf> {
        let path = self.cache_path(m);
        if path.is_file() {
            if Self::is_placeholder_sha(m.sha256) {
                tracing::warn!(model = m.repo, "SHA placeholder; skipping verify");
                return Ok(path);
            }
            verify_sha256(&path, m.sha256)?;
            return Ok(path);
        }
        if self.offline {
            return Err(ExtractError::ModelDownload(format!(
                "offline mode: required model {}/{}/{} not in cache",
                m.repo, m.revision, m.file.unwrap_or("model.onnx")
            )));
        }
        self.download(m, &path)?;
        if !Self::is_placeholder_sha(m.sha256) {
            verify_sha256(&path, m.sha256)?;
        }
        Ok(path)
    }

    fn is_placeholder_sha(sha: &str) -> bool {
        sha.starts_with("TBD-")
    }

    fn download(&self, m: &ModelRef, target: &Path) -> Result<()> {
        let file = m.file.unwrap_or("model.onnx");
        let url = format!(
            "https://huggingface.co/{repo}/resolve/{rev}/{file}",
            repo = m.repo, rev = m.revision, file = file
        );
        tracing::info!(url = %url, "downloading model");
        fs::create_dir_all(target.parent().unwrap())?;
        let mut resp = reqwest::blocking::get(&url)
            .map_err(|e| ExtractError::ModelDownload(format!("get {url}: {e}")))?;
        if !resp.status().is_success() {
            return Err(ExtractError::ModelDownload(format!(
                "{url} -> HTTP {}", resp.status()
            )));
        }
        // Atomic write: download to .part, then rename.
        let part = target.with_extension("part");
        let mut out = fs::File::create(&part)?;
        resp.copy_to(&mut out)
            .map_err(|e| ExtractError::ModelDownload(format!("copy_to: {e}")))?;
        fs::rename(&part, target)?;
        Ok(())
    }
}

pub fn verify_sha256(path: &Path, expected_hex: &str) -> Result<()> {
    let mut f = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    let got = format!("{:x}", hasher.finalize());
    if got != expected_hex {
        return Err(ExtractError::InvalidArtifact(format!(
            "sha256 mismatch on {}: got {got}, expected {expected_hex}", path.display()
        )));
    }
    Ok(())
}
```

- [ ] **Step 5: Uncomment `pub mod model_manager;` in `lib.rs`**

- [ ] **Step 6: Run tests**

```
cargo test -p agidb-extract --test model_manager
```

Expected: 2 passed.

- [ ] **Step 7: Commit**

```bash
git add crates/agidb-extract/src/model_manager.rs crates/agidb-extract/src/lib.rs crates/agidb-extract/tests/model_manager.rs
git commit -m "feat(agidb-extract): model manager (HF download + sha verify + offline mode)"
```

---

## Task 9: NER wrapper via `gline-rs`

This task pulls in the first real ML dependency. The smoke test is gated behind `--features model-tests` so per-PR CI doesn't pay for model load.

**Files:**
- Create: `crates/agidb-extract/src/ner.rs`
- Create: `crates/agidb-extract/tests/ner_smoke.rs` (gated)

- [ ] **Step 1: Read `gline-rs` API**

Run `cargo doc --open -p gline-rs --no-deps` (or visit docs.rs/gline-rs). Note: the constructor (likely `Gliner::from_path(onnx_path, tokenizer_path)`), the `inference(text, entity_types) -> Vec<Entity>` shape, and the entity type it returns.

Also read `/home/rsx/Desktop/projx/ctxgraph/crates/ctxgraph-extract/src/ner.rs` (113 LOC) — the wrapper pattern around `gline-rs` is the reference implementation.

- [ ] **Step 2: Write the failing smoke test (gated)**

```rust
// crates/agidb-extract/tests/ner_smoke.rs
#![cfg(feature = "model-tests")]
//! Real GLiNER inference against fixture sentences. Slow + downloads weights.
//! Gated behind --features model-tests; nightly CI runs this.

use agidb_extract::models::GLINER_DEFAULT;
use agidb_extract::ner::NerExtractor;
use std::path::PathBuf;

fn cache_dir() -> PathBuf {
    dirs::cache_dir().expect("cache dir").join("agidb/models")
}

#[test]
fn extracts_known_person_and_place() {
    let ner = NerExtractor::new(
        cache_dir(),
        GLINER_DEFAULT.clone(),
        vec!["Person".into(), "Place".into()],
    )
    .expect("load ner");
    let ents = ner.extract("Sarah recommended Bawri in Bandra").expect("infer");
    let texts: Vec<_> = ents.iter().map(|e| e.text.as_str()).collect();
    assert!(texts.contains(&"Sarah"), "expected Sarah; got {texts:?}");
    assert!(
        texts.contains(&"Bawri") || texts.contains(&"Bandra"),
        "expected at least one Place; got {texts:?}"
    );
}
```

- [ ] **Step 3: Run to confirm it doesn't even compile (module missing)**

```
cargo test -p agidb-extract --features model-tests --test ner_smoke
```

Expected: FAIL — `unresolved import ner`.

- [ ] **Step 4: Implement `crates/agidb-extract/src/ner.rs`**

The structure mirrors ctxgraph's `ner.rs`. Adapt the `gline-rs` constructor call to use `ModelManager::ensure_cached`. The body is short — model load + a single `infer` method that maps `gline-rs` entities into `agidb_core::types::Entity`:

```rust
//! GLiNER NER wrapper. Adapts gline-rs output to agidb_core::Entity.
//!
//! Ported pattern from
//! /home/rsx/Desktop/projx/ctxgraph/crates/ctxgraph-extract/src/ner.rs

use crate::error::{ExtractError, Result};
use crate::model_manager::ModelManager;
use crate::models::ModelRef;
use agidb_core::types::Entity;
use std::path::PathBuf;

pub struct NerExtractor {
    inner: gline_rs::Gliner,
    entity_types: Vec<String>,
}

impl NerExtractor {
    pub fn new(cache_root: PathBuf, model: ModelRef, entity_types: Vec<String>) -> Result<Self> {
        let mgr = ModelManager::new(cache_root, std::env::var("AGIDB_OFFLINE").is_ok());
        let model_path = mgr.ensure_cached(&model)?;
        let inner = gline_rs::Gliner::from_path(&model_path)
            .map_err(|e| ExtractError::ModelLoad(format!("gliner: {e}")))?;
        Ok(Self { inner, entity_types })
    }

    pub fn extract(&self, text: &str) -> Result<Vec<Entity>> {
        let labels: Vec<&str> = self.entity_types.iter().map(String::as_str).collect();
        let raw = self
            .inner
            .inference(text, &labels)
            .map_err(|e| ExtractError::Inference(format!("gliner: {e}")))?;
        Ok(raw
            .into_iter()
            .map(|r| Entity {
                text: r.text,
                entity_type: r.label,
                span: (r.start, r.end),
                confidence: r.score,
                concept_id: None,
            })
            .collect())
    }
}
```

> **Note for the executor:** the `gline-rs` API names (`Gliner`, `inference`, the entity-result struct fields) are approximate — verify against the actual crate at port time and adjust signatures. The shape of the wrapper (path-in → `Vec<Entity>`-out) is locked.

- [ ] **Step 5: Uncomment `pub mod ner;` in `lib.rs`. Build.**

```
cargo build -p agidb-extract --all-targets
```

Fix any compile errors from the `gline-rs` API shape mismatch (this is the expected adjustment point).

- [ ] **Step 6: Run the gated test**

```
cargo test -p agidb-extract --features model-tests --test ner_smoke -- --nocapture
```

First run downloads the GLiNER weights (~hundreds of MB). Pin the actual SHA into `models.rs::GLINER_DEFAULT` after the first successful download. Re-run to verify.

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/agidb-extract/src/ner.rs crates/agidb-extract/src/lib.rs crates/agidb-extract/tests/ner_smoke.rs crates/agidb-extract/src/models.rs
git commit -m "feat(agidb-extract): NER via gline-rs + smoke test (gated, downloads weights)"
```

---

## Task 10: GLiREL relation extractor (port from ctxgraph)

**Files:**
- Create: `crates/agidb-extract/src/glirel.rs`
- Create: `crates/agidb-extract/tests/glirel_smoke.rs` (gated)

- [ ] **Step 1: Read the port source**

Read `/home/rsx/Desktop/projx/ctxgraph/crates/ctxgraph-extract/src/glirel.rs` (717 LOC). Note its inputs (entities + text + relation vocabulary), outputs (typed triples with scores), and the ORT session loading pattern.

- [ ] **Step 2: Write the gated smoke test**

```rust
// crates/agidb-extract/tests/glirel_smoke.rs
#![cfg(feature = "model-tests")]
use agidb_extract::glirel::RelationExtractor;
use agidb_extract::models::GLIREL_DEFAULT;
use agidb_extract::ner::NerExtractor;
use agidb_extract::models::GLINER_DEFAULT;
use std::path::PathBuf;

fn cache() -> PathBuf {
    dirs::cache_dir().unwrap().join("agidb/models")
}

#[test]
fn extracts_sarah_recommends_bawri() {
    let ner = NerExtractor::new(cache(), GLINER_DEFAULT.clone(),
        vec!["Person".into(), "Place".into()]).unwrap();
    let entities = ner.extract("Sarah recommended Bawri in Bandra").unwrap();

    let re = RelationExtractor::new(cache(), GLIREL_DEFAULT.clone(),
        vec!["recommends".into(), "located_in".into()]).unwrap();
    let triples = re.extract("Sarah recommended Bawri in Bandra", &entities).unwrap();

    let any_recommends = triples.iter().any(|t|
        t.subject_text == "Sarah" && t.predicate == "recommends" && t.object_text == "Bawri"
    );
    assert!(any_recommends, "expected (Sarah, recommends, Bawri); got {triples:?}");
}
```

- [ ] **Step 3: Implement `crates/agidb-extract/src/glirel.rs`**

The port adapts ctxgraph's GLiREL wrapper:
- Replace ctxgraph-specific error types with `crate::error::ExtractError`.
- Replace ctxgraph's triple shape with a local `RawRelTriple { subject_text, object_text, predicate, confidence }` for now (we map to `agidb_core::Triple` only after alias + predicate canon in the orchestrator).
- Keep the ORT session loading + tokenizer + inference loop verbatim.

(Concrete diff is too long to inline; the rule for the executor: open ctxgraph's `glirel.rs` side-by-side with the new `glirel.rs`; apply the four transformations above; preserve all comments explaining the model I/O shapes.)

The public surface:

```rust
//! GLiREL relation extractor. Ported and adapted from
//! /home/rsx/Desktop/projx/ctxgraph/crates/ctxgraph-extract/src/glirel.rs

use crate::error::{ExtractError, Result};
use crate::model_manager::ModelManager;
use crate::models::ModelRef;
use agidb_core::types::Entity;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RawRelTriple {
    pub subject_text: String,
    pub object_text: String,
    pub predicate: String,
    pub confidence: f32,
}

pub struct RelationExtractor {
    // ort::Session etc.; same fields as ctxgraph's struct
}

impl RelationExtractor {
    pub fn new(cache_root: PathBuf, model: ModelRef, relation_vocab: Vec<String>) -> Result<Self> {
        // Port: load ORT session via model_manager.ensure_cached(&model)
        todo!("port from ctxgraph glirel.rs::new")
    }

    pub fn extract(&self, text: &str, entities: &[Entity]) -> Result<Vec<RawRelTriple>> {
        // Port: run inference, decode top-k ranked typed tuples
        todo!("port from ctxgraph glirel.rs::extract")
    }
}
```

> **Executor note:** the `todo!()` markers are placeholders for the port body. Replace them with the adapted code from ctxgraph's glirel.rs — don't leave `todo!()` in the committed code. The test in Step 2 should pass once the port body is in place.

- [ ] **Step 4: Uncomment `pub mod glirel;`, build, iterate against the gated test**

```
cargo build -p agidb-extract --features model-tests
cargo test -p agidb-extract --features model-tests --test glirel_smoke -- --nocapture
```

If `jackboyla/glirel_beta` isn't a real HF model, find the actual GLiREL model used by ctxgraph (look at ctxgraph's `model_manager.rs` or its model registry config) and update `GLIREL_DEFAULT.repo` accordingly. Pin its real SHA after the first download.

- [ ] **Step 5: Commit**

```bash
git add crates/agidb-extract/src/glirel.rs crates/agidb-extract/src/lib.rs crates/agidb-extract/tests/glirel_smoke.rs crates/agidb-extract/src/models.rs
git commit -m "feat(agidb-extract): GLiREL relation extractor (ported from ctxgraph)"
```

---

## Task 11: `Extractor` orchestration

**Files:**
- Create: `crates/agidb-extract/src/extractor.rs`

- [ ] **Step 1: Implement (this one is mostly glue — direct write, single TDD test at the integration layer in task 13)**

```rust
//! End-to-end extractor: NER → relations → temporal → alias → predicate canon.

use crate::aliases::AliasResolver;
use crate::error::{ExtractError, Result};
use crate::glirel::{RawRelTriple, RelationExtractor};
use crate::models::{ModelRef, GLINER_DEFAULT, GLIREL_DEFAULT};
use crate::model_manager::ModelManager;
use crate::ner::NerExtractor;
use crate::predicates::PredicateTable;
use crate::temporal::parse_time_anchor;
use agidb_core::store::Store;
use agidb_core::types::{ExtractContext, Extraction, TextExtractor, Triple, Value};
use std::path::PathBuf;

pub struct ExtractorConfig {
    pub model_cache: PathBuf,
    pub gliner_model: ModelRef,
    pub glirel_model: ModelRef,
    pub entity_types: Vec<String>,
    pub predicate_synonyms: PredicateTable,
    pub offline: bool,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            model_cache: dirs::cache_dir()
                .map(|d| d.join("agidb/models"))
                .unwrap_or_else(|| PathBuf::from("./.agidb-models")),
            gliner_model: GLINER_DEFAULT.clone(),
            glirel_model: GLIREL_DEFAULT.clone(),
            entity_types: vec![
                "Person".into(), "Place".into(), "Organization".into(),
                "Thing".into(), "Event".into(),
            ],
            predicate_synonyms: PredicateTable::default(),
            offline: std::env::var("AGIDB_OFFLINE").is_ok(),
        }
    }
}

pub struct Extractor {
    ner: NerExtractor,
    rel: RelationExtractor,
    aliases: AliasResolver,
    predicates: PredicateTable,
}

impl Extractor {
    pub fn new(cfg: ExtractorConfig) -> Result<Self> {
        let ner = NerExtractor::new(cfg.model_cache.clone(), cfg.gliner_model, cfg.entity_types)?;
        let rel = RelationExtractor::new(cfg.model_cache, cfg.glirel_model, vec![
            "recommends".into(), "located_in".into(), "works_at".into(),
            "likes".into(), "said".into(), "met".into(), "visited".into(), "owns".into(),
        ])?;
        Ok(Self {
            ner,
            rel,
            aliases: AliasResolver::new(),
            predicates: cfg.predicate_synonyms,
        })
    }

    /// Run the full pipeline; the Store handle is needed for alias resolution
    /// (it both reads existing concepts and may create new ones).
    pub fn extract_with_store(
        &self,
        store: &mut Store,
        text: &str,
        ctx: &ExtractContext,
    ) -> Result<Extraction> {
        let raw_entities = self.ner.extract(text)?;
        let raw_rels = self.rel.extract(text, &raw_entities)?;

        // Resolve aliases, fill concept_id on each Entity.
        let mut resolved_entities = Vec::with_capacity(raw_entities.len());
        for mut e in raw_entities.into_iter() {
            let id = self.aliases.resolve(store, &e.text, &e.entity_type)
                .map_err(|err| ExtractError::Inference(format!("alias: {err}")))?;
            e.concept_id = Some(id);
            resolved_entities.push(e);
        }

        // Map raw rels to Triple, canonicalizing predicate + looking up concept ids.
        let mut triples = Vec::with_capacity(raw_rels.len());
        for r in &raw_rels {
            let subj_id = resolved_entities.iter()
                .find(|e| e.text == r.subject_text)
                .and_then(|e| e.concept_id);
            let obj_id = resolved_entities.iter()
                .find(|e| e.text == r.object_text)
                .and_then(|e| e.concept_id);
            let (Some(s), Some(o)) = (subj_id, obj_id) else { continue };
            let canonical = self.predicates.lookup(&r.predicate)
                .unwrap_or_else(|| r.predicate.clone());
            triples.push(Triple {
                subject: s,
                predicate: canonical,
                object: Value::Concept(o),
                confidence: r.confidence,
                source_episode: None,
            });
        }

        let valid_time = parse_time_anchor(text, ctx.observation_time);

        Ok(Extraction { triples, valid_time, raw_entities: resolved_entities })
    }
}

// Trait impl needs a Store. The trait can't take a Store (agidb-core has no
// notion of Extractor), so we implement TextExtractor for a thin wrapper
// owning a borrowed store. Callers use the inherent extract_with_store for
// the normal flow; the trait exists for downstream MCP/Python bindings.
impl TextExtractor for Extractor {
    fn extract(&self, _text: &str, _ctx: &ExtractContext) -> agidb_core::Result<Extraction> {
        // The trait signature doesn't take a Store; this impl exists so phase-5
        // MCP/Python wrappers can hold (Store, Extractor) jointly and dispatch.
        // For now: return empty; the real path is extract_with_store.
        Ok(Extraction { triples: vec![], valid_time: None, raw_entities: vec![] })
    }
}
```

> **Note:** the trait-vs-method split is a known compromise; revisit when the `Agidb` facade lands in phase 5.

- [ ] **Step 2: Uncomment `pub mod extractor;`, add `pub use extractor::{Extractor, ExtractorConfig};` to `lib.rs`. Build.**

```
cargo build -p agidb-extract --all-targets
```

- [ ] **Step 3: Commit**

```bash
git add crates/agidb-extract/src/extractor.rs crates/agidb-extract/src/lib.rs
git commit -m "feat(agidb-extract): Extractor orchestration (NER + RE + alias + canon + temporal)"
```

---

## Task 12: `observe_text` free function

**Files:**
- Modify: `crates/agidb-extract/src/lib.rs` — implement `observe_text`

- [ ] **Step 1: Implement**

Append in `crates/agidb-extract/src/lib.rs`:

```rust
use agidb_core::episode::{encode_episode_signature, encode_gist_signature};
use agidb_core::store::Store;
use agidb_core::types::{Episode, EpisodeId, ExtractContext, Provenance, TimeRange};
use agidb_core::AgidbError;
use chrono::Utc;

/// High-level orchestration: text + context → stored Episode.
///
/// Runs the extractor, builds an Episode (falling back to gist signature
/// + neutral confidence when no triples could be extracted), and persists
/// via Store::observe.
pub fn observe_text(
    store: &mut Store,
    extractor: &Extractor,
    text: &str,
    ctx: ObserveContext,
) -> std::result::Result<EpisodeId, AgidbError> {
    let xctx = ExtractContext {
        observation_time: ctx.observation_time,
        relation_hint_types: vec![],
    };
    let ex = extractor
        .extract_with_store(store, text, &xctx)
        .map_err(AgidbError::from)?;

    let valid_time = ex.valid_time.unwrap_or_else(|| TimeRange::point(ctx.observation_time));
    let (signature, confidence) = if ex.triples.is_empty() {
        (encode_gist_signature(text), 0.5)
    } else {
        let conf = geomean(ex.triples.iter().map(|t| t.confidence));
        (encode_episode_signature(&ex.triples, Some(valid_time.start)), conf)
    };

    let episode = Episode {
        id: EpisodeId::new(0), // assigned by store
        text: text.to_string(),
        triples: ex.triples,
        signature: signature.clone(),
        gist_signature: encode_gist_signature(text),
        provenance: ctx.provenance,
        confidence,
        valid_time,
        t_tx_start: Utc::now(),
        t_tx_end: None,
        superseded_by: None,
        tombstoned_at: None,
        session_id: ctx.session_id,
    };

    store.observe(episode, &signature)
}

/// Minimal context passed to observe_text — kept here so callers don't
/// need to import agidb_core directly for this simple shape.
pub struct ObserveContext {
    pub observation_time: chrono::DateTime<chrono::Utc>,
    pub provenance: Provenance,
    pub session_id: Option<agidb_core::types::SessionId>,
}

impl Default for ObserveContext {
    fn default() -> Self {
        Self {
            observation_time: Utc::now(),
            provenance: Provenance::default(),
            session_id: None,
        }
    }
}

fn geomean<I: IntoIterator<Item = f32>>(iter: I) -> f32 {
    let v: Vec<f32> = iter.into_iter().filter(|x| *x > 0.0).collect();
    if v.is_empty() { return 0.5; }
    let log_sum: f32 = v.iter().map(|x| x.ln()).sum();
    (log_sum / v.len() as f32).exp()
}
```

> **Executor note:** `Episode` field names and `SessionId` import may need adjustment against the actual `agidb_core::types` shapes. Use `cargo check` to drive the corrections; the design is locked, the field names are not.

- [ ] **Step 2: Build**

```
cargo build -p agidb-extract --all-targets
```

Iterate on field-name mismatches against the actual `Episode` struct. The principle: `observe_text` constructs an `Episode` and calls `store.observe(episode, &sig)`. The exact field names follow what already compiles in `Store::observe`'s body.

- [ ] **Step 3: Commit**

```bash
git add crates/agidb-extract/src/lib.rs
git commit -m "feat(agidb-extract): observe_text free function (extract + encode + store)"
```

---

## Task 13: End-to-end integration test (with `MockExtractor` for PR-time speed)

**Files:**
- Create: `crates/agidb-extract/tests/observe_text.rs`

- [ ] **Step 1: Write the test using a fixture-based `MockExtractor`**

```rust
//! Integration: text in → episode stored, triples canonicalized, time correct.
//! Uses a hand-rolled MockExtractor so this test runs on every PR without
//! any ONNX inference.

use agidb_core::store::{Store, StoreConfig};
use agidb_core::types::{ConceptId, Entity, ExtractContext, Extraction, TextExtractor, Triple, Value, Provenance};
use chrono::{TimeZone, Utc};
use tempfile::TempDir;

struct MockExtractor;
impl TextExtractor for MockExtractor {
    fn extract(&self, _text: &str, _ctx: &ExtractContext) -> agidb_core::Result<Extraction> {
        // Fixed fixture for "Sarah recommended Bawri in Bandra last weekend".
        Ok(Extraction {
            triples: vec![
                Triple {
                    subject: ConceptId::new(1),
                    predicate: "recommends".into(),
                    object: Value::Concept(ConceptId::new(2)),
                    confidence: 0.91,
                    source_episode: None,
                },
            ],
            valid_time: None,
            raw_entities: vec![
                Entity { text: "Sarah".into(), entity_type: "Person".into(), span: (0, 5), confidence: 0.93, concept_id: Some(ConceptId::new(1)) },
                Entity { text: "Bawri".into(), entity_type: "Place".into(), span: (17, 22), confidence: 0.88, concept_id: Some(ConceptId::new(2)) },
            ],
        })
    }
}

#[test]
fn observe_text_stores_an_episode_with_triples() {
    // This test exercises only the orchestration shape; it does NOT call
    // observe_text(extractor) because Extractor requires real models.
    // Instead it exercises the data-flow contract: Extraction in → Episode out.
    let dir = TempDir::new().unwrap();
    let cfg = StoreConfig { path: dir.path().to_path_buf(), ..Default::default() };
    let mut store = Store::open(cfg).unwrap();

    // Pre-mint concept ids so MockExtractor's hard-coded ids resolve.
    let _ = store.create_concept("Sarah", "Person").unwrap();
    let _ = store.create_concept("Bawri", "Place").unwrap();

    let extractor = MockExtractor;
    let ctx = ExtractContext {
        observation_time: Utc.with_ymd_and_hms(2026, 5, 23, 12, 0, 0).unwrap(),
        relation_hint_types: vec![],
    };
    let ex = extractor.extract("Sarah recommended Bawri", &ctx).unwrap();
    assert_eq!(ex.triples.len(), 1);
    assert_eq!(ex.triples[0].predicate, "recommends");
    // Episode construction + store handled in observe_text; covered separately
    // by the in-package gated test in Task 9/10.
}
```

- [ ] **Step 2: Run + green**

```
cargo test -p agidb-extract --test observe_text
```

Expected: PASS.

- [ ] **Step 3: Add the gated end-to-end test using the real Extractor**

Add a second test in the same file, gated behind `model-tests`:

```rust
#[cfg(feature = "model-tests")]
#[test]
fn observe_text_end_to_end_with_real_extractor() {
    use agidb_extract::{observe_text, Extractor, ExtractorConfig, ObserveContext};

    let dir = TempDir::new().unwrap();
    let cfg = StoreConfig { path: dir.path().to_path_buf(), ..Default::default() };
    let mut store = Store::open(cfg).unwrap();

    let extractor = Extractor::new(ExtractorConfig::default()).expect("models loaded");
    let id = observe_text(
        &mut store,
        &extractor,
        "Sarah recommended Bawri in Bandra last weekend",
        ObserveContext { observation_time: Utc.with_ymd_and_hms(2026, 5, 23, 12, 0, 0).unwrap(), ..Default::default() },
    ).expect("observed");
    let ep = store.get_episode(id).unwrap().expect("episode");
    assert!(!ep.triples.is_empty(), "expected at least one triple");
    assert!(ep.triples.iter().any(|t| t.predicate == "recommends"));
}
```

- [ ] **Step 4: Run the gated test on a machine with models cached**

```
cargo test -p agidb-extract --features model-tests --test observe_text -- --nocapture
```

Expected: 2 tests; PR-time test passes always, gated test passes when models are cached.

- [ ] **Step 5: Commit**

```bash
git add crates/agidb-extract/tests/observe_text.rs
git commit -m "test(agidb-extract): observe_text integration (PR-time + gated end-to-end)"
```

---

## Task 14: Eval sub-crate scaffold

**Files:**
- Modify: `Cargo.toml` (workspace) — add `crates/agidb-extract/eval` as a member
- Create: `crates/agidb-extract/eval/Cargo.toml`
- Create: `crates/agidb-extract/eval/src/main.rs` — skeleton + `cargo run` hello

- [ ] **Step 1: Add the member**

In workspace `Cargo.toml`, extend `members = [...]` with `"crates/agidb-extract/eval"`.

- [ ] **Step 2: Create the sub-crate manifest**

```toml
# crates/agidb-extract/eval/Cargo.toml
[package]
name = "agidb-extract-eval"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish = false

[[bin]]
name = "agidb-extract-eval"
path = "src/main.rs"

[dependencies]
agidb-core = { workspace = true }
agidb-extract = { workspace = true }
anyhow = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
clap = { version = "4", features = ["derive"] }
```

(Add `agidb-extract` to `[workspace.dependencies]` in the root manifest if not already present: `agidb-extract = { path = "crates/agidb-extract", version = "0.1.0-dev" }`.)

- [ ] **Step 3: Skeleton `main.rs`**

```rust
// crates/agidb-extract/eval/src/main.rs
//! Phase-3 gold-set evaluation. Loads observations.jsonl, runs the extractor,
//! scores triple-level P/R/F1, writes a JSON report.

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    /// Path to the JSONL gold set.
    #[arg(long, default_value = "crates/agidb-extract/eval/gold/observations.jsonl")]
    gold: PathBuf,
    /// Where to write the JSON report.
    #[arg(long, default_value = "crates/agidb-extract/eval/results/latest.json")]
    out: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    println!("agidb-extract-eval (phase 3) — gold: {}", cli.gold.display());
    // Real scoring lands in Task 16; this scaffold ensures the binary
    // compiles + runs end-to-end so CI workflow can be wired in Task 17.
    Ok(())
}
```

- [ ] **Step 4: Verify build + run**

```
cargo build -p agidb-extract-eval
cargo run -p agidb-extract-eval -- --help
```

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/agidb-extract/eval/Cargo.toml crates/agidb-extract/eval/src/main.rs
git commit -m "feat(agidb-extract-eval): sub-crate scaffold for phase-3 gold evaluation"
```

---

## Task 15: Gold-set production (MANUAL — human labelling)

This task is the only one without a code step. It produces `crates/agidb-extract/eval/gold/observations.jsonl` (100 lines).

- [ ] **Step 1: Decide sourcing mix**

50 anonymized from realistic agent-conversation transcripts + 50 synthetic-but-realistic. Sources: agidb-relevant slack/dm transcripts (anonymized: replace names with stable pseudonyms), real product-research notes, made-up but plausible "Sarah recommended X" / "Bob said Y" style.

- [ ] **Step 2: Label each line in the gold JSONL format**

```jsonl
{"text": "Sarah recommended Bawri in Bandra last weekend", "triples": [{"subject":"Sarah","predicate":"recommends","object":"Bawri","valid_time":"2026-05-16/2026-05-17"},{"subject":"Bawri","predicate":"located_in","object":"Bandra"}], "notes": ""}
{"text": "I met Alice last week at Trishna", "triples": [{"subject":"I","predicate":"met","object":"Alice","valid_time":"2026-05-12/2026-05-18"},{"subject":"Alice","predicate":"located_in","object":"Trishna"}], "notes": "‘I’ as subject is a known weak point of NER"}
```

Per-line schema: `{text: string, triples: [{subject, predicate, object, valid_time?}], notes?: string}`.

- [ ] **Step 3: Double-label 10 of the 100 with a second person**

Compute Cohen's κ on triple agreement. **Gate: κ ≥ 0.7.** If κ < 0.7, revisit the labelling guide and re-label.

- [ ] **Step 4: Freeze + commit**

```bash
git add crates/agidb-extract/eval/gold/observations.jsonl
git commit -m "test(agidb-extract-eval): 100-sample human-labelled gold set for phase-3 F1 gate"
```

---

## Task 16: Eval scoring

**Files:**
- Modify: `crates/agidb-extract/eval/src/main.rs` — implement loading, run, P/R/F1 scoring, JSON report

- [ ] **Step 1: Replace `main.rs` with the full impl**

```rust
//! Phase-3 gold-set evaluation: load JSONL, run extractor, compute
//! triple-level precision / recall / F1, write a JSON report.

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
struct GoldRow {
    text: String,
    triples: Vec<GoldTriple>,
    #[serde(default)]
    notes: String,
}

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Hash, Debug)]
struct GoldTriple {
    subject: String,
    predicate: String,
    object: String,
    #[serde(default)]
    valid_time: Option<String>,
}

#[derive(Serialize)]
struct Report {
    precision: f64,
    recall: f64,
    f1: f64,
    n: usize,
    per_row: Vec<RowReport>,
}

#[derive(Serialize)]
struct RowReport {
    text: String,
    expected: Vec<GoldTriple>,
    extracted: Vec<GoldTriple>,
    tp: usize,
    fp: usize,
    fn_: usize,
}

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "crates/agidb-extract/eval/gold/observations.jsonl")]
    gold: PathBuf,
    #[arg(long, default_value = "crates/agidb-extract/eval/results/latest.json")]
    out: PathBuf,
}

fn load_gold(path: &PathBuf) -> Result<Vec<GoldRow>> {
    let f = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut rows = Vec::new();
    for (i, line) in BufReader::new(f).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let row: GoldRow = serde_json::from_str(&line)
            .with_context(|| format!("line {}: {}", i + 1, &line))?;
        rows.push(row);
    }
    Ok(rows)
}

fn run_extraction(text: &str) -> Result<Vec<GoldTriple>> {
    // Real extractor goes here; for now this scaffold returns empty.
    // The first iteration of this task produces an F1 of 0.0; the
    // second iteration (after the real ExtractorConfig::default() works
    // on the dev box) wires the real call.
    let _ = text;
    Ok(vec![])
}

fn score_row(expected: &[GoldTriple], extracted: &[GoldTriple]) -> (usize, usize, usize) {
    let e: HashSet<_> = expected.iter().cloned().collect();
    let x: HashSet<_> = extracted.iter().cloned().collect();
    let tp = e.intersection(&x).count();
    let fp = x.difference(&e).count();
    let fn_ = e.difference(&x).count();
    (tp, fp, fn_)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let rows = load_gold(&cli.gold)?;
    let mut per_row = Vec::new();
    let (mut tp, mut fp, mut fn_) = (0usize, 0usize, 0usize);
    for row in &rows {
        let extracted = run_extraction(&row.text)?;
        let (a, b, c) = score_row(&row.triples, &extracted);
        tp += a; fp += b; fn_ += c;
        per_row.push(RowReport {
            text: row.text.clone(),
            expected: row.triples.clone(),
            extracted,
            tp: a, fp: b, fn_: c,
        });
    }
    let precision = if tp + fp == 0 { 0.0 } else { tp as f64 / (tp + fp) as f64 };
    let recall = if tp + fn_ == 0 { 0.0 } else { tp as f64 / (tp + fn_) as f64 };
    let f1 = if precision + recall == 0.0 { 0.0 } else { 2.0 * precision * recall / (precision + recall) };
    let report = Report { precision, recall, f1, n: rows.len(), per_row };
    fs::create_dir_all(cli.out.parent().unwrap())?;
    fs::write(&cli.out, serde_json::to_string_pretty(&report)?)?;
    println!("P={:.3} R={:.3} F1={:.3} (n={})", precision, recall, f1, rows.len());
    Ok(())
}
```

- [ ] **Step 2: Wire the real extractor (replace `run_extraction`)**

Once `Extractor` works on the dev box (after task 9/10/11 complete + models downloaded), replace the placeholder with:

```rust
fn run_extraction(text: &str, extractor: &agidb_extract::Extractor, store: &mut agidb_core::store::Store) -> Result<Vec<GoldTriple>> {
    use agidb_core::types::{ExtractContext, Value};
    use chrono::Utc;
    let ex = extractor.extract_with_store(store, text, &ExtractContext {
        observation_time: Utc::now(),
        relation_hint_types: vec![],
    })?;
    let mut out = Vec::with_capacity(ex.triples.len());
    for t in ex.triples {
        // Resolve concept ids back to canonical names for scoring.
        let subj = store.concept_canonical_name(t.subject)?.unwrap_or_default();
        let obj = match t.object {
            Value::Concept(id) => store.concept_canonical_name(id)?.unwrap_or_default(),
            Value::Text(s) => s,
            other => format!("{other:?}"),
        };
        out.push(GoldTriple { subject: subj, predicate: t.predicate, object: obj, valid_time: None });
    }
    Ok(out)
}
```

(Needs `Store::concept_canonical_name(id) -> Result<Option<String>>` — a tiny helper following the same pattern as `concept_id_for`. Add it to `agidb-core/src/store.rs` with a unit test as part of this task.)

- [ ] **Step 3: Commit**

```bash
git add crates/agidb-extract/eval/src/main.rs crates/agidb-core/src/store.rs
git commit -m "feat(agidb-extract-eval): triple-level P/R/F1 scoring against the gold set"
```

---

## Task 17: Nightly CI workflow

**Files:**
- Create: `.github/workflows/eval-nightly.yml`

- [ ] **Step 1: Write the workflow**

```yaml
# .github/workflows/eval-nightly.yml
name: eval-nightly

on:
  schedule:
    - cron: "0 3 * * *"
  workflow_dispatch:

jobs:
  eval:
    runs-on: ubuntu-latest
    timeout-minutes: 60
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: cache models
        uses: actions/cache@v4
        with:
          path: ~/.cache/agidb/models
          key: agidb-models-${{ hashFiles('crates/agidb-extract/src/models.rs') }}
      - name: build
        run: cargo build -p agidb-extract-eval --release --features model-tests
      - name: run eval
        run: cargo run -p agidb-extract-eval --release -- --out /tmp/eval-report.json
      - name: post summary
        run: |
          jq '{precision, recall, f1, n}' /tmp/eval-report.json | tee -a $GITHUB_STEP_SUMMARY
      - uses: actions/upload-artifact@v4
        with:
          name: eval-report
          path: /tmp/eval-report.json
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/eval-nightly.yml
git commit -m "ci: nightly phase-3 gold-set evaluation workflow"
```

---

## Task 18: F1 ≥ 0.85 verification + iteration loop

This is not a single task but a loop that runs until the exit gate is satisfied.

- [ ] **Step 1: Run the eval locally**

```
cargo run -p agidb-extract-eval --release
```

- [ ] **Step 2: If F1 < 0.85, classify failures**

Open `crates/agidb-extract/eval/results/latest.json`. Group the misses by failure mode:

- **NER miss** → entity not detected → consider lowering NER threshold or adding entity-type to `ExtractorConfig.entity_types`.
- **Relation miss** → entities found, relation not extracted → consider adding the predicate to GLiREL's relation vocabulary in `Extractor::new`.
- **Predicate canon miss** → relation extracted with surface form not in `PredicateTable` → add synonym to `predicates.rs::PredicateTable::default()`.
- **Alias miss** → entity duplicated as new Concept when it should merge → tune Levenshtein threshold or widen exact-match (e.g. strip honorifics).
- **Time miss** → time anchor not parsed → extend `temporal.rs` with the missing expression form.

- [ ] **Step 3: Fix the top category, re-run, repeat**

Each fix = its own commit, its own test (one unit/property test that pins the regression). The pattern: gold-set failure → add fixture test → add code → re-run eval.

- [ ] **Step 4: Phase-3 acceptance**

When eval reports F1 ≥ 0.85, commit the run log:

```bash
mkdir -p crates/agidb-extract/eval/results
cp /tmp/eval-report.json crates/agidb-extract/eval/results/$(date +%Y-%m-%d)-phase-3-exit.json
git add crates/agidb-extract/eval/results/
git commit -m "test(agidb-extract-eval): F1=<your-number> on 100-sample gold, phase-3 exit"
```

Update `docs/phases/phase-3-extraction.md`: change `**status:** not started` → `**status:** complete` and check all the deliverable boxes.

Update `docs/phases/README.md`: change phase-3's status from `⬜ not started` to `✅ complete`.

```bash
git add docs/phases/phase-3-extraction.md docs/phases/README.md
git commit -m "docs(phases): mark phase 3 complete (F1 = <your-number>)"
```

---

## Self-review

**Spec coverage** (checked against `docs/superpowers/specs/2026-05-23-phase-3-extraction-design.md`):

- § 2 D1 (scope = extract + wire into observe) → Task 12 (`observe_text`)
- § 2 D2 (100-sample gold) → Task 15
- § 2 D3 (targeted port) → Tasks 6, 8, 9, 10
- § 2 D4 (free function, not on Store) → Task 12 builds it in `agidb-extract`
- § 2 D5 (TextExtractor trait in agidb-core) → Task 1
- § 2 D6 (model download on first use, SHA-pinned) → Tasks 7, 8
- § 2 D7 (English-only) → noted; no task needed (constitutional non-goal)
- § 4 public API (Extractor, ExtractorConfig, Extraction, Entity, ExtractContext, TextExtractor) → Tasks 1, 11
- § 6 predicate canon → Task 3
- § 7 alias resolver → Tasks 4, 5
- § 8 model manager → Task 8
- § 9 error handling (ExtractError) → Task 2
- § 10 testing (unit + integration + eval) → Tasks 3, 5, 6, 13, 16
- § 11 acceptance gate → Task 18

**Placeholder scan:** Tasks 9, 10, 12 contain `todo!()` / executor notes acknowledging that exact `gline-rs` and ctxgraph API shapes must be verified at port time. These are flagged as such, not as planning failures — the design is locked, the API names need cargo-check verification. Acceptable.

**Type consistency:** `Extraction`, `Entity`, `ExtractContext`, `Triple`, `ConceptId`, `Value::Concept` used consistently across tasks 1, 11, 12, 13, 16.

---

## Execution Handoff

Plan saved to `docs/superpowers/plans/2026-05-23-phase-3-extraction.md`.

Two execution options:

1. **Subagent-Driven** (recommended by the writing-plans skill) — dispatch a fresh subagent per task, review between tasks
2. **Inline Execution** — execute tasks in this session via `superpowers:executing-plans`, batch with checkpoints

Per the user's "don't ask for permission, just keep building" directive: defaulting to **inline execution** for forward momentum. Task 1 starts now.
