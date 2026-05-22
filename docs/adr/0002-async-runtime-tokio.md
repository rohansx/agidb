# ADR-0002: Async runtime — tokio

- **Status:** Accepted
- **Date:** 2026-05-14
- **Deciders:** rohan

## Context

agidb's public API is async (`async fn observe`, `async fn recall`, `async fn consolidate`, etc.) per the `Memory` trait in [`docs/spec/tech-spec.md`](../spec/tech-spec.md). The async runtime choice cascades into every dependency — tokio-compatible crates are not always async-std-compatible and vice versa.

The candidates:

1. **tokio** — de facto standard, used by hyper, axum, redb's async helpers, pyo3-asyncio, mcp-rust-sdk
2. **async-std** — alternative runtime, smaller ecosystem
3. **smol** — minimal runtime, popular in embedded contexts
4. **runtime-agnostic** (avoid choosing) — possible via traits but adds friction to every consumer

## Decision

**Use tokio as the canonical async runtime across all agidb crates.** Configure it via:

```toml
tokio = { version = "1", features = ["full"] }
```

at the workspace level. All member crates depend on `tokio` via the workspace inheritance pattern.

## Consequences

- Users of agidb must run inside a tokio runtime. This is the dominant rust async pattern in 2026; effectively no friction for the target audience.
- The MCP server (`agidb-mcp`) inherits tokio for the MCP protocol async handlers — matches `mcp-rust-sdk` expectations.
- The python bindings (`agidb-py`) use `pyo3-asyncio` with the tokio runtime, the most-tested combination.
- The benchmark harness (`agidb-bench`) can use tokio's `JoinSet` for parallel baseline runs.
- We lose the option to be runtime-agnostic. If a future consumer needs async-std, we'll need a compatibility shim — accepted cost.

## Alternatives considered

- **async-std** — smaller ecosystem, less compatibility with the MCP and pyo3 worlds. Rejected.
- **smol** — minimal, but adds maintenance overhead in a project that won't benefit from its smallness (we're already shipping ONNX runtime; smol's binary size savings are noise).
- **Runtime-agnostic via traits** — adds API friction to every consumer, complicates the `Memory` trait, with no concrete benefit at this scale.
- **No async (sync API)** — clean but cuts off MCP, pyo3-async, and concurrent observe/recall composition that real agent frameworks need.

## References

- Constitution [article VIII](../spec/constitution.md) — "Forbidden: ... async-std and other tokio competitors" — this ADR is the formal record of that choice.
- [`docs/spec/tech-spec.md`](../spec/tech-spec.md) § Dependencies — lists tokio as the canonical runtime.
- [tokio.rs](https://tokio.rs) — official docs.
