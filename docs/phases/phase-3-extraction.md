# phase 3 — extraction

**duration:** weeks 1-4
**status:** not started
**depends on:** [phase 2](./phase-2-storage.md)

## goal

land layer 2: replace the phase-2 regex placeholder with a real GLiNER ONNX pipeline that extracts entities, relations, and time anchors with calibrated confidence.

## deliverables

- [ ] vendor / port the GLiNER ONNX loading and inference code from ctxgraph
- [ ] `agidb-extract/src/lib.rs` with the `Extraction` pipeline:
  - entity extraction with type labels (Person, Place, Organization, etc.)
  - relation extraction producing `(subject, predicate, object)` triples
  - time anchor parsing via chrono + small grammar ("last weekend", "yesterday", "in 2024")
  - confidence scoring from GLiNER logits propagated to triples
- [ ] alias resolution and concept canonicalization (uses `concepts` table from phase 2)
- [ ] predicate canonicalization with a hand-curated synonym table (`recommended` ≡ `suggested` ≡ `told me about`)
- [ ] 20-sample gold dataset committed to `eval/gold/observations.jsonl`
- [ ] eval script that computes F1 against gold

## exit criterion

`observe()` correctly extracts triples from 20 sample observations with **>85% F1** against the human-labeled gold set. measured by the eval script in CI.

## tasks

1. port the ctxgraph GLiNER loader; verify model loads on linux + macOS
2. write the gold dataset first (20 observations, hand-labeled triples)
3. wire entity extraction; measure F1 — should already be reasonable from GLiNER alone
4. add relation extraction; measure F1
5. add time anchor parsing
6. add alias resolution against the concepts table
7. add predicate canonicalization
8. iterate on the synonym table until F1 ≥ 85%

## risks

| risk | mitigation |
|---|---|
| GLiNER ONNX inference too slow (>200ms) | quantize to int8; cache ORT session; consider distilled smaller model |
| F1 stalls below 85% | augment with regex/BM25 backstop for high-recall low-precision entities; document tradeoff |
| time anchor grammar covers only english | scope: english-only for v0.1, document explicitly in [constitution](../spec/constitution.md) non-goals |
| gold dataset bias | have a second person label 5 of the 20 observations and compute inter-annotator agreement |

## what unblocks next

phase 4 needs real extracted triples flowing into the binding step. without phase 3 the signatures encode noise.

## references

- [architecture/layer-2-extraction.md](../architecture/layer-2-extraction.md) — pipeline detail
- [spec/tech-spec.md](../spec/tech-spec.md) — `Triple` type, confidence propagation
