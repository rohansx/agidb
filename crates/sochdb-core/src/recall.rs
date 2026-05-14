//! Tiered recall — the layer-1 read path.
//!
//! Implements [`Store::recall`] as the four-tier cascade documented in
//! `docs/architecture/layer-1-recall.md`:
//!
//! - **Tier A — Exact**: cue tokens looked up in the concept index;
//!   any matching episode is returned with confidence 1.0.
//! - **Tier B — Similarity**: structured HDC signature similarity.
//!   *Skipped in phase 4* — needs phase-3 extraction to produce a
//!   structured cue signature; the cascade currently goes A → C → D.
//! - **Tier C — Gist**: gist HV (token bundle) similarity against
//!   every episode, similarity floor 0.55, confidence band [0.3, 0.6].
//! - **Tier D — NearestNeighbor**: top-k by gist similarity regardless
//!   of threshold, confidence capped at 0.3.
//!
//! Per [constitution](../../.specify/memory/constitution.md) article VI,
//! recall never returns the empty set under the default `tier_floor`.

use crate::episode::{encode_gist_signature, encode_query_signature, tokenize};
use crate::error::{Result, SochError};
use crate::store::{Store, EPISODES, SEMANTIC_ATOMS};
use crate::types::*;
use redb::ReadableTable;
use std::collections::HashSet;
use std::time::Instant;

/// Tier-C similarity floor. Two random HVs have expected similarity
/// ≈ 0.5; the floor sits a few percent above that to keep noise out
/// of the high-confidence band.
const TIER_C_SIM_FLOOR: f32 = 0.55;

/// Linear map ranges for confidence calibration.
const TIER_C_BAND: (f32, f32) = (0.3, 0.6);
const TIER_D_CAP: f32 = 0.3;

impl Store {
    /// Run a recall against the store. Per the constitution, never
    /// returns an empty `Recall::matches` under the default `tier_floor`
    /// of `NearestNeighbor`.
    ///
    /// `Recall::semantic_atoms` also carries any consolidated atoms
    /// whose anchoring concept matches a cue token — this is how phase
    /// 6 surfaces consolidated knowledge alongside raw episodes.
    pub fn recall(&self, query: &Query) -> Result<Recall> {
        let started = Instant::now();
        let matches = self.run_cascade(query)?;
        let semantic_atoms = self.semantic_atoms_for_cue(query)?;
        let tier_used = matches
            .iter()
            .map(|m| m.source_tier)
            .min_by_key(|t| t.depth())
            .unwrap_or(Tier::NearestNeighbor);
        let elapsed_ms = started.elapsed().as_millis().min(u32::MAX as u128) as u32;
        Ok(Recall {
            matches,
            semantic_atoms,
            tier_used,
            elapsed_ms,
        })
    }

    /// Look up every `SemanticAtom` whose anchoring concept matches a
    /// token in the cue. O(N) over atoms today; a concept→atoms
    /// inverted index is a phase-6 follow-up.
    fn semantic_atoms_for_cue(&self, query: &Query) -> Result<Vec<SemanticMatch>> {
        let mut wanted: HashSet<ConceptId> = HashSet::new();
        for token in tokenize(&query.cue) {
            if let Some(cid) = self.concept_id_for(&token)? {
                wanted.insert(cid);
            }
        }
        if wanted.is_empty() {
            return Ok(Vec::new());
        }
        let tx = self.db.begin_read()?;
        let table = tx.open_table(SEMANTIC_ATOMS)?;
        let mut out = Vec::new();
        for entry in table.iter()? {
            let (_, v) = entry?;
            let bytes = v.value();
            let atom: SemanticAtom = bincode::deserialize(&bytes)
                .map_err(|e| SochError::Internal(format!("decode atom: {e}")))?;
            if wanted.contains(&atom.concept) {
                out.push(SemanticMatch::from(atom));
            }
        }
        Ok(out)
    }

    fn run_cascade(&self, query: &Query) -> Result<Vec<RecallMatch>> {
        // Tier A — exact concept lookup
        if Tier::Exact.depth() <= query.tier_floor.depth() {
            let a = self.tier_a_exact(query)?;
            if !a.is_empty() {
                return Ok(self.finalize(a, query));
            }
        }

        // Tier B — structured similarity. Skipped in phase 4 (needs
        // phase-3 extraction to produce a structured cue signature).
        // The cascade falls straight through to C.

        // Tier C — gist similarity in the high-confidence band
        if Tier::Gist.depth() <= query.tier_floor.depth() {
            let scored = self.scan_with_gist(query)?;
            let c = self.tier_c_matches(&scored, query);
            if !c.is_empty() {
                return Ok(self.finalize(c, query));
            }

            // Tier D — nearest neighbor (no threshold, low confidence)
            if Tier::NearestNeighbor.depth() <= query.tier_floor.depth() {
                let d = self.tier_d_matches(&scored, query);
                return Ok(self.finalize(d, query));
            }
        }

        Ok(vec![])
    }

    fn tier_a_exact(&self, query: &Query) -> Result<Vec<RecallMatch>> {
        let mut out = Vec::new();
        let mut seen: HashSet<EpisodeId> = HashSet::new();
        for token in tokenize(&query.cue) {
            let Some(cid) = self.concept_id_for(&token)? else {
                continue;
            };
            for ep in self.recall_exact(cid, query.as_of)? {
                if seen.insert(ep.id) {
                    out.push(into_match(ep, 1.0, Tier::Exact));
                }
            }
        }
        Ok(out)
    }

    /// Scan every episode, compute gist similarity to the query, and
    /// return a list sorted by similarity descending.
    ///
    /// O(N) over the store. Phase 4 lives with this; tier-B activation
    /// in phase 3 replaces the full scan with an inverted-index
    /// intersection for the high-confidence band.
    fn scan_with_gist(&self, query: &Query) -> Result<Vec<(f32, Episode)>> {
        let query_hv = encode_query_signature(&query.cue);
        let tx = self.db.begin_read()?;
        let table = tx.open_table(EPISODES)?;

        let mut scored: Vec<(f32, Episode)> = Vec::new();
        for entry in table.iter()? {
            let (_, v) = entry?;
            let bytes = v.value();
            let ep: Episode = bincode::deserialize(&bytes)
                .map_err(|e| SochError::Internal(format!("decode episode: {e}")))?;
            if let Some(t) = query.as_of {
                if !ep.valid_time.contains(t) {
                    continue;
                }
            }
            let gist = encode_gist_signature(&ep.text);
            let sim = query_hv.similarity(&gist);
            scored.push((sim, ep));
        }
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored)
    }

    fn tier_c_matches(&self, scored: &[(f32, Episode)], query: &Query) -> Vec<RecallMatch> {
        scored
            .iter()
            .filter(|(s, _)| *s >= TIER_C_SIM_FLOOR)
            .take(query.k)
            .map(|(s, ep)| {
                let confidence = calibrate_band(*s, TIER_C_SIM_FLOOR, 1.0, TIER_C_BAND);
                into_match(ep.clone(), confidence, Tier::Gist)
            })
            .collect()
    }

    fn tier_d_matches(&self, scored: &[(f32, Episode)], query: &Query) -> Vec<RecallMatch> {
        scored
            .iter()
            .take(query.k)
            .map(|(s, ep)| {
                let confidence = (s * TIER_D_CAP).clamp(0.0, TIER_D_CAP);
                into_match(ep.clone(), confidence, Tier::NearestNeighbor)
            })
            .collect()
    }

    /// Sort by confidence descending, apply `min_confidence`, and
    /// truncate to `k`. Final shape returned to the caller.
    fn finalize(&self, mut matches: Vec<RecallMatch>, query: &Query) -> Vec<RecallMatch> {
        matches.retain(|m| m.confidence >= query.min_confidence);
        matches.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matches.truncate(query.k);
        matches
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn into_match(ep: Episode, confidence: f32, tier: Tier) -> RecallMatch {
    RecallMatch {
        episode_id: ep.id,
        text: ep.text,
        triples: ep.triples,
        confidence,
        valid_time: ep.valid_time,
        provenance: ep.provenance,
        superseded: ep.superseded_by.is_some(),
        source_tier: tier,
    }
}

/// Linearly map a similarity score from `[sim_lo, sim_hi]` into the
/// confidence band `(conf_lo, conf_hi)`. Values outside `[sim_lo, sim_hi]`
/// are clamped to the corresponding band edge.
fn calibrate_band(sim: f32, sim_lo: f32, sim_hi: f32, band: (f32, f32)) -> f32 {
    if sim <= sim_lo {
        return band.0;
    }
    if sim >= sim_hi {
        return band.1;
    }
    let t = (sim - sim_lo) / (sim_hi - sim_lo);
    band.0 + t * (band.1 - band.0)
}
