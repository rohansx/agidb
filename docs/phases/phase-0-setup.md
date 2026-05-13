# phase 0 — setup

**duration:** week 0
**status:** not started
**owner:** rohan

## goal

stand up the project skeleton so phase 1 can start without administrative friction.

## deliverables

- [ ] `sochdb.com` and `sochdb.dev` domains registered
- [ ] github org `sochdb` created
- [ ] `sochdb/sochdb` repo initialized
- [ ] `sochdb` reserved on crates.io
- [ ] `sochdb` reserved on PyPI
- [ ] workspace `Cargo.toml` with member crates:
  - `sochdb-core`
  - `sochdb-extract`
  - `sochdb-cli`
  - `sochdb-mcp`
  - `sochdb-py`
  - `sochdb-bench`
- [ ] github actions CI: `cargo test`, `cargo clippy`, `cargo fmt --check`
- [ ] license file (Apache-2.0)
- [ ] this docs tree committed
- [ ] `CONTRIBUTING.md` with the [constitution](../spec/constitution.md) linked at the top

## exit criterion

`cargo test --workspace` passes on a hello-world crate in CI on linux + macOS.

## dependencies

none. this phase is the starting line.

## risks

| risk | mitigation |
|---|---|
| crates.io name collision | check before announcing; have backup names ready (`sochdb-rs`, `sochmem`) |
| domain squatting on `sochdb.com` | buy on day 0; use `sochdb.dev` as primary if `.com` is gone |
| pypi name collision | same — check first, reserve fast |

## decisions to record as ADRs

- [ ] license choice: Apache-2.0 (rationale: matches lancedb, qdrant, redb; permits enterprise relicensing later)
- [ ] minimum supported rust version: pin in `rust-toolchain.toml`
- [ ] async runtime: tokio (per [constitution](../spec/constitution.md) article 8)

## what unblocks next

phase 1 needs the workspace structure and CI green. nothing more.
