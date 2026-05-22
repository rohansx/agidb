# ADR-0003: Adopt the agidb v2 constitution

- **Status:** Accepted
- **Date:** 2026-05-22
- **Deciders:** rohan

## Context

sochdb v1 shipped with a 14-article constitution (`.specify/memory/constitution.md`)
governing an embedded, content-addressable HDC memory database. Phases 0, 1, 2, 4,
and 6 of that plan are complete and verified by 44 passing tests.

The project is expanding to **agidb v2** — a *cognitive substrate* for autonomous
agents. The v2 scope adds first-class goals and beliefs, a sensory floor, a
self-model with a self-vector, a non-destructive unlearn primitive, a
neurosymbolic translation layer, and (in v2.1) multimodal sensory encoders with
brain-aligned surprise calibration. The full v2 plan is documented in
[`../product/roadmap.md`](../product/roadmap.md) and the design is in
[`../architecture/architecture.md`](../architecture/architecture.md).

The v1 constitution cannot govern v2 unchanged:

1. It has no articles covering cognitive primitives, unlearn, belief revision, or
   brain-alignment — the load-bearing commitments of the v2 thesis.
2. **Article IV ("No LLM in the Read Path")** as written is satisfiable, but the v2
   belief-revision design needs an LLM *at write time* to judge contradictions.
   Without an explicit amendment, that path would be unconstitutional.
3. Article X (benchmark honesty), Article XII (sacred non-goals), and Article XIII
   (the decision gate) all need extension to cover v2.1 artifacts (BAMS reporting,
   brain-decoding as a non-goal, v2.1 gating).

The repo's own rule is explicit: *"Constitutional changes are not made by code
review. They are made by ADR."* The doc set is being reconciled to v2 in the same
change; the constitution must move with it.

## Decision

**Replace `.specify/memory/constitution.md` with the agidb v2 constitution
(version 2.1, 18 articles).** The change is structured, not wholesale rewrite:

- **Inherited unchanged:** Articles I, II, III, V, VI, VII, VIII, IX, XI, XIV.
- **Article IV — amended:** LLMs are permitted at write time for belief revision
  and consolidation. The read path (`recall`, `what_about`, `between`,
  `recall_procedure`) stays LLM-free. v2.1 clarification: V-JEPA 2 / Wav2Vec-BERT /
  Llama-3.2-3B used as *frozen feature extractors* are not "LLM calls" in the
  constitutional sense.
- **Article X — extended:** the six-metric benchmark honesty rule now also covers
  BAMS reporting.
- **Article XII — extended:** a brain-decoding service is added to the sacred
  non-goals.
- **Article XIII — extended:** v2.1 work is gated on a "Commit" outcome at the
  week-12 decision gate.
- **Articles XV–XVIII — added:** cognitive primitives as first-class typed shapes
  (XV); non-destructive unlearn with permanent audit and self-vector subtraction
  (XVI); belief revision with an explicit append-only revision log (XVII);
  brain-alignment is empirical and additive (XVIII).

The constitution's own *"Amendments since sochdb v1"* section is the canonical,
in-document changelog and is kept current there.

This ADR covers the **constitution and documentation** only. The code rename
(`sochdb-*` crates → `agidb-*`, error types, manifest strings, namespaces) is a
separate, tracked **pre-week-0** task on the roadmap and is *not* enacted here —
the docs now say "agidb"; the crates still say "sochdb" until that task runs.

## Consequences

- The v2 cognitive features (goals, beliefs, unlearn, self-model, brain-alignment)
  are now constitutionally sanctioned rather than out-of-scope.
- The write-time LLM exception is bounded and explicit: it is a documented
  amendment, not a quiet erosion of Article IV. The read path remains provably
  deterministic.
- The constitution is longer (18 articles vs 14); contributors have more to read,
  but the amendment log makes the v1→v2 delta auditable at a glance.
- The constitution version is now **2.1**, last amended 2026-05-20.
- A naming gap is created on purpose: documentation uses "agidb", code uses
  "sochdb". This is tracked and time-boxed by the pre-week-0 rebrand task; it is
  not drift.

## Alternatives considered

- **Keep the v1 14-article constitution; treat v2 features as out-of-scope.**
  Rejected — it directly contradicts the adopted v2 roadmap and would make every
  v2 phase a standing constitutional violation.
- **Amend article-by-article via separate ADRs.** Rejected — the v2 expansion is
  one coherent strategic decision; four added articles plus three extensions
  recorded as one ADR is more honest about the actual unit of decision.
- **Leave Article IV untouched and forbid LLM-assisted belief revision.** Rejected
  — it would force a strictly heuristic contradiction-detection path, which the
  design work judged materially worse, for no constitutional gain (the read-path
  guarantee is preserved either way).

## References

- `.specify/memory/constitution.md` — the v2.1 constitution, including its
  "Amendments since sochdb v1" section.
- [`../product/roadmap.md`](../product/roadmap.md) — the 16-phase v2 plan; the
  pre-week-0 rebrand task.
- [`../architecture/cognitive-primitives.md`](../architecture/cognitive-primitives.md)
  — the design behind Articles XV and XVII.
- [`../architecture/brain-alignment.md`](../architecture/brain-alignment.md) — the
  design behind Article XVIII.
- ADR-0001, ADR-0002 — prior decisions, unaffected by this amendment.
