# sochdb — phases

> the 6-month build plan, one file per phase. each phase has a single owner, a single exit criterion, and a hard date.

## timeline

| phase | weeks | goal | exit criterion |
|---|---|---|---|
| [phase 0](./phase-0-setup.md) | week 0 | repo, domains, package reservations, CI | `cargo test` runs green on a hello-world crate |
| [phase 1](./phase-1-hdc-kernel.md) | weeks 1-2 | the HDC kernel | 8192-bit hamming scan over 100k sigs in <5ms on M2 |
| [phase 2](./phase-2-storage.md) | weeks 3-4 | redb + mmap + bi-temporal | open → observe → close → reopen → recall by exact match works |
| [phase 3](./phase-3-extraction.md) | weeks 5-6 | GLiNER pipeline | >85% F1 on 20-sample gold set |
| [phase 4](./phase-4-binding-recall.md) | weeks 7-8 | end-to-end tiered recall | 1k-episode synthetic recall with calibrated confidence; p95 <50ms |
| [phase 5](./phase-5-mcp-python.md) | weeks 9-10 | MCP + python | claude desktop uses sochdb via MCP; `pip install sochdb` works |
| [phase 6](./phase-6-consolidation.md) | weeks 11-12 | background consolidation | 10k-episode store shrinks ≥30% in atom count, recall accuracy preserved |
| [phase 7](./phase-7-decision-gate.md) | week 12 | benchmark vs Mem0/Zep/Letta | commit / reposition / retreat decision |
| [phase 8](./phase-8-hardening-launch.md) | weeks 13-26 | harden + launch | public release at week 26 |

## the rule

a phase exits only when its exit criterion is met **on a reproducible benchmark**. partial implementations do not exit a phase. they are tracked but they do not unblock the next phase.

## status

phase 0 has not started. update [phases/README.md](./README.md) and the relevant phase doc as work begins.

## see also

- [../product/roadmap.md](../product/roadmap.md) — the narrative version of this plan with the risk register
- [../spec/constitution.md](../spec/constitution.md) — the immutable principles each phase must honor
