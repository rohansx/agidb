//! Phase 10/11 — the self-vector (floor 7).
//!
//! A slowly-drifting 8192-bit HV representing "what kind of agent am I
//! right now." EMA-updated each consolidation epoch toward the bundle of
//! newly-consolidated atom signatures; *subtracted* from on unlearn so
//! forgotten content leaves no centroid contamination (constitution
//! article XVI extension).
//!
//! Math (binary HDC, per CONTEXT.md):
//! - **EMA add** (consolidation): weighted per-bit majority.
//!   `new_bit = 1 if (1-α)·old + α·bundle > 0.5 else 0`, `α ≈ 0.05`.
//! - **Subtract** (unlearn): XOR the self-vector with the bundle of
//!   tombstoned signatures, then the result is the new self-vector.
//!   This flips the bits the unlearned content set, removing its
//!   contribution. Verified by a drop in
//!   `similarity(self_vec, unlearned_concept_sig)`.

use chrono::{DateTime, Utc};
use redb::{ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::hdc::{D, HV};
use crate::store::{decode, encode, Store, MANIFEST};

/// Timestamped snapshots of the self-vector over time. Keyed by a
/// monotonic seq; the *current* self-vector lives in the manifest for
/// O(1) access.
pub const SELF_VECTOR_HISTORY: TableDefinition<u64, Vec<u8>> = TableDefinition::new("self_vector_history");

/// Manifest key for the current self-vector (1024 bytes).
const KEY_SELF_VECTOR: &str = "self_vector";

/// Manifest key for the self-vector history seq counter.
const KEY_SELF_VECTOR_SEQ: &str = "self_vector_seq";

/// EMA weight for consolidation updates. α=0.5 gives visible drift on
/// each pass (half old, half new); production tuning should lower this
/// toward 0.05 for slower drift. The math uses ±1 representation so
/// bits actually flip across the 0 threshold.
pub const SELF_VECTOR_ALPHA: f32 = 0.5;

/// A timestamped snapshot of the self-vector.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SelfVectorSnapshot {
    pub hv: HV,
    pub at: DateTime<Utc>,
    pub seq: u64,
}

// ---------------------------------------------------------------------------
// Binary HDC math.
// ---------------------------------------------------------------------------

/// EMA update: drift `current` toward `bundle` by weight `alpha`.
/// Uses ±1 representation (bit 0 → -1, bit 1 → +1) so bits flip across
/// the zero threshold: `new_val = (1-α)*old ± α*bundle`, `new_bit = 1
/// if new_val >= 0`.
pub fn hv_ema_update(current: &HV, bundle: &HV, alpha: f32) -> HV {
    let mut out = [0u8; D / 8];
    let inv = 1.0 - alpha;
    for i in 0..(D / 8) {
        let mut byte = 0u8;
        for bit in 0..8 {
            let old = if (current.0[i] >> bit) & 1 == 1 { 1.0f32 } else { -1.0 };
            let bun = if (bundle.0[i] >> bit) & 1 == 1 { 1.0f32 } else { -1.0 };
            let new_val = inv * old + alpha * bun;
            if new_val >= 0.0 {
                byte |= 1 << bit;
            }
        }
        out[i] = byte;
    }
    HV(out)
}

/// Subtraction: remove `bundle`'s contribution from `current` via XOR.
/// Bits set in the bundle get flipped in the current vector, removing
/// the unlearned content's alignment.
pub fn hv_subtract(current: &HV, bundle: &HV) -> HV {
    let mut out = [0u8; D / 8];
    for i in 0..(D / 8) {
        out[i] = current.0[i] ^ bundle.0[i];
    }
    HV(out)
}

/// Bundle a set of HVs (per-bit majority) — used to combine tombstoned
/// signatures before subtraction.
pub fn hv_bundle(hvs: &[HV]) -> HV {
    HV::bundle(hvs)
}

// ---------------------------------------------------------------------------
// Store API.
// ---------------------------------------------------------------------------

impl Store {
    /// Read the current self-vector. Returns a zero HV if none has been
    /// initialized yet (fresh store).
    pub fn self_vector(&self) -> Result<HV> {
        let tx = self.db.begin_read()?;
        let manifest = tx.open_table(MANIFEST)?;
        if let Some(v) = manifest.get(KEY_SELF_VECTOR)? {
            let bytes = v.value();
            if bytes.len() == D / 8 {
                let mut hv = [0u8; D / 8];
                hv.copy_from_slice(&bytes);
                return Ok(HV(hv));
            }
        }
        Ok(HV([0u8; D / 8]))
    }

    /// Every self-vector snapshot in chronological order.
    pub fn self_vector_history(&self) -> Result<Vec<SelfVectorSnapshot>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(SELF_VECTOR_HISTORY)?;
        let mut out = Vec::new();
        for entry in table.iter()? {
            let (_, v) = entry?;
            out.push(decode(&v.value())?);
        }
        Ok(out)
    }

    /// Update the self-vector via EMA toward `bundle` and append a
    /// timestamped snapshot to the history. Called by the consolidation
    /// worker. Returns the hamming distance between old and new (the
    /// "drift" this pass caused).
    pub fn update_self_vector(&mut self, bundle: &HV, alpha: f32) -> Result<u32> {
        let current = self.self_vector()?;
        let new = hv_ema_update(&current, bundle, alpha);
        let drift = current.hamming(&new);
        self.write_self_vector(&new)?;
        // Snapshot the updated vector to the history.
        let seq = self.next_self_vector_seq()?;
        let snapshot = SelfVectorSnapshot {
            hv: new,
            at: Utc::now(),
            seq,
        };
        let tx = self.db.begin_write()?;
        {
            let mut hist = tx.open_table(SELF_VECTOR_HISTORY)?;
            hist.insert(seq, encode(&snapshot)?)?;
        }
        tx.commit()?;
        // Emit a learning event.
        let event = crate::learning_log::LearningEvent::SelfVectorUpdated {
            drift_hamming: drift,
            at: Utc::now(),
        };
        self.record_event(event)?;
        Ok(drift)
    }

    /// Subtract `bundle` from the self-vector (unlearn). Returns the
    /// hamming distance between old and new.
    pub fn subtract_from_self_vector(&mut self, bundle: &HV) -> Result<u32> {
        let current = self.self_vector()?;
        let new = hv_subtract(&current, bundle);
        let drift = current.hamming(&new);
        self.write_self_vector(&new)?;
        // Snapshot the corrected vector.
        let seq = self.next_self_vector_seq()?;
        let snapshot = SelfVectorSnapshot {
            hv: new,
            at: Utc::now(),
            seq,
        };
        let tx = self.db.begin_write()?;
        {
            let mut hist = tx.open_table(SELF_VECTOR_HISTORY)?;
            hist.insert(seq, encode(&snapshot)?)?;
        }
        tx.commit()?;
        Ok(drift)
    }

    /// Initialize the self-vector to a zero HV if not yet set. Called on
    /// store open so `self_vector()` always returns a valid value.
    pub fn init_self_vector(&mut self) -> Result<()> {
        let tx = self.db.begin_write()?;
        {
            let mut manifest = tx.open_table(MANIFEST)?;
            if manifest.get(KEY_SELF_VECTOR)?.is_none() {
                manifest.insert(KEY_SELF_VECTOR, encode(&[0u8; D / 8].to_vec())?)?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    fn write_self_vector(&mut self, hv: &HV) -> Result<()> {
        let tx = self.db.begin_write()?;
        {
            let mut manifest = tx.open_table(MANIFEST)?;
            manifest.insert(KEY_SELF_VECTOR, hv.0.to_vec())?;
        }
        tx.commit()?;
        Ok(())
    }

    fn next_self_vector_seq(&mut self) -> Result<u64> {
        let tx = self.db.begin_write()?;
        let seq;
        {
            let mut manifest = tx.open_table(MANIFEST)?;
            let raw = manifest.get(KEY_SELF_VECTOR_SEQ)?.map(|v| v.value());
            let current: u64 = match raw {
                Some(bytes) => decode(&bytes)?,
                None => 0,
            };
            let next = current + 1;
            manifest.insert(KEY_SELF_VECTOR_SEQ, encode(&next)?)?;
            seq = next;
        }
        tx.commit()?;
        Ok(seq)
    }
}
