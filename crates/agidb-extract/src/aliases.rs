//! Exact + fuzzy alias resolution against the Store's concepts table.
//!
//! Strategy:
//!   1. Exact (case-folded) → existing ConceptId.
//!   2. Levenshtein <= 3 → unique-candidate match.
//!   3. Otherwise mint a new Concept with the NER-derived `entity_type`.
//!
//! Fuzzy-match conflict resolution is deliberately conservative: if more
//! than one candidate is within distance, we DO NOT merge — we create a
//! new concept rather than guess. The phase-3 design spec § 7 records
//! this choice.

use agidb_core::store::Store;
use agidb_core::types::ConceptId;
use agidb_core::Result;

const FUZZY_THRESHOLD: usize = 3;

#[derive(Default)]
pub struct AliasResolver;

impl AliasResolver {
    pub fn new() -> Self {
        Self
    }

    /// Resolve a surface `mention` to a `ConceptId`. Mints one when no
    /// exact / unique-fuzzy match is found, using `kind` as the new
    /// concept's `entity_type`.
    pub fn resolve(&self, store: &mut Store, mention: &str, kind: &str) -> Result<ConceptId> {
        let folded = mention.to_lowercase();
        if let Some(id) = store.concept_id_for_ci(&folded)? {
            return Ok(id);
        }
        let candidates = store.fuzzy_concept_candidates(&folded, FUZZY_THRESHOLD)?;
        if candidates.len() == 1 {
            return Ok(candidates[0]);
        }
        store.create_concept(mention, kind)
    }
}
