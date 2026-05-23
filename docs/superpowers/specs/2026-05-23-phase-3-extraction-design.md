# Phase 3 — Extraction (GLiNER + GLiREL): design spec

- **Status:** Approved
- **Date:** 2026-05-23
- **Phase:** 3 (extraction, weeks 1–4 of agidb v2)
- **Author:** rohan + claude (brainstorming session)
- **Implements:** [`docs/phases/phase-3-extraction.md`](../../phases/phase-3-extraction.md), [`docs/architecture/layer-2-extraction.md`](../../architecture/layer-2-extraction.md)

## 1 — Context

agidb-core ships phases 0/1/2/4/6: HDC kernel, redb + mmap storage, episode binding, four-tier recall, consolidation. Layer 1 and 3 work. Layer 2 (extraction) is a stub — `Store::observe(Episode, &HV)` takes pre-built triples, so the only way to populate the database today is to hand-construct `Triple` values, which means **tier B (similarity) of the recall cascade is currently dead** (it relies on canonicalized triple-signature similarity).

Phase 3 lands layer 2: text in, structured triples + canonical entities + parsed time anchors out. Once it lands, tier B activates and `recall()` answers natural-language cues with the full cascade.

A sibling Rust repo `ctxgraph` already has a working extraction pipeline (`ctxgraph-extract`, 9.6k LOC) — GLiNER for NER via the `gline-rs` crate, GLiREL for relations, `temporal.rs` for time parsing, `model_manager.rs` for ONNX model downloads. The phase-3 brief explicitly says *"vendor / port the GLiNER ONNX loading and inference code from ctxgraph"*.

## 2 — Decisions

| # | Decision | Rationale |
|---|---|---|
| D1 | **Scope = extract crate + wire into a text observe path** (not the full `Agidb` umbrella type). | Satisfies the literal exit criterion "`observe()` correctly extracts triples" without overlapping phase 5 (MCP/Python/`Agidb` facade). |
| D2 | **Gold set = 100 hand-labelled observations** (not 20). | Statistical meaningfulness — at 100, every mis-label moves F1 by 1pp, so the >85% threshold is interpretable for the phase-7 decision gate. The 20-number in `phase-3-extraction.md` was a placeholder; roadmap + `layer-2-extraction.md` already say 100. |
| D3 | **Approach: targeted port from ctxgraph**: `gline-rs` for NER + port `glirel.rs` / `temporal.rs` / `model_manager.rs` into a clean `agidb-extract`. | Heuristic relations don't reliably hit 85% F1. Whole-crate-dep on ctxgraph violates embedded-first / small-core. Targeted port is the only approach where the F1 bar is realistic on first attempt without the heavy crate-dep. |
| D4 | **Orchestration as a free function `agidb-extract::observe_text(&mut Store, &Extractor, text, ctx)`** — *not* a method on `Store`. | Keeps `agidb-core` (layer 1+3) extraction-blind. No churn on existing `Store::observe(Episode, &HV)` signature. No circular deps. |
| D5 | **`TextExtractor` trait in `agidb-core`, impl in `agidb-extract`**. | Future callers (MCP server, Python bindings, tests) can take any extractor; no agidb-core → agidb-extract dep needed. |
| D6 | **Model weights: download on first use, cached at `~/.cache/agidb/models/<sha>/`, SHA-pinned in code.** | Constitution-compliant (no network at read/write *time*, only setup). Matches `layer-2-extraction.md` ("`agidb-cli setup-encoders`"). `AGIDB_OFFLINE=1` to forbid downloads. |
| D7 | **English-only for v0.1.** | Per `phase-3-extraction.md` risk table; documented constitutional non-goal (Article XII). |

## 3 — Architecture (layering)

```
crates/
├── agidb-core/          (layer 1 + 3 + types — extraction-blind)
│   ├── types.rs         Triple, Concept, TimeRange, … + new: TextExtractor trait,
│   │                                                          ExtractContext, Extraction, Entity
│   ├── store.rs         Store::observe(Episode, &HV)   (unchanged signature)
│   │                    Store::create_concept(...)     (new helper for alias resolver)
│   └── episode.rs       encode_episode_signature(…)    (unchanged)
│
└── agidb-extract/       (layer 2 — new home for everything text-to-triples)
    ├── lib.rs           pub fn observe_text(&mut Store, &Extractor, text, ctx) -> Result<EpisodeId>
    │                    pub use extractor::{Extractor, ExtractorConfig};
    ├── extractor.rs     Extractor + pipeline orchestration + impl TextExtractor
    ├── ner.rs           gline-rs wrapper (entities + types)
    ├── glirel.rs        ported relation extractor (ORT, ranked typed tuples)
    ├── temporal.rs      ported chrono_english + small grammar
    ├── aliases.rs       alias resolver (exact + Levenshtein ≤ 3) against Store's concepts
    ├── predicates.rs    predicate canonicalizer (curated trie)
    ├── model_manager.rs ported HF download + SHA cache → ~/.cache/agidb/models/
    ├── models.rs        SHA-pinned ModelRef constants for the default GLiNER + GLiREL weights
    └── error.rs         ExtractError → converts to AgidbError::Extraction at the boundary

crates/agidb-extract/
├── eval/                (binary + gold set, behind a CI workflow, not per-PR)
│   ├── Cargo.toml       (separate sub-crate; bin target = agidb-extract-eval)
│   ├── src/main.rs      load gold, run extractor, compute P/R/F1, write JSON + console summary
│   ├── gold/observations.jsonl    100 hand-labelled samples
│   └── results/                   .gitignored; nightly run output lands here
```

**Boundary, one line:** `agidb-extract` knows about `agidb-core`; `agidb-core` knows nothing about `agidb-extract`.

## 4 — Public API

### `agidb-core` (additions, layer-2-blind)

```rust
// agidb-core/src/types.rs (additions)
pub struct ExtractContext {
    pub observation_time: DateTime<Utc>,
    pub relation_hint_types: Vec<String>,
}

pub struct Extraction {
    pub triples: Vec<Triple>,           // already alias-resolved + predicate-canonicalized
    pub valid_time: Option<TimeRange>,  // None → caller falls back to observation_time
    pub raw_entities: Vec<Entity>,      // pre-resolution, for debugging
}

pub struct Entity {
    pub text: String,
    pub entity_type: String,
    pub span: (usize, usize),
    pub confidence: f32,
    pub concept_id: Option<ConceptId>,  // populated by alias resolver
}

pub trait TextExtractor {
    fn extract(&self, text: &str, ctx: &ExtractContext) -> Result<Extraction>;
}
```

### `agidb-extract`

```rust
// agidb-extract/src/lib.rs
pub use crate::extractor::{Extractor, ExtractorConfig};
pub use crate::error::ExtractError;

pub fn observe_text(
    store: &mut Store,
    extractor: &Extractor,
    text: &str,
    ctx: ObserveContext,
) -> Result<EpisodeId, AgidbError>;

// agidb-extract/src/extractor.rs
pub struct Extractor { /* private: ner, glirel, temporal, aliases, predicates */ }

impl Extractor {
    pub fn new(config: ExtractorConfig) -> Result<Self, ExtractError>; // blocking; setup-time
    pub fn extract(&self, text: &str, ctx: &ExtractContext) -> Result<Extraction, ExtractError>;
}

impl TextExtractor for Extractor {
    fn extract(&self, text: &str, ctx: &ExtractContext) -> Result<Extraction> {
        self.extract(text, ctx).map_err(Into::into)
    }
}

pub struct ExtractorConfig {
    pub model_cache: PathBuf,                  // default: dirs::cache_dir().join("agidb/models")
    pub gliner_model: ModelRef,                // default: models::GLINER_DEFAULT
    pub glirel_model: ModelRef,                // default: models::GLIREL_DEFAULT
    pub entity_types: Vec<String>,             // default: Person, Place, Organization, Thing, Event
    pub predicate_synonyms: PredicateTable,    // default: built-in trie
    pub offline: bool,                         // honors AGIDB_OFFLINE env if true
}

impl Default for ExtractorConfig { /* sensible defaults from the constants */ }

// agidb-extract/src/models.rs
pub struct ModelRef { pub repo: &'static str, pub revision: &'static str, pub sha256: &'static str }

pub const GLINER_DEFAULT: ModelRef = ModelRef {
    repo: "urchade/gliner_multi-v2.1",
    revision: "main",
    sha256: "TBD-PIN-AT-VENDOR-TIME",
};
pub const GLIREL_DEFAULT: ModelRef = ModelRef {
    repo: "jackboyla/glirel_beta",   // candidate — confirm during vendoring
    revision: "main",
    sha256: "TBD-PIN-AT-VENDOR-TIME",
};
```

> The SHA placeholders are deliberate: the exact ONNX-file SHA only becomes available when the vendoring step downloads the artifact for the first time. The implementation plan's *"wire `model_manager`"* task includes a step to pin both SHAs and commit them. The GLiREL `repo` candidate also gets confirmed (or swapped) in week 1 — `jackboyla/glirel_beta` is a placeholder, not a load-bearing choice.

## 5 — Data flow (`observe_text`)

```
USER: observe_text(store, ext, "Sarah recommended Bawri in Bandra last weekend", ctx)
  │
  ▼ extractor.extract(text, ctx)
  │
  │   1. NER (gline-rs)
  │      → [Sarah:Person@0..5 (0.93), Bawri:Place@17..22 (0.88), Bandra:Place@26..32 (0.91)]
  │
  │   2. RELATION EXTRACTION (GLiREL, ORT)
  │      → [(Sarah, "recommended", Bawri, 0.91), (Bawri, "in", Bandra, 0.83)]
  │
  │   3. TEMPORAL (chrono_english anchored at ctx.observation_time = 2026-05-23)
  │      "last weekend" → TimeRange { start: 2026-05-16T00:00Z, end: 2026-05-17T23:59Z }
  │
  │   4. ALIAS RESOLUTION (against store.concepts / store.concept_by_name)
  │      Sarah  → ConceptId(42) (exact match)
  │      Bawri  → ConceptId(43, new); Bandra → ConceptId(44, new)
  │
  │   5. PREDICATE CANONICALIZATION (curated trie)
  │      "recommended" → recommends ; "in" → located_in
  │
  │   → Extraction { triples: […], valid_time: Some(…), raw_entities: […] }
  │
  ▼ observe_text continues:
  │   episode = Episode {
  │     text: original,
  │     triples: extraction.triples,
  │     valid_time: extraction.valid_time.unwrap_or_else(|| TimeRange::point(ctx.observation_time)),
  │     provenance: ctx.provenance,
  │     confidence: geomean(extraction.triples.iter().map(|t| t.confidence)),
  │     t_tx_start: now,
  │     ...
  │   };
  │   signature = encode_episode_signature(&episode.triples, Some(episode.valid_time.start));
  │   store.observe(episode, &signature) -> EpisodeId
  ▼
USER gets EpisodeId
```

**Edge case — no triples extracted.** When `extraction.triples` is empty (the extractor found no entities or relations), we still store the episode (the raw text remains useful for tier C / gist recall) with `confidence = 0.5` and `triples = vec![]`. The episode signature falls back to `encode_gist_signature(text)` instead of `encode_episode_signature(triples, …)`. An integration test pins this behavior.

## 6 — Predicate canonicalization

Curated trie, ~100 entries to start, one canonical form per surface verb. Exact match first (case-folded), multi-word OK.

| Canonical | Surface forms (examples — full list in `predicates.rs`) |
|---|---|
| `recommends` | recommended, suggested, "told me about", pitched |
| `located_in` | in, "based in", "is from", "lives in" |
| `works_at` | "works at", "is employed by", "is at" |
| `likes` | likes, loves, prefers, "is into" |
| `said` | said, told, claimed, mentioned |
| `met` | met, "ran into", "saw" |

Unknown predicate → keep surface form verbatim with `predicate_canonical: None` flag. Custom synonyms loadable via `ExtractorConfig.predicate_synonyms`.

## 7 — Alias resolution

Against `Store`'s existing `concepts` + `concept_by_name` tables:

1. **Exact match** (case-folded) on `canonical_name` or any `aliases[]` → return existing `ConceptId`. Most cases.
2. **Levenshtein ≤ 3** on canonical names (O(N) scan; N small for v0.1) → if exactly one candidate within distance 3, treat as alias; otherwise be conservative: don't merge, create new.
3. **Miss** → `Store::create_concept(Concept { canonical_name: entity.text, aliases: vec![], concept_type: entity.entity_type, signature: HV::from_name(...) })` → new `ConceptId`.

Embedding-similarity fuzzy matching is deferred to v0.3+ per `layer-2-extraction.md`.

## 8 — Model acquisition (`model_manager.rs`)

Ported from ctxgraph. Behavior:

- Cache at `~/.cache/agidb/models/<repo-path-sanitized>/<sha>/`.
- First `Extractor::new` checks cache; on miss, downloads from HuggingFace via `reqwest` (rustls-tls), verifies SHA256, atomic-renames into place.
- All model refs SHA-pinned in `agidb-extract/src/models.rs`. Updating a model = a code change + new SHA.
- **Offline mode:** `AGIDB_OFFLINE=1` or `ExtractorConfig.offline = true` → error on cache miss instead of downloading.
- **Constitution-compliant:** zero network calls at read or write *time*; downloads are setup-time, explicit, opt-out-able.

## 9 — Error handling

```rust
// agidb-extract/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("model load: {0}")]      ModelLoad(String),
    #[error("model download: {0}")]  ModelDownload(String),
    #[error("ort inference: {0}")]   Inference(String),
    #[error("tokenize: {0}")]        Tokenize(String),
    #[error("io: {0}")]              Io(#[from] std::io::Error),
}

impl From<ExtractError> for AgidbError {
    fn from(e: ExtractError) -> Self { AgidbError::Extraction(e.to_string()) }
}
```

- **Fatal**: model load / inference / tokenize failures.
- **Non-fatal**: time-parse failure → `Extraction.valid_time = None`; the orchestrator falls back to `ctx.observation_time`. Unknown predicate → keep surface verb (`predicate_canonical: None`). We don't fail an observation over a phrase we couldn't normalize.

## 10 — Testing

| Layer | Location | What |
|---|---|---|
| **Unit** | `agidb-extract/tests/` | `ner_smoke.rs` (5–10 fixture sentences); `glirel_smoke.rs` (5–10); `temporal_properties.rs` (50 cases incl. proptest); `aliases_properties.rs` (exact + Levenshtein boundary); `predicates_properties.rs` (~30 surface→canonical). Property: `extract()` deterministic (same text + same model → same triples). |
| **Integration** | `agidb-extract/tests/observe_text.rs` | 3 end-to-end fixtures: "Sarah recommended Bawri", "I met Alice last week at Trishna", "Bob works at Acme". Assert episode stored, triples present + canonicalized, valid_time correct, signature deterministic. Uses a `MockExtractor` (skips model load) for fast PR-time runs; one slower test with the real `Extractor` gated behind a `cargo test --features model-tests`. |
| **Eval** | `agidb-extract/eval/` | 100-sample gold-set evaluation. Run via `cargo run -p agidb-extract-eval --release`. Triple-level P/R/F1 (correct subject + canonical predicate + correct object). Writes `eval/results/<date>.json`. |

### Gold set — `eval/gold/observations.jsonl`

Schema:
```json
{"text": "...", "triples": [{"subject":"Sarah","predicate":"recommends","object":"Bawri","valid_time":"2026-05-16/2026-05-17"}], "notes": "optional"}
```

Production workflow:
1. Source: ~50 from realistic anonymized agent-conversation transcripts, ~50 synthetic-but-realistic.
2. Solo-label all 100; double-label 10 random ones; Cohen's κ ≥ 0.7 → frozen.
3. Committed at `eval/gold/observations.jsonl`; locked for phase 3 (revisit only during phase-7 prep).

### CI

- `cargo test --workspace` (per-PR): unit + integration tests; **no real model inference** (mocks + fixtures only). Target: still completes under 90s.
- `eval-nightly.yml` (nightly + `workflow_dispatch`): runs `agidb-extract-eval` against the gold set, posts F1 + per-category breakdown to the Actions summary. Caches `~/.cache/agidb/models/` between runs.

## 11 — Phase-3 acceptance gate

The phase exits **only** when all of:

1. `cargo test --workspace` green (existing 44 + ~20 new = ~64).
2. `cargo clippy --workspace --all-targets -- -D warnings` clean.
3. `cargo fmt --all -- --check` clean.
4. Gold-set eval reports **F1 ≥ 0.85** on the 100-sample set, with the raw run log committed to `eval/results/`.
5. `observe_text` works end-to-end (covered by the integration tests).
6. Tier B reachability: a new integration test asserts that after `observe_text` of a fixture, a `Store::recall` cue lands `Tier::Similarity` (currently dead because no canonicalized triples exist).

## 12 — What's NOT in phase 3 (explicit deferrals)

- **Belief extraction** → phase 9 (constitution Article XVII). Phase 3 only produces `Triple`s; `Belief` candidates wait.
- **`Agidb` umbrella API type with `observe(text)`** → phase 5, alongside MCP server and Python bindings.
- **Multimodal extraction** (V-JEPA 2 / Wav2Vec-BERT / Llama-3.2-3B) → phase 14 (v2.1, gated on the decision gate).
- **Embedding-similarity fuzzy alias matching** → v0.3+.
- **GLiNER2** (joint NER+RE in one model) → blocked on ONNX export from Fastino; revisit when available.
- **LLM-fallback extraction for low-confidence cases** → defer; if F1 stalls below 0.85, the phase-3 risk-table mitigation is regex/BM25 backstop (Article IV-compliant), not an LLM call at write time.

## 13 — Risks (with mitigations)

| Risk | Mitigation |
|---|---|
| GLiREL ONNX inference >200ms p50 | quantize to int8; cache ORT session per-process; choose smaller GLiREL variant if needed |
| F1 stalls below 0.85 on 100-sample gold | augment with predicate-synonym additions; high-recall regex backstop for known-hard entity types; document tradeoff in `eval/results/notes.md` |
| Time-anchor grammar is English-only | scoped non-goal; document in constitution article XII non-goals |
| Gold-set labelling bias | inter-annotator agreement check on 10/100 samples; κ ≥ 0.7 gate |
| Model weights drift on HF (different SHA) | SHA-pinned; download verifies; updating a model is an explicit code change |
| ctxgraph port introduces subtle bugs vs original | port one file at a time with its tests; run ctxgraph's existing tests against the ported version where possible |
| `gline-rs` upstream API churn | pin exact version in workspace deps; track in `Cargo.lock` |

## 14 — As-built adjustments (logged 2026-05-23 during execution)

The design above was written against the agidb v2 **tech-spec** types. The
actually-implemented v1 types in `agidb-core` are simpler than the spec
assumed. The plan was adjusted inline during execution; recording the
deltas here so the spec remains an honest record:

- **`Triple` is simpler than the spec.** The v1 `Triple` has
  `subject: String`, `object: String`, `episode_id: EpisodeId` (no `Value`
  enum, no `Option<EpisodeId>`). Phase 3 introduced **`ExtractedTriple`**
  in `agidb-core::types` as the layer-2-facing shape (no `episode_id`);
  `observe_text` promotes `ExtractedTriple` to `Triple` once it mints an
  `EpisodeId`.
- **`Entity` uses `canonical_name: Option<String>`** instead of
  `concept_id: Option<ConceptId>` — keeps everything string-based to
  match the existing `Triple.subject`/`object` shape.
- **`Concept` lacks `signature` / `created_at` / `withdrawn_at`** and uses
  `entity_type` (not `concept_type`). `Store::create_concept` writes only
  the fields that actually exist.
- **`Episode` uses `signature_offset: u64`**, not `signature: HV`. The
  caller passes `signature_offset: 0`; `Store::observe` overwrites it
  with the real mmap offset after appending the HV to `signatures.dat`.
- **No `next_episode_id` counter existed in phase-2 storage.** Phase 3
  added `Store::next_episode_id` (analogous to `next_concept_id`), along
  with the `KEY_NEXT_EPISODE_ID` manifest key.
- **No `SessionId` type exists.** `Provenance.session_id` is
  `Option<String>` in the v1 schema; `ObserveContext` accordingly carries
  only `observation_time` + `provenance`, not a separate session id.
- **`chrono_english` 0.1.8 doesn't parse English number-words** ("two
  months ago"). Added a small word-to-digit normalizer in
  `temporal.rs::normalize_number_words` so NER-produced word forms reach
  the parser as digits.
- **`StoreConfig` field is `root`**, not `path`; constructed with
  `StoreConfig::at(path)`, not struct literal with a `Default` (there is
  none).

These adjustments preserve every behavioral decision in §§ 1–13. Only
the concrete type names / field names changed.

## 15 — References

- [`docs/phases/phase-3-extraction.md`](../../phases/phase-3-extraction.md) — the phase brief (this design implements it)
- [`docs/architecture/layer-2-extraction.md`](../../architecture/layer-2-extraction.md) — layer-2 architecture
- [`docs/spec/tech-spec.md`](../../spec/tech-spec.md) — `Triple`, `ConceptId`, `Episode` types
- [`docs/product/roadmap.md`](../../product/roadmap.md) — week 1–4 plan
- `/home/rsx/Desktop/projx/ctxgraph/crates/ctxgraph-extract/` — port source
- [`.specify/memory/constitution.md`](../../../.specify/memory/constitution.md) — articles IV (no LLM in read path), XI (small composable API), XII (sacred non-goals)
