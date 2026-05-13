# sochdb — documentation

> docs are organized into four tracks. start with **product** if you're new, **architecture** if you're integrating, **spec** if you're contributing code, **phases** if you're tracking the build.

## reading paths

| if you are... | read in this order |
|---|---|
| evaluating sochdb as a user | [product/overview](./product/overview.md) → [product/biological-mapping](./product/biological-mapping.md) → [product/roadmap](./product/roadmap.md) |
| building an integration | [product/overview](./product/overview.md) → [architecture/architecture](./architecture/architecture.md) → [spec/tech-spec](./spec/tech-spec.md) |
| contributing to the engine | [spec/constitution](./spec/constitution.md) → [architecture/architecture](./architecture/architecture.md) → the three layer docs → [spec/tech-spec](./spec/tech-spec.md) |
| picking up a build phase | [spec/constitution](./spec/constitution.md) → [phases/README](./phases/README.md) → the phase you own |

## product

what sochdb is, who it's for, where it's going.

- [product/overview.md](./product/overview.md) — the product. the problem, the wedge, the comparisons, the non-goals.
- [product/biological-mapping.md](./product/biological-mapping.md) — how sochdb maps to the five biological memory tiers (sensory, working, episodic, semantic, procedural).
- [product/roadmap.md](./product/roadmap.md) — v0.1 → v1.0 milestones, the week-12 decision gate, and the risk register.

## architecture

how sochdb is built. three engineering layers, one user-facing API.

- [architecture/architecture.md](./architecture/architecture.md) — the three-layer model, write path, read path, consolidation loop.
- [architecture/layer-1-recall.md](./architecture/layer-1-recall.md) — HDC signatures, binding, bundling, tiered retrieval (the mind-like layer).
- [architecture/layer-2-extraction.md](./architecture/layer-2-extraction.md) — GLiNER ONNX entity/relation extraction (the scaffolding).
- [architecture/layer-3-storage.md](./architecture/layer-3-storage.md) — redb + mmap, bi-temporal schema, crash-safety (the plumbing).

## spec

the contract. what the code must do.

- [spec/constitution.md](./spec/constitution.md) — immutable project principles. anything that contradicts the constitution is out of scope.
- [spec/tech-spec.md](./spec/tech-spec.md) — rust API, types, performance targets, dependencies, error model.

## phases

the build plan, one file per phase. each phase has deliverables, exit criteria, dependencies, and risks.

- [phases/README.md](./phases/README.md) — phase index + week-by-week timeline.
- [phases/phase-0-setup.md](./phases/phase-0-setup.md) — week 0: repo, domains, package reservations, CI.
- [phases/phase-1-hdc-kernel.md](./phases/phase-1-hdc-kernel.md) — weeks 1-2: the HDC kernel.
- [phases/phase-2-storage.md](./phases/phase-2-storage.md) — weeks 3-4: redb + mmap + bi-temporal.
- [phases/phase-3-extraction.md](./phases/phase-3-extraction.md) — weeks 5-6: GLiNER pipeline.
- [phases/phase-4-binding-recall.md](./phases/phase-4-binding-recall.md) — weeks 7-8: tiered recall end-to-end.
- [phases/phase-5-mcp-python.md](./phases/phase-5-mcp-python.md) — weeks 9-10: MCP server + python bindings.
- [phases/phase-6-consolidation.md](./phases/phase-6-consolidation.md) — weeks 11-12: background consolidation.
- [phases/phase-7-decision-gate.md](./phases/phase-7-decision-gate.md) — week 12: benchmarks vs Mem0, Zep, Letta. commit / reposition / retreat.
- [phases/phase-8-hardening-launch.md](./phases/phase-8-hardening-launch.md) — weeks 13-26: hardening, whitepaper, design partners, public launch.

## conventions

- file names are kebab-case lowercase
- relative links only — no absolute URLs to this repo
- diagrams: ASCII first, mermaid second, no images
- every claim with a number cites a source inline
- benchmarks always publish raw logs alongside summary numbers
