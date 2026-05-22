# phase 0 — setup

**duration:** weeks — (inherited from sochdb v1)
**status:** complete (inherited from sochdb v1)
**owner:** rohan

## goal

stand up the project skeleton so phase 1 can start without administrative friction.

## deliverables

- [x] `agidb.com` and `agidb.dev` domains registered
- [x] github org `agidb` created
- [x] `agidb/agidb` repo initialized
- [x] `agidb` reserved on crates.io
- [x] `agidb` reserved on PyPI
- [x] workspace `Cargo.toml` with member crates:
  - `agidb-core`
  - `agidb-extract`
  - `agidb-cli`
  - `agidb-mcp`
  - `agidb-py`
  - `agidb-bench`
- [x] github actions CI: `cargo test`, `cargo clippy`, `cargo fmt --check`
- [x] license file (Apache-2.0)
- [x] this docs tree committed
- [x] `CONTRIBUTING.md` with the [constitution](../spec/constitution.md) linked at the top

## exit criterion

`cargo test --workspace` passes on a hello-world crate in CI on linux + macOS.

## dependencies

none. this phase is the starting line.

## risks

| risk | mitigation |
|---|---|
| crates.io name collision | check before announcing; have backup names ready (`agidb-rs`, `agimem`) |
| domain squatting on `agidb.com` | buy on day 0; use `agidb.dev` as primary if `.com` is gone |
| pypi name collision | same — check first, reserve fast |

## decisions to record as ADRs

- [ ] license choice: Apache-2.0 (rationale: matches lancedb, qdrant, redb; permits enterprise relicensing later)
- [ ] minimum supported rust version: pin in `rust-toolchain.toml`
- [ ] async runtime: tokio (per [constitution](../spec/constitution.md) article 8)

## what unblocks next

phase 1 needs the workspace structure and CI green. nothing more.
