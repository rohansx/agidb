# phase 8 — hardening + launch

**duration:** weeks 31-36
**status:** not started
**depends on:** [phase 7](./phase-7-decision-gate.md) (commit threshold met)

## goal

go from "benchmark-credible engine" to "public v0.1 release with design partners and a credible whitepaper."

this phase only runs if phase 7's commit threshold was met. if reposition, see the reposition track below.

## deliverables

### code hardening

- [ ] expand benchmark harness coverage to all of LoCoMo (not just the sampled subset)
- [ ] fuzz suite over `observe`, `recall`, `consolidate` (`cargo-fuzz`)
- [ ] long-running soak test: 30-day continuous observe + recall + consolidate, no leaks, no corruption
- [ ] cross-platform CI: linux x86_64, linux aarch64, macOS x86_64, macOS aarch64, windows x86_64
- [ ] semver discipline established: 0.1.x patches, 0.2 for breaking changes only
- [ ] `manifest.toml` migration path tested across format versions

### writing

- [ ] arxiv whitepaper, target 12-15 pages
  - venue: NeurIPS workshop or ICLR system track
  - audience: ML researchers + systems engineers
  - sections: problem framing, HDC primer, agidb architecture, evaluation, limitations
- [ ] launch blog post on agidb.dev
- [ ] one-pager PDF for investor conversations
- [ ] 60-second demo video (terminal recording + voiceover)

### distribution

- [ ] crates.io publish of `agidb`, `agidb-core`, `agidb-extract`, `agidb-mcp`
- [ ] PyPI publish of `agidb` (wheel matrix)
- [ ] MCP registry submission
- [ ] `cargo binstall agidb` smoke test (single-binary install path)
- [ ] homebrew formula for `agidb-mcp` (later if time)

### design partners

- [ ] identify 5-10 candidate teams building agent products
- [ ] convert 3 to confirmed design partners with weekly check-ins
- [ ] each partner has a real production use case, not a toy
- [ ] feedback loop: design-partner issues land on github with `partner` label

### launch motion

- [ ] HN Show post draft, scheduled for tuesday 8am ET
- [ ] Product Hunt submission
- [ ] X / twitter thread + threadly version
- [ ] linkedin post for the indian / SEA founder audience
- [ ] outreach list of 50 agent-framework maintainers; personal email to each
- [ ] tracker board for response rates

## target metrics at launch (week 26)

- **500+ GitHub stars in week 1**
- **3+ confirmed design-partner deployments**
- **benchmarks reproducible by external developers** (verified: one external dev runs the harness end-to-end before launch)
- **documentation tested by 3 external readers** (have them walk through "build a memory-enabled agent in 30 minutes"; capture friction)

## the reposition track

if phase 7 hit reposition threshold (not commit):

- ship v0.1 anyway with the smaller "embedded memory for edge agents" positioning
- skip the investor outreach
- skip the YC application
- focus on edge / on-device use cases (mobile agents, desktop assistants)
- continue to v0.2 with refined focus
- timeline: 3 months not 14 weeks

## risks

| risk | mitigation |
|---|---|
| solo-dev burnout | scope cut: defer non-critical hardening; ship v0.1 with `experimental` flag where appropriate |
| design partners drop out before launch | overcommit at 5; expect 2-3 to convert |
| whitepaper rejected | submit as preprint regardless; venue is a nice-to-have |
| HN launch is quiet | have 3 backup launch surfaces (Product Hunt, dev.to, indiehackers) |
| competitive announcement during launch week | move the launch by 1 week if Mem0/Zep ships something specifically targeting our wedge |

## what's deferred to v0.2

- encryption at rest
- learned predicate similarity
- LSH over signatures (for >1M episodes)
- batch observe
- WAL streaming

see [product/roadmap.md](../product/roadmap.md#v02--the-consolidation-release-months-7-9) for the v0.2 plan.

## references

- [product/roadmap.md](../product/roadmap.md) — the v0.2 → v1.0 horizon
- [spec/constitution.md](../spec/constitution.md) — what cannot change during launch
