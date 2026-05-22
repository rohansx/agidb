# agidb — phases

> the agidb v2 build plan, one file per phase. each phase has a single owner, a single exit criterion, and a hard date.

## timeline

| # | phase | weeks | status |
|---|---|---|---|
| 0 | [setup](./phase-0-setup.md) | — | ✅ complete (inherited from sochdb v1) |
| 1 | [HDC kernel](./phase-1-hdc-kernel.md) | — | ✅ complete (inherited) |
| 2 | [storage](./phase-2-storage.md) | — | ✅ complete (inherited) |
| 3 | [extraction (GLiNER)](./phase-3-extraction.md) | 1-4 | ⬜ not started — v2.0 critical |
| 4 | [binding + recall](./phase-4-binding-recall.md) | — | ✅ complete (inherited) |
| 5 | [MCP + Python](./phase-5-mcp-python.md) | 5-8 | ⬜ not started — v2.0 critical |
| 6 | [consolidation](./phase-6-consolidation.md) | — | ✅ complete (inherited) |
| 7 | [decision gate](./phase-7-decision-gate.md) | 11-13 | ⬜ not started — **binding** |
| 8 | [hardening + launch](./phase-8-hardening-launch.md) | 31-36 | ⬜ not started — v2.0 ship |
| 9 | [cognitive primitives](./phase-9-cognitive-primitives.md) | 13-18 | ⬜ not started — v2.0 wedge |
| 10 | [sensory + self-model](./phase-10-sensory-self-model.md) | 19-22 | ⬜ not started |
| 11 | [unlearn API](./phase-11-unlearn.md) | 23-25 | ⬜ not started |
| 12 | [neurosymbolic interface](./phase-12-neurosymbolic.md) | 26-27 | ⬜ not started |
| 13 | [cognitive benchmarks](./phase-13-cognitive-benchmarks.md) | 28-30 | ⬜ not started |
| 14 | [multimodal sensory](./phase-14-multimodal-sensory.md) | 37-42 | ⬜ not started — v2.1 (gated) |
| 15 | [brain-calibrated surprise](./phase-15-brain-calibrated-surprise.md) | 43-46 | ⬜ not started — v2.1 (gated) |
| 16 | [BAMS benchmark + ICLR paper](./phase-16-bams-benchmark.md) | 47-52 | ⬜ not started — v2.1 (gated) |

## the rule

a phase exits only when its exit criterion is met **on a reproducible benchmark**. partial implementations do not exit a phase. they are tracked but they do not unblock the next phase.

## status

phases 0, 1, 2, 4, and 6 are complete — inherited from sochdb v1 and verified by 44 passing tests.

phases 3, 5, and 7–16 are not started.

weeks 9-10 are a benchmark-harness build that is phase-7 prep, not a separate phase.

note: "weeks" count from the agidb v2 kickoff. a pre-week-0 rebrand (sochdb→agidb) precedes week 1 — namespace lock, crate renames, and the GitHub org move happen before the week-counter starts.

## see also

- [../product/roadmap.md](../product/roadmap.md) — the narrative version of this plan with the risk register
- [../spec/constitution.md](../spec/constitution.md) — the immutable principles each phase must honor
