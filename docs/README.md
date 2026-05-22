# agidb — documentation

> **New here? Read [PROJECT.md](./PROJECT.md) first** — the single-document master
> reference covering vision, current status, architecture, and the full roadmap.
>
> docs are organized into four tracks. start with **product** if you're new, **architecture** if you're integrating, **spec** if you're contributing code, **phases** if you're tracking the build.

## reading paths

| if you are... | read in this order |
|---|---|
| evaluating agidb as a user | [product/overview](./product/overview.md) → [product/biological-mapping](./product/biological-mapping.md) → [product/roadmap](./product/roadmap.md) |
| building an integration | [product/overview](./product/overview.md) → [architecture/architecture](./architecture/architecture.md) → [spec/tech-spec](./spec/tech-spec.md) |
| contributing to the engine | [spec/constitution](./spec/constitution.md) → [architecture/architecture](./architecture/architecture.md) → the layer docs → [spec/tech-spec](./spec/tech-spec.md) |
| picking up a build phase | [spec/constitution](./spec/constitution.md) → [phases/README](./phases/README.md) → the phase you own |
| tracking the AGI thesis | [product/overview](./product/overview.md) → [architecture/cognitive-primitives](./architecture/cognitive-primitives.md) → [product/agi-trajectory](./product/agi-trajectory.md) |

## product

what agidb is, who it's for, where it's going.

- [product/overview.md](./product/overview.md) — the product. the problem, the wedge, the comparisons, the non-goals.
- [product/biological-mapping.md](./product/biological-mapping.md) — the seven cognitive floors mapped to the cognitive-psychology and neuroscience literature.
- [product/roadmap.md](./product/roadmap.md) — the 16-phase, 12-month plan: v2.0 substrate (month 9), v2.1 brain-alignment (month 12), the week-12 decision gate, and the risk register.
- [product/agi-trajectory.md](./product/agi-trajectory.md) — the 5-year shape: why a cognitive substrate is the path toward AGI-grade infrastructure.

## architecture

how agidb is built. three engineering layers, seven cognitive floors, one user-facing API.

- [architecture/architecture.md](./architecture/architecture.md) — the three-layer model, seven floors, write path, read path, consolidation loop, unlearn loop.
- [architecture/layer-1-recall.md](./architecture/layer-1-recall.md) — HDC signatures, binding, bundling, tiered retrieval (the mind-like layer).
- [architecture/layer-2-extraction.md](./architecture/layer-2-extraction.md) — GLiNER ONNX extraction; v2.1 multimodal encoders (the scaffolding).
- [architecture/layer-3-storage.md](./architecture/layer-3-storage.md) — redb + mmap, bi-temporal schema, append-only audit logs, crash-safety (the plumbing).
- [architecture/cognitive-primitives.md](./architecture/cognitive-primitives.md) — goals and beliefs as first-class typed shapes; the seven-floor cognitive semantics.
- [architecture/neurosymbolic.md](./architecture/neurosymbolic.md) — the signature↔triple translation layer and hybrid structured/fuzzy queries.
- [architecture/brain-alignment.md](./architecture/brain-alignment.md) — v2.1: the TRIBE-derived brain-alignment thesis and surprise calibration.
- [architecture/bams-benchmark.md](./architecture/bams-benchmark.md) — v2.1: the brain-aligned memory similarity benchmark protocol.

## spec

the contract. what the code must do.

- [spec/constitution.md](./spec/constitution.md) — the 18 immutable principles. anything that contradicts the constitution is out of scope. canonical at `.specify/memory/constitution.md`.
- [spec/tech-spec.md](./spec/tech-spec.md) — the full Rust API, types, performance targets, dependencies, error model (v2.0 + v2.1).

## phases

the build plan, one file per phase. each phase has deliverables, exit criteria, and dependencies.

- [phases/README.md](./phases/README.md) — phase index + 16-phase timeline.
- **inherited (sochdb v1, complete):** phases 0–2, 4, 6 — setup, HDC kernel, storage, binding+recall, consolidation.
- **v2.0 critical path:** phases 3, 5, 7–13 — extraction, MCP+Python, decision gate, hardening, cognitive primitives, sensory+self-model, unlearn, neurosymbolic, cognitive benchmarks.
- **v2.1 (gated on the decision gate):** phases 14–16 — multimodal sensory, brain-calibrated surprise, BAMS benchmark + ICLR paper.

## adr

- [adr/README.md](./adr/README.md) — architecture decision records: the dated *why* behind non-trivial choices.

## conventions

- file names are kebab-case lowercase
- relative links only — no absolute URLs to this repo
- diagrams: ASCII first, mermaid second, no images
- every claim with a number cites a source inline
- benchmarks always publish raw logs alongside summary numbers
