# phase 12 — neurosymbolic interface

**duration:** weeks 26-27
**status:** not started
**depends on:** [phase 9](./phase-9-cognitive-primitives.md)

## goal

expose the implicit signature↔triple translation as a first-class API and support hybrid queries that blend structured triple-pattern matching with fuzzy HDC similarity. five translation directions become callable, and `neurosymbolic_query` lets callers tune the structured/fuzzy mix.

## deliverables

### week 26

- [ ] add the `agidb-ns` crate (already scaffolded); implement the five translation directions: triple_to_signature, signature_to_triples, cue_to_partial_signature, belief_to_signature, multimodal-factorization stub (full multimodal in phase 14)
- [ ] implement `Agidb::neurosymbolic_query` with `HybridWeights` — combines structured triple-pattern matching with fuzzy HDC similarity
- [ ] default hybrid weights for `recall()`: `{structured: 0.7, fuzzy: 0.3}`

### week 27

- [ ] property tests: bind-then-unbind roundtrip recovers triples with low hamming error; hybrid weights at the extremes (1,0) and (0,1) reduce to pure structured / pure fuzzy
- [ ] MCP + Python expose `neurosymbolic_query`, `signature_to_triples`, `triples_to_signature`
- [ ] docs: `neurosymbolic.md` matches shipped behavior

## exit criterion

hybrid queries with 50/50 weights return appropriately blended results. **Phase 12 complete.**

## see also

- [../product/roadmap.md](../product/roadmap.md)
- [../spec/constitution.md](../spec/constitution.md)
- [../architecture/neurosymbolic.md](../architecture/neurosymbolic.md)
