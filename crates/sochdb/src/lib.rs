//! sochdb — embedded, content-addressable memory database for AI agents.
//!
//! This is the umbrella crate. It re-exports the stable public surface of
//! `sochdb-core` and `sochdb-extract` under one namespace, so users only
//! need a single `sochdb = "0.1"` dependency.
//!
//! See [the README](https://github.com/sochdb/sochdb) and the docs at
//! `docs/README.md` for usage and architecture.

// The public API lands as phases 1-6 produce real types in sochdb-core
// and sochdb-extract. Until then, the re-exports below stay narrow.

#[doc(inline)]
pub use sochdb_core as core;

#[doc(inline)]
pub use sochdb_extract as extract;
