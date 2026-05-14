//! Episode-level HDC encoding.
//!
//! Turns extracted triples + raw text into the hypervectors the recall
//! tiers compare against. Symmetric across the write and read paths —
//! `observe()` calls these to compute the stored signature, `recall()`
//! calls the same functions on the cue.
//!
//! Role HVs are derived from fixed strings via [`HV::from_name`]. Once
//! computed they are stable across builds, processes, and machines, so
//! a sochdb file written on one host opens correctly on another.

use crate::hdc::HV;
use crate::types::Triple;
use chrono::{DateTime, Utc};

// ---------------------------------------------------------------------------
// Role hypervectors
// ---------------------------------------------------------------------------

/// HV representing the `subject` slot of a triple.
pub fn role_subj() -> HV {
    HV::from_name("__SOCH_ROLE_SUBJECT__")
}

/// HV representing the `predicate` slot of a triple.
pub fn role_pred() -> HV {
    HV::from_name("__SOCH_ROLE_PREDICATE__")
}

/// HV representing the `object` slot of a triple.
pub fn role_obj() -> HV {
    HV::from_name("__SOCH_ROLE_OBJECT__")
}

/// HV representing the time-anchor slot of an episode.
pub fn role_time() -> HV {
    HV::from_name("__SOCH_ROLE_TIME__")
}

// ---------------------------------------------------------------------------
// Triple + episode encoding
// ---------------------------------------------------------------------------

/// Bind a triple into a single hypervector:
/// `(SUBJ ⊗ subject) ⊕ (PRED ⊗ predicate) ⊕ (OBJ ⊗ object)`.
///
/// The result is uncorrelated to any of its operands but recovers each
/// filler when unbound with the corresponding role HV.
pub fn bind_triple(triple: &Triple) -> HV {
    let s = HV::from_name(&triple.subject);
    let p = HV::from_name(&triple.predicate);
    let o = HV::from_name(&triple.object);
    let subj_bound = role_subj().bind(&s);
    let pred_bound = role_pred().bind(&p);
    let obj_bound = role_obj().bind(&o);
    HV::bundle(&[subj_bound, pred_bound, obj_bound])
}

/// Encode a full episode signature: bundle every triple-binding,
/// optionally adding a time-anchor binding.
///
/// Empty input returns [`HV::zero`] so the function is total over all
/// inputs and `observe()` can always proceed.
pub fn encode_episode_signature(triples: &[Triple], time: Option<DateTime<Utc>>) -> HV {
    let mut bound: Vec<HV> = triples.iter().map(bind_triple).collect();
    if let Some(t) = time {
        let time_hv = HV::from_name(&t.format("%Y-%m-%d").to_string());
        bound.push(role_time().bind(&time_hv));
    }
    if bound.is_empty() {
        HV::zero()
    } else {
        HV::bundle(&bound)
    }
}

/// Compute the gist signature for raw text.
///
/// Tokenization is case-folded so "Sarah" and "sarah" produce the same
/// gist HV. This is the fallback signature for tiers C and D when no
/// structured extraction is available (phase 4) or when the structured
/// path fails to match (phase 5+).
pub fn encode_gist_signature(text: &str) -> HV {
    let tokens = tokenize_lower(text);
    if tokens.is_empty() {
        return HV::zero();
    }
    let hvs: Vec<HV> = tokens.iter().map(|t| HV::from_name(t)).collect();
    HV::bundle(&hvs)
}

/// Compute the query signature for a recall cue. Same algorithm as
/// [`encode_gist_signature`] today; phase 3 replaces this with the
/// structured extraction path so tier B becomes substantive.
pub fn encode_query_signature(cue: &str) -> HV {
    encode_gist_signature(cue)
}

// ---------------------------------------------------------------------------
// Tokenization
// ---------------------------------------------------------------------------

/// Case-preserving alphanumeric tokenizer. Splits on every non-alnum
/// character (whitespace, punctuation) and drops empty segments.
/// Tier-A concept lookup uses this form so canonical-case names like
/// "Sarah" survive intact.
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Lower-cased tokenizer. Tier-C/D gist signatures use this form so
/// "Bawri", "BAWRI", and "bawri" all collapse to the same HV.
pub fn tokenize_lower(text: &str) -> Vec<String> {
    tokenize(&text.to_lowercase())
}
