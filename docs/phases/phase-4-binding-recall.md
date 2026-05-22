# phase 4 — binding + tiered recall

**duration:** weeks — (inherited from sochdb v1)
**status:** complete (inherited from sochdb v1)
**depends on:** [phase 3](./phase-3-extraction.md)

## goal

land the layer-1 read path end to end: bind triples into episode signatures, compute query signatures, run the four-tier retrieval, calibrate confidence per tier.

## deliverables

- [x] `agidb-core/src/episode.rs`:
  - bind triples into role-filler patterns: `triple = (SUBJ⊗s) ⊕ (PRED⊗p) ⊕ (OBJ⊗o)`
  - bundle triples into an episode signature with a time-anchor binding
  - compute a parallel raw-text gist signature for tier C fallback
- [x] `agidb-core/src/recall.rs`:
  - tier A — exact canonical entity match via `concept_index`
  - tier B — HDC similarity over `inverted_index ∩ candidate set`, POPCOUNT scan
  - tier C — raw-text gist signature similarity
  - tier D — nearest-neighbor with `low_confidence=true`
  - tier fall-through with explicit `tier_floor` honored
- [x] inverted-index update path on `observe`
- [x] confidence calibration:
  - tier A: 1.0 - alias_distance / max_distance
  - tier B: hamming-derived (1 - h/D) with linear mapping to [0.6, 0.95]
  - tier C: lower band [0.3, 0.6]
  - tier D: capped at 0.3
- [x] 1,000-episode synthetic dataset generator + recall eval

## exit criterion

recall on the 1,000-episode synthetic dataset returns expected matches with **calibrated confidence (ECE ≤ 0.05) and p95 < 50ms** on the benchmark laptop. raw logs committed.

## tasks

1. write the synthetic dataset generator (1k observations, hand-designed recall queries with ground truth)
2. implement role HVs (`SUBJ_hv`, `PRED_hv`, `OBJ_hv`, `TIME_hv`) — deterministic, fixed at compile time
3. implement triple binding and episode bundling
4. wire the inverted-index update on observe
5. implement tier A (exact entity)
6. implement tier B (HDC similarity with index intersection)
7. implement tier C (gist fallback)
8. implement tier D (nearest neighbor)
9. measure ECE per tier; calibrate the mapping bands
10. measure p95 latency; optimize the index intersection if needed

## risks

| risk | mitigation |
|---|---|
| tier B confidence poorly calibrated | isotonic regression at consolidation time as a corrective; recalibrate per release |
| inverted index intersection too slow at scale | roaring bitmap `Treap`-style early termination; cap candidate set size with random-projection prefiltering |
| query signature for partial cues degrades fast | partial-cue test set in synthetic eval; track tier-B recall@10 specifically for cue-with-2-of-3-roles |
| recall returns dominated by frequent entities | per-entity rate-limit in tier A; bias tier B toward less-common signatures |

## what unblocks next

phase 5 wraps this in MCP and python. without tier A + B + C + D, those wrappers expose nothing useful.

## references

- [architecture/layer-1-recall.md](../architecture/layer-1-recall.md) — the math and the tier definitions
- [spec/tech-spec.md](../spec/tech-spec.md) — `Query`, `Recall`, `Tier`
