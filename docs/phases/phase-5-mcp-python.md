# phase 5 — MCP + python bindings

**duration:** weeks 5-8
**status:** not started
**depends on:** [phase 4](./phase-4-binding-recall.md)

## goal

reach agents and python users. the engine works; now make it installable everywhere it matters.

## deliverables

- [ ] `agidb-mcp/src/main.rs` MCP server exposing:
  - `memory_observe`
  - `memory_recall`
  - `memory_what_about`
  - `memory_between`
  - `memory_consolidate`
  - tool input schemas match [spec/tech-spec.md](../spec/tech-spec.md#the-mcp-server)
- [ ] stdio transport (primary) and streamable-http transport (secondary)
- [ ] claude desktop config example tested end-to-end
- [ ] `agidb-py/src/lib.rs` pyo3 bindings:
  - async via `pyo3-asyncio`
  - all public types mapped (Recall, RecallMatch, Query, ObserveOpts, Provenance, Procedure)
  - errors translate to typed python exceptions
- [ ] python wheels built for linux x86_64/aarch64, macOS x86_64/aarch64, windows x86_64
- [ ] `pip install agidb` published to PyPI test index
- [ ] basic usage examples in `examples/python/` and `examples/mcp/`

## exit criterion

1. claude desktop can use agidb as a memory tool via MCP — manual demo recorded
2. `pip install agidb` works on linux + macOS, smoke test passes

## tasks

1. write the MCP tool schemas to match `Memory` trait surface
2. implement the MCP server with `mcp-rust-sdk` or hand-rolled stdio
3. test against claude desktop; record the demo
4. set up pyo3 + pyo3-asyncio scaffold
5. wrap types one by one with proptests for round-trip equality
6. set up `maturin` for wheel building
7. set up github actions matrix for the wheel build
8. publish to test.pypi.org; verify install
9. write the python README + example notebook

## risks

| risk | mitigation |
|---|---|
| MCP protocol churn | pin the spec version; track breaking changes; ship under feature flag if needed |
| pyo3 async friction | use `pyo3-asyncio` + tokio runtime; mirror lancedb's pattern |
| wheel build matrix CI cost | only build on tagged releases initially; nightly wheels later |
| windows mmap differences | test windows wheel before announcing; document any caveats |

## what unblocks next

phase 6 consolidation is independent of this phase. but phase 7's benchmark harness will use the python bindings, so wheels need to work first.

## references

- [spec/tech-spec.md](../spec/tech-spec.md#the-mcp-server) — exact MCP tool definitions
- [spec/tech-spec.md](../spec/tech-spec.md#python-bindings) — python surface
