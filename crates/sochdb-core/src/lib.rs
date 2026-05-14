//! sochdb engine: HDC kernel, redb metadata, mmap'd signatures, tiered recall.
//!
//! Layers:
//! - `hdc`       — 8192-bit hypervector type, bind, bundle, hamming (phase 1)
//! - `store`     — redb tables + bi-temporal schema (phase 2)
//! - `signatures`— mmap'd signatures.dat (phase 2)
//! - `episode`   — triple binding and episode bundling (phase 4)
//! - `recall`    — tiered retrieval (A/B/C/D) (phase 4)
//! - `consolidate` — background consolidation worker (phase 6)
//!
//! See `docs/architecture/` and `docs/phases/` for the build plan.
