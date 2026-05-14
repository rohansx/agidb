//! sochdb engine: HDC kernel, redb metadata, mmap'd signatures, tiered recall.
//!
//! Layers:
//! - [`hdc`]        — 8192-bit hypervector type, bind, bundle, hamming (phase 1, GREEN)
//! - [`types`]      — domain types (Episode, Triple, Concept, SemanticAtom, …) (phase 2)
//! - [`error`]      — `SochError` + `Result` (phase 2)
//! - [`signatures`] — mmap'd `signatures.dat` (phase 2)
//! - [`store`]      — redb tables + bi-temporal schema (phase 2)
//! - [`episode`]    — triple binding and episode bundling (phase 4)
//! - [`recall`]     — tiered retrieval A/B/C/D (phase 4; B activates in phase 3)
//! - `consolidate`  — background consolidation worker (phase 6)
//!
//! See `docs/architecture/` and `docs/phases/` for the build plan.

pub mod episode;
pub mod error;
pub mod hdc;
pub mod recall;
pub mod signatures;
pub mod store;
pub mod types;

pub use error::{Result, SochError};
