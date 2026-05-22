//! agidb — embedded, content-addressable memory database for AI agents.
//!
//! This is the umbrella crate. It re-exports the stable public surface of
//! `agidb-core` and `agidb-extract` under one namespace, so users only
//! need a single `agidb = "0.1"` dependency.
//!
//! See [the README](https://github.com/agidb/agidb) and the docs at
//! `docs/README.md` for usage and architecture.

// The public API lands as phases 1-6 produce real types in agidb-core
// and agidb-extract. Until then, the re-exports below stay narrow.

#[doc(inline)]
pub use agidb_core as core;

#[doc(inline)]
pub use agidb_extract as extract;
