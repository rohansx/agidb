# phase 11 — unlearn API

**duration:** weeks 23-25
**status:** not started
**depends on:** [phase 10](./phase-10-sensory-self-model.md)

## goal

deliver non-destructive cascading unlearn with self-vector subtraction and a permanent audit trail, per constitution article XVI. given a target, compute the full dependency cascade, tombstone affected rows, cascade corrections through beliefs and semantic atoms, subtract the unlearned content from the self-vector, and keep a permanent learning-event record.

## deliverables

### week 23

- [ ] add `agidb-core::unlearn` module — types `UnlearnTarget`, `UnlearnReport`, `Tombstone`, cascade-graph computation
- [ ] new redb table: `tombstones`
- [ ] cascade-graph algorithm: given a target (Concept/Episode/Belief/Session/Source), compute the full dependency set across episodes, beliefs, semantic atoms, procedures
- [ ] property test: the cascade-graph correctly identifies all dependents (gold set of 20 hand-traced cascades)

### week 24

- [ ] implement `Agidb::unlearn(target, reason)`:
  1. compute cascade
  2. tombstone all affected rows (set `tombstoned_at`)
  3. invalidate signatures in mmap (mark in slot header)
  4. cascade through beliefs: confidence reduce or withdraw; emit `BeliefRevision`
  5. cascade through semantic atoms: recompute without removed evidence; withdraw if evidence drops below threshold
  6. **self-vector subtraction:** `self_vec ← self_vec - α · bundle(tombstoned_sigs)`; append the corrected snapshot to `self_vector_history`
  7. emit `LearningEvent::Unlearned` (permanent, survives compaction)
- [ ] implement `Agidb::unlearn_report`, `unlearn_history`, `restore_within_window` (30-day recovery)

### week 25

- [ ] extend the bi-temporal filter in `recall()`: tombstoned rows excluded by default; `as_of` queries can still surface them within the 30-day window
- [ ] property tests: unlearn a 100-episode concept → all references gone within 100ms; self-vector hamming distance to the pre-unlearn state matches `α · bundle(tombstoned)`
- [ ] compliance test: simulate a GDPR Article 17 request (BySource unlearn); verify all data gone and the audit log entry permanent
- [ ] MCP + Python expose `unlearn`, `unlearn_history`, `restore_within_window`

## exit criterion

100-episode unlearn completes in ≤100ms. Self-vector verifiably no longer contains the unlearned concept. Audit log permanent. **Phase 11 complete.**

## see also

- [../product/roadmap.md](../product/roadmap.md)
- [../spec/constitution.md](../spec/constitution.md)
- [../architecture/architecture.md](../architecture/architecture.md)
