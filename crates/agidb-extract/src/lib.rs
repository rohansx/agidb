//! Layer 2 — extraction.
//!
//! Turns raw text into structured triples, canonicalized entities, and
//! parsed time anchors. Wraps `gline-rs` (NER) + a ported GLiREL
//! relation extractor + a ported chrono_english-based temporal parser.
//! Built so the agidb-core engine stays extraction-blind: callers go
//! through `Extractor::extract` directly or through the `observe_text`
//! free function (added in plan task 12).
//!
//! Layered per the phase-3 design:
//! `docs/superpowers/specs/2026-05-23-phase-3-extraction-design.md`.

pub mod aliases;
pub mod error;
pub mod model_manager;
pub mod models;
pub mod predicates;
pub mod temporal;

// The modules below are introduced by later plan tasks. Each task
// uncomments its module declaration as it lands.
//
// pub mod ner;            — plan task 9
// pub mod glirel;         — plan task 10
// pub mod extractor;      — plan task 11
//
// `observe_text` and `ObserveContext` are added to this file in plan
// task 12 once `extractor` lands.

pub use crate::error::{ExtractError, Result};
