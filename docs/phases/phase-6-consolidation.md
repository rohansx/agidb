# phase 6 — consolidation

**duration:** weeks — (inherited from sochdb v1)
**status:** complete (inherited from sochdb v1)
**depends on:** [phase 4](./phase-4-binding-recall.md) (phase 5 can run in parallel)

## goal

land the analog of sleep: a background tokio task that clusters episodes, creates semantic atoms, detects contradictions, decays unused memory, and compacts storage. the db manages its own working set.

## deliverables

- [x] `agidb-core/src/consolidate.rs` with five steps:
  1. **cluster** — scan recent episodic signatures (last 7 days), cluster by hamming distance
  2. **semantic atom creation** — clusters with N ≥ 3 episodes bundle into semantic atoms with evidence_count + provenance back to source episodes
  3. **contradiction detection** — same (subject, predicate), overlapping valid time, different object → newer fact gets `t_valid_start=now`, older gets `t_valid_end=now-1ms` and `superseded_by`
  4. **decay** — unreferenced atoms (no recall hits in 90 days) decay by factor λ; archive below floor
  5. **compact** — rewrite signatures.dat to remove archived entries; update offsets
- [x] consolidation log + audit trail in `consolidation_log` redb table
- [x] `Agidb::consolidate()` synchronous API returning `ConsolidationReport`
- [x] background task scheduler (default: every 5 minutes when idle)
- [x] 10k-episode synthetic dataset with known redundancy patterns

## exit criterion

consolidation reduces a 10k-episode store by **≥ 30% in semantic-atom count without losing recall accuracy** (recall@10 on the synthetic eval set unchanged). raw logs committed.

## tasks

1. write the 10k-episode synthetic generator with controlled redundancy
2. implement clustering (hamming-distance threshold; tunable)
3. implement semantic atom creation with provenance links
4. implement contradiction detection over the bi-temporal index
5. implement decay function (λ tunable; default `exp(-Δt / τ)` with τ=90 days)
6. implement compaction (in-place rewrite with safe offset updates)
7. wire the background tokio task with low priority and pause-on-busy
8. add the consolidation log
9. measure: scan time, atom reduction, recall preservation
10. tune until exit criterion met

## risks

| risk | mitigation |
|---|---|
| consolidation corrupts offsets during compact | dual-buffer rewrite; only swap atomically after verification |
| background task starves foreground reads | nice-priority via tokio task budget; pause when active queries >threshold |
| contradiction detection produces false positives | strict-mode toggle; emit `ConsolidationReport.contradictions_flagged` for review |
| decay too aggressive — loses useful memory | conservative λ default; recall-hit telemetry feeds back into refresh |
| 10k episodes too small to expose scaling issues | follow-up at phase 8 with 100k + 1M synthetic loads |

## what unblocks next

phase 7 benchmarks include consolidation behavior (Mem0 has implicit consolidation; we need a fair comparison). without phase 6, our long-context numbers will be worse than they need to be.

## references

- [architecture/architecture.md](../architecture/architecture.md#the-consolidation-loop) — the loop diagram
- [product/biological-mapping.md](../product/biological-mapping.md) — why consolidation is the McClelland-McNaughton-O'Reilly model in code
