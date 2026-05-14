# Contributing to sochdb

> Before anything else, read [`.specify/memory/constitution.md`](./.specify/memory/constitution.md). Every Core Principle is binding; violations require an ADR amendment, not a workaround.

## Quick links

- [Constitution](./.specify/memory/constitution.md) — the immutable principles
- [Architecture](./docs/architecture/architecture.md) — three-layer model + write/read/consolidation paths
- [Tech spec](./docs/spec/tech-spec.md) — public API, types, performance targets
- [Phase plan](./docs/phases/README.md) — week-by-week build with exit criteria
- [ADRs](./docs/adr/README.md) — durable record of architectural decisions

## Setup

```bash
git clone https://github.com/sochdb/sochdb.git
cd sochdb
cargo build --workspace          # builds all seven crates
cargo test --workspace           # runs unit + property tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

Required toolchain: stable Rust 1.89+ (auto-installed from `rust-toolchain.toml`). MSRV is enforced via clippy's `incompatible_msrv` lint.

## Development workflow

### 1. Pick a phase

Every change lands inside one of the phases documented in [`docs/phases/`](./docs/phases/README.md). If your change doesn't fit any phase, write an ADR proposing how the roadmap should change before writing code.

### 2. Test-first (NON-NEGOTIABLE)

Per [constitution article on testing](./.specify/memory/constitution.md), all new behavior begins with a failing test:

```bash
# Red — write a property test in tests/<feature>_properties.rs
# It must compile and fail. Confirm:
cargo test -p sochdb-core <test_name>   # expect FAILED

# Green — implement the minimum to pass
cargo test -p sochdb-core <test_name>   # expect ok

# Refactor — clean up, keep tests green
cargo clippy --workspace --all-targets -- -D warnings
```

Property tests (`proptest`) cover algebraic invariants (e.g. HDC binding is self-inverse). Unit tests cover the deterministic surface. Integration tests cover the public API end-to-end. CI runs all three on every PR — bench is gated to manual runs.

### 3. Honest benchmarking

Every public performance or accuracy claim ships with **all six metrics** — BLEU + F1 + LLM-judge (binary) + token cost + p95 latency + noisy-cue degradation — plus raw logs and the harness commit hash. Single-number claims are rejected. See [`docs/spec/tech-spec.md` § benchmark reporting contract](./docs/spec/tech-spec.md).

### 4. Conventional commits

```
<type>(<scope>): <subject>
<blank>
<body explaining the why>
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`, `ci`.

Scope is optional but useful: `feat(sochdb-core): …`, `docs(phases): …`, `chore(ci): …`.

Subject line ≤ 70 chars. Body explains *why*, not *what* — the diff already shows the what.

Co-author trailers are disabled globally. Don't add them.

### 5. Architectural changes need an ADR

If your change touches any of the Core Principles in the constitution — adding a runtime that isn't tokio, adding a query language, adding LLM calls to the read path — open an ADR first under `docs/adr/NNNN-<kebab-title>.md`. See the [ADR template](./docs/adr/README.md#template).

## What lives where

| Surface | Path |
|---|---|
| The constitution | `.specify/memory/constitution.md` |
| Project plan | `docs/phases/` |
| Architecture | `docs/architecture/` |
| Public API spec | `docs/spec/tech-spec.md` |
| Glossary | `CONTEXT.md` |
| Decision records | `docs/adr/` |
| Engine code | `crates/sochdb-core/src/` |
| Extraction code | `crates/sochdb-extract/src/` |
| MCP server | `crates/sochdb-mcp/src/` |
| Python bindings | `crates/sochdb-py/src/` |
| CLI | `crates/sochdb-cli/src/` |
| Benchmark harness | `crates/sochdb-bench/src/` |
| Property tests | `crates/<crate>/tests/` |
| Benches | `crates/<crate>/benches/` |
| Per-repo agent config | `docs/agents/` |
| CI | `.github/workflows/` |

## Spec-driven workflow (optional but encouraged)

For non-trivial features, use the [spec-kit](https://github.com/github/spec-kit) flow:

```
/speckit-specify <one-line description>      # draft spec under .specify/specs/NNN-<slug>/
/speckit-clarify                              # de-risk ambiguous areas
/speckit-plan                                 # turn spec into implementation plan
/speckit-tasks                                # generate actionable tasks
/speckit-implement                            # execute the plan
```

The spec under `.specify/specs/NNN-*/spec.md` is the durable record of *what* a feature does. The phase doc (e.g. `docs/phases/phase-2-storage.md`) is the durable record of *when* and *why*. Both should reference each other.

## Asking questions

- For design questions: open a discussion or ping the maintainers in the issue tracker. Tag with `needs-triage`.
- For bugs: open an issue with a minimal reproduction. Tag with `needs-triage`.
- For "is this a good ADR?" questions: open the ADR PR with `[draft]` in the title — discussion happens there.

## License

By contributing, you agree your contributions are licensed under Apache-2.0 (see [LICENSE](./LICENSE) and [ADR-0001](./docs/adr/0001-license-apache-2.md)).
