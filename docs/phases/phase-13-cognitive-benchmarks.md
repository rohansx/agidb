# phase 13 — cognitive benchmarks

**duration:** weeks 28-30
**status:** not started
**depends on:** [phase 12](./phase-12-neurosymbolic.md)

## goal

build the four cognitive benchmarks no other system can run on itself — goal consistency, belief revision, unlearn cascade, and multi-floor retrieval — run them against agidb and the incumbent baselines, and write up the results for the eventual v2.0 launch paper.

## deliverables

### week 28

- [ ] build the `agidb-bench::cognitive` module with four benchmark suites:
  - **goal consistency:** 50 simulated agent sessions with goal trees of depth 3; verify the state machine never violates invariants
  - **belief revision:** 50 sequences of (assertion, contradiction, re-assertion) with known correct revision history; verify agidb's audit log matches
  - **unlearn cascade:** 30 GDPR-style requests; verify cascading removal completes correctly and the self-vector reflects the subtraction
  - **multi-floor retrieval:** 50 queries requiring information from 2+ floors (e.g. "what did Sarah say about my current goal?"); verify recall returns matches grounded across floors

### week 29

- [ ] run the benchmarks against agidb; document thresholds: goal consistency ≥99%, belief revision audit ≥95% match, unlearn cascade ≥99%, multi-floor retrieval F1 ≥80%
- [ ] comparison baselines (where applicable): run goal consistency + belief revision against mem0/letta/zep — most will score near 0% because they don't have these primitives; that's the point

### week 30

- [ ] write the cognitive benchmark whitepaper section (becomes part of the eventual v2.0 launch arxiv paper)
- [ ] integrate cognitive benchmarks into CI: every PR runs goal consistency + multi-floor retrieval as smoke tests

## exit criterion

all four cognitive benchmarks pass agidb thresholds. **Phase 13 complete.**

## see also

- [../product/roadmap.md](../product/roadmap.md)
- [../spec/constitution.md](../spec/constitution.md)
- [../architecture/bams-benchmark.md](../architecture/bams-benchmark.md)
