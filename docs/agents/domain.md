# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

- **`CONTEXT.md`** at the repo root (single-context layout — sochdb is one project, one engine, one workspace)
- **`docs/adr/`** — read ADRs that touch the area you're about to work in
- **`docs/spec/constitution.md`** — the immutable principles (symlinked into `.specify/memory/constitution.md`)
- **`docs/architecture/`** — the three-layer architecture and per-layer details
- **`docs/phases/`** — the per-phase build plan; check the phase you're working in for context

If any of these files don't exist, **proceed silently**. Don't flag their absence; don't suggest creating them upfront. The producer skill (`/grill-with-docs`) creates them lazily when terms or decisions actually get resolved.

## File structure

Single-context repo:

```
/
├── CONTEXT.md
├── docs/
│   ├── adr/
│   │   ├── 0001-license-apache-2.md
│   │   └── 0002-async-runtime-tokio.md
│   ├── spec/constitution.md   ← symlink to .specify/memory/constitution.md
│   ├── architecture/
│   ├── phases/
│   └── product/
└── <workspace crates>/
```

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in `CONTEXT.md`. Don't drift to synonyms the glossary explicitly avoids.

If the concept you need isn't in the glossary yet, that's a signal — either you're inventing language the project doesn't use (reconsider) or there's a real gap (note it for `/grill-with-docs`).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding:

> _Contradicts ADR-0007 (event-sourced orders) — but worth reopening because…_

## Flag constitutional conflicts

If your output contradicts a Core Principle in `.specify/memory/constitution.md`, surface it the same way. The constitution can be amended via ADR — but the amendment is the first move, not the violation.
