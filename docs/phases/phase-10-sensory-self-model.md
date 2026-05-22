# phase 10 — sensory + self-model

**duration:** weeks 19-22
**status:** not started
**depends on:** [phase 9](./phase-9-cognitive-primitives.md)

## goal

build floor 1 (a sensory ring buffer with surprise gating) and floor 7 (a learning event log plus a self-vector EMA). sensory frames with high surprise auto-promote to episodic memory; every state-changing operation emits a learning event; an 8192-bit self-vector drifts with each consolidation pass.

## deliverables

### week 19

- [ ] add `agidb-core::sensory` module — types `SensoryFrame`, `SensoryData`, `Modality`, ring-buffer logic
- [ ] new redb table: `sensory_buffer` (with ring-eviction semantics)
- [ ] implement `Agidb::observe_sensory`, `working_state`, `surprise_score`
- [ ] surprise computation: `1 - similarity(new_sig, bundle_of(recent_beliefs))`

### week 20

- [ ] surprise-gated promotion: sensory frames with `surprise > threshold` (default 0.4) auto-promote to episodic via an internal `observe()` call
- [ ] add `agidb-core::learning_log` module; new redb table: `learning_events`
- [ ] implement the `LearningEvent` enum (closed set per constitution XV implication); emit events from every state-changing operation across the engine

### week 21

- [ ] implement `Agidb::what_did_i_learn(since)` — query the learning log
- [ ] add `attention_trace` recording to the recall path: when `query.trace_attention = true`, build an `AttentionTrace` and emit it to the learning log
- [ ] implement `Agidb::attention_trace(recall_id)` lookup

### week 22

- [ ] self-vector implementation; new redb table: `self_vector_history` (originally scheduled for v2.1, brought forward into v2.0 because phase 11's unlearn needs it) — 8192-bit HV, EMA update on each consolidate pass: `self_vec ← (1-α) self_vec + α bundle(consolidated_atoms)`
- [ ] implement `Agidb::self_vector`, `self_vector_at(time)`, `self_vector_history`
- [ ] wire the self-vector update into the consolidation worker (extends phase 6 code)
- [ ] benchmark: sensory ingest 1000 frames/sec, surprise gating promotes ~5%, learning log writes don't bottleneck observe

## exit criterion

sensory buffer ingests at target rate. Surprise gating promotes only the novel. Learning log captures every state change. Self-vector drifts with consolidation. **Phase 10 complete.**

## see also

- [../product/roadmap.md](../product/roadmap.md)
- [../spec/constitution.md](../spec/constitution.md)
- [../architecture/cognitive-primitives.md](../architecture/cognitive-primitives.md)
- [../architecture/architecture.md](../architecture/architecture.md)
