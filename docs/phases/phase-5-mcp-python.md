# phase 5 — MCP + python bindings

**duration:** weeks 5-8
**status:** MCP server scaffold in progress; Python bindings not started
**depends on:** [phase 4](./phase-4-binding-recall.md)

## goal

reach agents and python users. the engine works; now make it installable everywhere it matters.

## deliverables

- [x] `agidb-mcp/src/{main,lib,protocol,context,server,tools}.rs` MCP server skeleton exposing:
  - [x] `memory_observe` — text → stored Episode (via `observe_text` + alias resolver)
  - [x] `memory_recall` — tiered cascade against Store
  - [ ] `memory_what_about` — needs `Store::what_about(ConceptId)` first (not in agidb-core yet)
  - [ ] `memory_between` — needs `Store::between(start, end)` first (not in agidb-core yet)
  - [x] `memory_consolidate` — drives `Store::consolidate`
  - [x] **bonus:** `memory_get_episode` (cheap, useful for verification)
  - [x] tool input schemas inline in `tools.rs` (JSON-Schema)
- [x] stdio transport (primary) — line-delimited JSON-RPC 2.0; smoke-verified by piping `initialize` + `tools/list` into the binary
- [ ] streamable-http transport (secondary) — defer until there's a real client requiring it
- [ ] claude desktop config example tested end-to-end — manual; awaits the user
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

## progress (as of 2026-05-23)

**MCP server scaffold landed.** 4 of 5 originally-planned tools wired (observe, recall, consolidate, get_episode); `what_about` and `between` deferred until the underlying `Store` methods exist in `agidb-core`. The server:

- Reads line-delimited JSON-RPC 2.0 from stdin, writes responses to stdout (logs go to stderr).
- Implements `initialize`, `tools/list`, `tools/call`, `ping`, `notifications/initialized`.
- Falls back from a real `Extractor` to `NullExtractor` if model cache is cold so the server starts on machines without GLiNER weights — `observe` then stores text-only episodes.
- 7 dispatch tests via `handle_request` (no stdio, no models). Workspace at HEAD: 98 tests green, clippy + fmt clean.

What's left for phase 5 to exit:
- `Store::what_about(ConceptId)` + `Store::between(t0, t1)` in `agidb-core` + wire the two remaining MCP tools.
- Streamable-HTTP transport (defer unless a real client demands it).
- Claude Desktop smoke test (manual, your machine).
- The full python-bindings half: `agidb-py` pyo3 + maturin + wheel matrix CI + PyPI test index.

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
