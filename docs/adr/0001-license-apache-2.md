# ADR-0001: License — Apache-2.0

- **Status:** Accepted
- **Date:** 2026-05-14
- **Deciders:** rohan

## Context

sochdb is an open-source library distributed via crates.io, PyPI, and an MCP server binary. It needs a license that:

1. Lets external developers embed it in commercial agent products without contagion clauses
2. Provides a patent grant so contributors and downstream users are protected
3. Matches the licensing posture of other rust embedded-db / agent-memory libraries in the same ecosystem (so dependents face a consistent legal model)
4. Leaves the door open for a separate optional commercial hosted tier in v1.0+ without relicensing the OSS engine

## Decision

License the entire sochdb codebase under **Apache-2.0**, declared in:

- `LICENSE` at the repo root (canonical text from `apache.org`)
- `license = "Apache-2.0"` in `Cargo.toml` `[workspace.package]`, inherited by every member crate via `license.workspace = true`

## Consequences

- Commercial use is unrestricted (no copyleft).
- Contributors automatically grant patent rights to the project.
- The hosted tier (when it ships in v1.0+) can be a separate proprietary product on top of an unchanged Apache-2.0 engine, without dual-licensing or relicensing.
- Apache-2.0 is incompatible with GPLv2-only consumers — sochdb cannot be statically linked into GPLv2-only projects. This is acceptable; the target market is permissive-license agent frameworks.
- We commit to including the `NOTICE` file convention if/when third-party Apache-2.0 code is vendored.

## Alternatives considered

- **MIT** — simpler text, no patent grant. Rejected because the patent grant is a meaningful safety net for an embedded-db project that may attract patent trolls.
- **BSL (Business Source License)** — used by Sentry, MariaDB. Rejected because it complicates downstream use and is unusual in the rust embedded-db ecosystem (redb, lancedb, qdrant are all Apache-2.0 or MIT).
- **AGPL** — strongest copyleft. Rejected because it would discourage adoption by commercial agent products, which is sochdb's primary distribution path.
- **Dual MIT/Apache-2.0** (rust standard for many libs) — slightly more permissive for consumers. Rejected for simplicity; one license is one fewer thing to explain.

## References

- [crates.io: redb](https://crates.io/crates/redb) — Apache-2.0
- [crates.io: lancedb](https://crates.io/crates/lancedb) — Apache-2.0
- [crates.io: qdrant-client](https://crates.io/crates/qdrant-client) — Apache-2.0
- Constitution [article XII](../spec/constitution.md) and [article III](../spec/constitution.md) — the embedded-first posture this license choice supports.
