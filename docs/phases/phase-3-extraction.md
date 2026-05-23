# phase 3 — extraction

**duration:** weeks 1-4
**status:** in progress — v1 end-to-end working; v2 needs ONNX relation extractor + 100-sample gold + real model SHAs
**depends on:** [phase 2](./phase-2-storage.md)

## goal

land layer 2: replace the phase-2 regex placeholder with a real GLiNER ONNX pipeline that extracts entities, relations, and time anchors with calibrated confidence.

## deliverables

- [x] vendor / port the GLiNER ONNX loading and inference code from ctxgraph — [`ner.rs`](../../crates/agidb-extract/src/ner.rs) + [`model_manager.rs`](../../crates/agidb-extract/src/model_manager.rs); gated smoke test in [`tests/ner_smoke.rs`](../../crates/agidb-extract/tests/ner_smoke.rs)
- [x] `agidb-extract/src/lib.rs` with the `Extraction` pipeline ([`extractor.rs`](../../crates/agidb-extract/src/extractor.rs) implements `TextExtractor`):
  - [x] entity extraction with type labels (Person, Place, Organization, etc.) — via gline-rs
  - [x] relation extraction producing `(subject, predicate, object)` triples — **v1 stub** in [`heuristic_relations.rs`](../../crates/agidb-extract/src/heuristic_relations.rs) (PredicateTable-based, no ML); v2 work: port `glirel.rs` or `relex.rs` from ctxgraph
  - [x] time anchor parsing — [`temporal.rs`](../../crates/agidb-extract/src/temporal.rs)
  - [x] confidence scoring (GLiNER logits → Entity.confidence; heuristic → 0.5 triple confidence)
- [x] alias resolution and concept canonicalization (uses `concepts` table from phase 2) — [`aliases.rs`](../../crates/agidb-extract/src/aliases.rs)
- [x] predicate canonicalization with a hand-curated synonym table — [`predicates.rs`](../../crates/agidb-extract/src/predicates.rs)
- [ ] **100-sample** gold dataset committed to `crates/agidb-extract/eval/gold/observations.jsonl` (revised up from 20 during the brainstorming session). **3-entry placeholder committed**; human labelling work for the full 100 is plan task 15
- [x] eval script that computes F1 against gold — [`agidb-extract-eval`](../../crates/agidb-extract/eval/src/main.rs) sub-crate; exits 2 if F1 < 0.85; nightly CI in [`.github/workflows/eval-nightly.yml`](../../.github/workflows/eval-nightly.yml)

## exit criterion

`observe_text()` correctly extracts triples from **100 sample observations** with **F1 ≥ 0.85** against the human-labelled gold set. Measured by `cargo run -p agidb-extract-eval --release`; raw report committed under `crates/agidb-extract/eval/results/`.

## progress (as of 2026-05-23)

**Plan executed:** see [`docs/superpowers/plans/2026-05-23-phase-3-extraction.md`](../superpowers/plans/2026-05-23-phase-3-extraction.md). **14 of 18 plan tasks complete** — the end-to-end pipeline (text → NER → heuristic relations → temporal → alias → store) works in v1 form. Workspace at HEAD: 87 tests passing, 1 ignored; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean.

What's built (the model-free substrate):

| Component | Where | Tests |
|---|---|---|
| Layer-2 types (`ExtractContext`, `Extraction`, `ExtractedTriple`, `Entity`, `TextExtractor` trait) | `agidb-core/src/types.rs` | 5 |
| `ExtractError` + extraction-crate deps | `agidb-extract/src/error.rs` + `Cargo.toml` | — |
| Predicate canonicalizer | `agidb-extract/src/predicates.rs` | 6 |
| `Store::create_concept` (idempotent on canonical name) | `agidb-core/src/store.rs` | 3 |
| Alias resolver (exact + Levenshtein ≤ 3 + create-on-miss) | `agidb-extract/src/aliases.rs` + 2 new Store helpers | 5 |
| Temporal parser (`yesterday` / `last weekend` / `this weekend` / chrono_english + number-word normalizer) | `agidb-extract/src/temporal.rs` | 6 |
| `ModelRef` constants (SHA-pinned defaults) | `agidb-extract/src/models.rs` | — |
| `ModelManager` (HF download + SHA verify + `AGIDB_OFFLINE` mode) | `agidb-extract/src/model_manager.rs` | 5 |
| `Store::next_episode_id` (monotonic, manifest-persisted) | `agidb-core/src/store.rs` | 3 |
| `observe_text` + `ObserveContext` (text → extract → resolve aliases → mint id → store) | `agidb-extract/src/lib.rs` | 4 |
| `NerExtractor` (gline-rs `GLiNER<SpanMode>` wrapper) | `agidb-extract/src/ner.rs` | smoke gated |
| `heuristic_relations` (v1 PredicateTable-based stub) | `agidb-extract/src/heuristic_relations.rs` | 6 |
| `Extractor` + `ExtractorConfig` (NER + heuristic + temporal + canon; impl `TextExtractor`) | `agidb-extract/src/extractor.rs` | covered via `observe_text` integration test |
| `agidb-extract-eval` binary (load JSONL → run Extractor → P/R/F1 → JSON report; exits 2 if F1 < 0.85) | `crates/agidb-extract/eval/src/main.rs` | dry-run smoke verified |
| Nightly eval CI | `.github/workflows/eval-nightly.yml` | — |

What's not yet built:

| Component | Plan task | Blocker |
|---|---|---|
| ONNX-based relation extractor (replaces `heuristic_relations`) | 10 | port either `ctxgraph-extract/src/glirel.rs` (717 LOC, DeBERTa) or `relex.rs` (501 LOC, gliner-relex-large-v0.5) |
| Real SHA pins for GLINER_DEFAULT + GLINER_TOKENIZER_DEFAULT | 9 cleanup | requires one successful first-run download on a connected machine |
| **100-sample gold dataset** (3-entry placeholder is committed today) | 15 | **human labelling work — not automatable** |
| F1 ≥ 0.85 iteration loop | 18 | needs (10) + (15) + nightly eval runs |

## tasks (as originally planned, with progress)

1. ✅ port the ctxgraph GLiNER loader (`ner.rs`)
2. ⬜ write the gold dataset (3-entry placeholder committed; human work for the full 100)
3. ✅ wire entity extraction (via real GLiNER; smoke test gated)
4. 🟨 add relation extraction — v1 heuristic shipped; v2 needs ONNX port
5. ✅ add time anchor parsing
6. ✅ add alias resolution against the concepts table
7. ✅ add predicate canonicalization
8. ⬜ iterate on the synonym table until F1 ≥ 85% — gated on (2) + (4 v2)

## risks (with mitigations)

| risk | mitigation |
|---|---|
| GLiNER ONNX inference too slow (>200ms) | quantize to int8; cache ORT session; consider distilled smaller model |
| F1 stalls below 85% | augment with predicate-synonym additions; high-recall regex backstop for known-hard entity types; document tradeoff |
| time anchor grammar covers only english | scope: english-only for v0.1, documented in [constitution](../spec/constitution.md) non-goals |
| gold dataset bias | have a second person label 10 of the 100 observations; Cohen's κ ≥ 0.7 gate |
| `gline-rs` upstream API churn | pin exact version in workspace deps; track in `Cargo.lock` |

## what unblocks next

phase 4 needs real extracted triples flowing into the binding step. **Tier B in `recall()` is currently dead** — it activates the moment a real extractor produces canonicalized triples through `observe_text`. The substrate to do this is in place; only the NER + GLiREL wrappers stand between today's state and the exit gate.

## references

- [design spec](../superpowers/specs/2026-05-23-phase-3-extraction-design.md) — decisions, layering, type adjustments
- [implementation plan](../superpowers/plans/2026-05-23-phase-3-extraction.md) — 18-task TDD plan
- [architecture/layer-2-extraction.md](../architecture/layer-2-extraction.md) — pipeline detail
- [spec/tech-spec.md](../spec/tech-spec.md) — `Triple` type, confidence propagation
- port source: `/home/rsx/Desktop/projx/ctxgraph/crates/ctxgraph-extract/` — GLiREL + ner.rs + model_manager.rs
