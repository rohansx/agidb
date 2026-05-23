# phase 3 — extraction

**duration:** weeks 1-4
**status:** in progress — substrate done; models + 100-sample gold pending
**depends on:** [phase 2](./phase-2-storage.md)

## goal

land layer 2: replace the phase-2 regex placeholder with a real GLiNER ONNX pipeline that extracts entities, relations, and time anchors with calibrated confidence.

## deliverables

- [ ] vendor / port the GLiNER ONNX loading and inference code from ctxgraph (port skeleton in [`crates/agidb-extract/src/model_manager.rs`](../../crates/agidb-extract/src/model_manager.rs); NER/GLiREL wrappers gated on real model access)
- [ ] `agidb-extract/src/lib.rs` with the `Extraction` pipeline:
  - [ ] entity extraction with type labels (Person, Place, Organization, etc.) — NER wrapper deferred to model-access session
  - [ ] relation extraction producing `(subject, predicate, object)` triples — GLiREL port deferred
  - [x] time anchor parsing via chrono + small grammar ("last weekend", "yesterday", "in 2024") — see [`temporal.rs`](../../crates/agidb-extract/src/temporal.rs)
  - [ ] confidence scoring from GLiNER logits propagated to triples — depends on NER wrapper
- [x] alias resolution and concept canonicalization (uses `concepts` table from phase 2) — [`aliases.rs`](../../crates/agidb-extract/src/aliases.rs)
- [x] predicate canonicalization with a hand-curated synonym table (`recommended` ≡ `suggested` ≡ `told me about`) — [`predicates.rs`](../../crates/agidb-extract/src/predicates.rs)
- [ ] **100-sample** gold dataset committed to `crates/agidb-extract/eval/gold/observations.jsonl` (revised up from 20 during the brainstorming session — see the design spec for rationale)
- [ ] eval script that computes F1 against gold — scaffold deferred to plan tasks 14 + 16

## exit criterion

`observe_text()` correctly extracts triples from **100 sample observations** with **F1 ≥ 0.85** against the human-labelled gold set. Measured by `cargo run -p agidb-extract-eval --release`; raw report committed under `crates/agidb-extract/eval/results/`.

## progress (as of 2026-05-23)

**Plan executed:** see [`docs/superpowers/plans/2026-05-23-phase-3-extraction.md`](../superpowers/plans/2026-05-23-phase-3-extraction.md). 10 of 18 plan tasks complete on the substrate. Workspace at HEAD: 81 tests passing, 1 ignored; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean.

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

What's not yet built:

| Component | Plan task | Blocker |
|---|---|---|
| `NerExtractor` (gline-rs wrapper) | 9 | needs `gline-rs` API verification at port time + ~hundreds of MB GLiNER ONNX download |
| `RelationExtractor` (GLiREL port from ctxgraph) | 10 | needs the port from `ctxgraph-extract/src/glirel.rs` + a working GLiREL ONNX repo |
| `Extractor` struct (orchestrates NER + GLiREL + temporal + alias + canon) | 11 | depends on 9 + 10. **The `TextExtractor` trait is already in place**, so when the real `Extractor` lands it `impl`s the trait and `observe_text` accepts it with zero changes. |
| `agidb-extract-eval` sub-crate + binary | 14, 16 | trivial scaffold; depends on 11 for real scoring |
| **100-sample gold dataset** | 15 | **human labelling work — not automatable** |
| Nightly CI workflow | 17 | depends on 14–16 |
| F1 ≥ 0.85 iteration loop | 18 | the actual phase-3 exit gate |

## tasks (as originally planned, with progress)

1. ~~port the ctxgraph GLiNER loader; verify model loads on linux + macOS~~ — deferred to model-access session
2. ~~write the gold dataset first (100 observations, hand-labeled triples)~~ — human task, deferred
3. ~~wire entity extraction; measure F1~~ — gated on (1)
4. ~~add relation extraction; measure F1~~ — gated on (1)
5. ✅ add time anchor parsing
6. ✅ add alias resolution against the concepts table
7. ✅ add predicate canonicalization
8. ~~iterate on the synonym table until F1 ≥ 85%~~ — gated on (1) + (2)

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
