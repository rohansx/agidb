# sochdb — Claude Code instructions

> sochdb is an embedded, content-addressable memory database for AI agents — Rust top to bottom, single binary, no query language. See [`README.md`](./README.md) for the user-facing pitch, [`docs/README.md`](./docs/README.md) for the doc tree, and [`.specify/memory/constitution.md`](./.specify/memory/constitution.md) for the immutable principles.

## Before changing anything

1. Read the relevant phase doc in [`docs/phases/`](./docs/phases/README.md) — code changes outside the current phase need a justification.
2. Read [`docs/spec/constitution.md`](./docs/spec/constitution.md) — violations require an ADR in `docs/adr/`.
3. Read the relevant architecture layer in [`docs/architecture/`](./docs/architecture/architecture.md) before touching layer 1, 2, or 3 code.

## Agent skills

### Issue tracker

Issues for sochdb live as GitHub issues at `github.com/sochdb/sochdb` (transitional: local markdown under `.scratch/` until the remote is configured). See [`docs/agents/issue-tracker.md`](./docs/agents/issue-tracker.md).

### Triage labels

Default five-role vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See [`docs/agents/triage-labels.md`](./docs/agents/triage-labels.md).

### Domain docs

Single-context layout — one `CONTEXT.md` + `docs/adr/` at the repo root, plus the spec-kit constitution at `.specify/memory/constitution.md`. See [`docs/agents/domain.md`](./docs/agents/domain.md).

## Spec-driven workflow

This project uses GitHub Spec-Kit. The constitution is canonical at [`.specify/memory/constitution.md`](./.specify/memory/constitution.md) (symlinked from `docs/spec/constitution.md`). Slash commands:

- `/speckit-constitution` — revise project principles
- `/speckit-specify` — write a baseline spec for a feature
- `/speckit-clarify` — de-risk ambiguous areas before planning
- `/speckit-plan` — turn a spec into an implementation plan
- `/speckit-tasks` — generate actionable tasks from a plan
- `/speckit-analyze` — cross-artifact consistency check
- `/speckit-checklist` — quality checklist for the spec/plan
- `/speckit-implement` — execute the planned tasks

## House rules

- **Rust top to bottom in `sochdb-core`.** No Python or JavaScript. ONNX runtime via `ort` is the only permitted FFI.
- **No LLM in the read path.** `recall`, `what_about`, `between`, `recall_procedure` must be deterministic.
- **Test-first.** Property tests for HDC algebra, unit tests for each crate, integration tests for the public API. CI runs unit + property on every PR.
- **Benchmark honestly.** Every public claim ships with the full six-metric stack (BLEU + F1 + LLM-judge + token cost + p95 latency + noisy-cue) and raw logs.
- **Commits**: conventional commit prefixes (`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`, `perf:`, `ci:`). No attribution lines (disabled globally).

## When in doubt

Ask. The constitution is binding; the architecture is documented; the phase plan is granular. If something contradicts any of those three, surface it explicitly.

<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan
<!-- SPECKIT END -->
