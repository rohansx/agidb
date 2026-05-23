//! Predicate canonicalization: surface verbs → a small canonical vocabulary.
//!
//! Curated, not learned. The built-in table is the starting set documented
//! in the phase-3 design spec § 6; custom synonyms are loadable
//! per-deployment by calling [`PredicateTable::add_synonym`].
//!
//! Lookup is case-insensitive on the surface form. Unknown surface
//! returns `None` — callers fall back to the raw surface verb verbatim.

use std::collections::HashMap;

/// Lookup table. Key = lowercased surface form; value = canonical predicate.
#[derive(Debug, Clone)]
pub struct PredicateTable {
    table: HashMap<String, String>,
}

impl PredicateTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    /// Add `surface` as a synonym of `canonical`. Idempotent — re-adding
    /// overwrites the existing mapping.
    pub fn add_synonym(&mut self, canonical: &str, surface: &str) {
        self.table
            .insert(surface.to_lowercase(), canonical.to_string());
    }

    /// Look up `surface`. Returns `None` for unknown forms.
    pub fn lookup(&self, surface: &str) -> Option<String> {
        self.table.get(&surface.to_lowercase()).cloned()
    }
}

impl Default for PredicateTable {
    /// The built-in curated vocabulary. Extend per-deployment by calling
    /// [`PredicateTable::add_synonym`]. Tracked in the phase-3 design
    /// spec § 6.
    fn default() -> Self {
        let mut t = Self::new();
        for s in [
            "recommended",
            "suggested",
            "told me about",
            "pitched",
            "mentioned to me",
        ] {
            t.add_synonym("recommends", s);
        }
        for s in ["in", "based in", "is from", "lives in", "is located in"] {
            t.add_synonym("located_in", s);
        }
        for s in ["works at", "is employed by", "is at", "works for"] {
            t.add_synonym("works_at", s);
        }
        for s in ["likes", "loves", "prefers", "is into", "enjoys"] {
            t.add_synonym("likes", s);
        }
        for s in ["said", "told", "claimed", "mentioned"] {
            t.add_synonym("said", s);
        }
        for s in ["met", "ran into", "saw", "encountered"] {
            t.add_synonym("met", s);
        }
        for s in ["visited", "went to", "stopped by", "dropped in at"] {
            t.add_synonym("visited", s);
        }
        for s in ["owns", "has", "possesses"] {
            t.add_synonym("owns", s);
        }
        // expanded vocabulary (phase-3 v1 polish)
        for s in ["chose", "selected", "picked", "opted for"] {
            t.add_synonym("chose", s);
        }
        for s in ["avoided", "declined", "skipped"] {
            t.add_synonym("avoided", s);
        }
        for s in ["introduced", "presented", "brought up"] {
            t.add_synonym("introduced", s);
        }
        for s in ["arrived at", "got to", "made it to", "reached"] {
            t.add_synonym("arrived_at", s);
        }
        for s in ["wrote", "authored", "drafted", "composed"] {
            t.add_synonym("wrote", s);
        }
        for s in ["discussed", "talked about", "chatted about"] {
            t.add_synonym("discussed", s);
        }
        for s in ["bought", "purchased", "acquired"] {
            t.add_synonym("bought", s);
        }
        for s in ["sold", "traded"] {
            t.add_synonym("sold", s);
        }
        for s in ["founded", "established"] {
            t.add_synonym("founded", s);
        }
        for s in ["knows", "is acquainted with"] {
            t.add_synonym("knows", s);
        }
        for s in ["hired", "recruited"] {
            t.add_synonym("hired", s);
        }
        for s in ["married", "is married to"] {
            t.add_synonym("married", s);
        }
        t.add_synonym("thanked", "thanked");
        t.add_synonym("praised", "praised");
        t
    }
}

/// Convenience: look up a surface predicate. Returns `None` for unknown.
pub fn canonicalize(table: &PredicateTable, surface: &str) -> Option<String> {
    table.lookup(surface)
}
