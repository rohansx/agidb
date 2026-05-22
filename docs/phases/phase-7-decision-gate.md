# phase 7 — decision gate

**duration:** weeks 11-13
**status:** not started
**depends on:** [phase 6](./phase-6-consolidation.md), [phase 5](./phase-5-mcp-python.md)

## goal

run agidb head-to-head against Mem0, Zep/Graphiti, and Letta on a shared harness. publish raw logs. decide whether to **commit**, **reposition**, or **retreat**.

this is the most important phase. the entire 6-month bet collapses to one week of benchmarks.

## the benchmark suite

three benchmarks, no cherry-picking:

| benchmark | what it tests | Q count | source |
|---|---|---|---|
| **LongMemEval-S** | long-context memory accuracy on episodic recall | ~500 questions | Wu et al., 2024 |
| **LoCoMo** | long conversation memory across 10+ sessions | ~9,000 questions | Maharana et al., 2024 |
| **BEAM** | contradiction resolution, instruction following, scale to millions of tokens | mem0-published harness | Mem0, 2026 |

## the metric stack

every benchmark publishes **all five** metrics — never a single number:

1. **BLEU** — surface-form match (lower bound, conservative)
2. **F1** — token overlap (industry standard)
3. **LLM-judge (binary)** — semantic correctness, judged by a held-out LLM
4. **token cost** — total tokens spent per query (prompt + completion)
5. **p95 latency** — end-to-end recall latency
6. **noisy-cue degradation test** — accuracy when 20% of cue tokens are corrupted; tests graceful degradation

all raw logs ship with each metric. harness commit hash recorded. baseline systems run from pinned versions.

## deliverables

- [ ] `agidb-bench` workspace crate with:
  - LongMemEval-S harness
  - LoCoMo harness
  - BEAM harness
  - noisy-cue harness
- [ ] adapters for Mem0, Zep/Graphiti, Letta (use their published clients; pin versions in `bench/lockfile.toml`)
- [ ] LLM-judge prompt + judge model pinned (Claude Sonnet 4.6 or equivalent; recorded)
- [ ] runbook: `cargo run --bin bench-all -- --output bench/results/phase-7/`
- [ ] published raw logs + summary table
- [ ] decision memo: `bench/results/phase-7/decision.md`

## decision thresholds

### commit threshold — proceed to launch

**all three must hold:**

- agidb ≥ Zep/Graphiti accuracy on LongMemEval-S (F1 within 1pp **and** LLM-judge within 1pp)
- agidb ≥ 3× lower p95 recall latency than Mem0
- agidb ≥ 3× lower token cost than Mem0 (target: < 2,500 tokens/query against Mem0's ~7k)
- agidb wins on the noisy-cue degradation test (graceful tier-C/D fallback is the whole point)

if met:
- proceed to [phase 8](./phase-8-hardening-launch.md) (hardening + launch)
- begin investor conversations (Lightspeed India, Accel, Peak XV, 100x)
- file the YC summer batch application

### reposition threshold — ship smaller

agidb within 3pp of Mem0 accuracy on LongMemEval-S **and** ≥ 10× memory footprint savings.

if met but commit threshold isn't:
- reposition as "embedded memory for edge agents"
- ship anyway, smaller positioning, no fundraise
- continue to v0.2 with refined focus

### retreat threshold — fold back into ctxgraph

agidb more than 10pp behind dense baselines on LongMemEval-S **and** the gap doesn't close with reranking.

if hit:
- drop the HDC bet
- reposition as "Graphiti without Neo4j"
- merge agidb learnings back into ctxgraph
- continue Utkrusht day job full time

## the rule

the gate is binding. it is not negotiable. the decision is recorded in `bench/results/phase-7/decision.md` and signed off before phase 8 starts.

## risks

| risk | mitigation |
|---|---|
| benchmark dispute (the zep-vs-mem0 LoCoMo pattern) | publish every metric, every raw log, every harness commit; invite reproduction |
| Mem0 ships an update mid-benchmark | pin versions in `lockfile.toml`; run all baselines on the same day |
| LLM-judge bias toward Mem0 | use a held-out judge model the bench team didn't tune for; rerun with 2 judges |
| timeline slips into week 13-14 | acceptable — gate quality > gate speed |

## references

- [product/roadmap.md](../product/roadmap.md) — the narrative version of this decision
- [spec/constitution.md](../spec/constitution.md) — article 10 (benchmark honestly) + article 13 (gate is binding)
