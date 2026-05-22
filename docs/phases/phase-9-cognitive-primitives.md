# phase 9 — cognitive primitives (goals + beliefs)

**duration:** weeks 13-18
**status:** not started
**depends on:** [phase 7](./phase-7-decision-gate.md) (gated on the "Commit" decision), [phase 3](./phase-3-extraction.md)

## goal

make `Goal` and `Belief` first-class typed shapes with state machines, revision audit, and HDC signatures — the wedge no other agent memory system has. goals get state-machine transition validation and goal-biased retrieval; beliefs get Bayesian-style confidence updates, an append-only revision log, and LLM-assisted contradiction judgement at write time.

## deliverables

### week 13

- [ ] add `agidb-core::goal` module — types `Goal`, `GoalState`, `GoalPatch`, `GoalTree`, `SuccessCriterion`, plus a state-machine transition validator
- [ ] add `agidb-core::belief` module — types `Belief`, `BeliefRevision`, `Evidence`, `RevisionReport`
- [ ] two new redb tables: `goals`, `beliefs`; migration code opens a v2.0 db without these tables and creates them empty
- [ ] property tests: goal state machine invariants (Completed/Abandoned are terminal; pause/resume preserves history)

### week 14

- [ ] implement `Agidb::set_goal`, `revise_goal`, `complete_goal`, `abandon_goal`, `active_goals`, `goal_tree`, `get_goal`
- [ ] goal HDC signature derivation: bind description tokens with parent context
- [ ] add `belief_revisions` redb table (third v2.0 table this phase)
- [ ] implement `Agidb::assert_belief`, `revise_belief`, `what_do_i_believe`, `belief_history`, `withdraw_belief`

### week 15

- [ ] belief revision math: Bayesian-style confidence update on new evidence; append `BeliefRevision` to the log on every change
- [ ] LLM-assisted revision (constitution article IV amendment): when evidence is ambiguous, call an LLM at write time to judge contradiction — structured prompt to structured `RevisionDecision`; document supported LLMs (Claude, GPT, local Llama via Ollama)
- [ ] withdraw belief on confidence drop below 0.5 (configurable)
- [ ] 100-step goal-mutation property test: a random walk through goal state machines never violates invariants

### week 16

- [ ] wire goal-biased retrieval into `recall()` — active goals' HDC signatures up-weight related episode matches by `goal_bias_weight * similarity(episode_sig, goal_sig)`
- [ ] add `Recall::active_goals` and `Recall::goal_biased` fields
- [ ] extend the MCP server with goal/belief tools: `set_goal`, `revise_goal`, `assert_belief`, `revise_belief`, `what_do_i_believe`, `active_goals`
- [ ] extend Python bindings with the same

### week 17

- [ ] belief context in recall results: populate `Recall::beliefs` with beliefs about the queried subject
- [ ] concept-level belief lookups: `what_do_i_believe(ConceptId)` fast (indexed by belief.subject)
- [ ] property test: the belief revision log captures every change; replaying the log reconstructs current confidence

### week 18

- [ ] integration test: 20-turn agent simulation where goals get set/revised/completed and beliefs get asserted/revised/withdrawn; verify final state matches expected
- [ ] benchmark: `set_goal` ≤ 5ms, `assert_belief` ≤ 5ms, `revise_belief` ≤ 50ms (LLM-assisted path can be slower)
- [ ] docs update: `cognitive-primitives.md` matches shipped behavior

## exit criterion

100-step goal mutation test passes. Belief revision audit log captures every change. Goal-biased retrieval working. **Phase 9 complete.**

## see also

- [../product/roadmap.md](../product/roadmap.md)
- [../spec/constitution.md](../spec/constitution.md)
- [../architecture/cognitive-primitives.md](../architecture/cognitive-primitives.md)
