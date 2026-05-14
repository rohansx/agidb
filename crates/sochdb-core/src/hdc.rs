//! Layer 1 — the HDC kernel.
//!
//! Phase 1 builds:
//! - `HV` — a binary hypervector of [`D`] bits / [`D_BYTES`] bytes
//! - `bind`, `bundle`, `hamming` — the algebraic primitives
//! - AVX-512 + NEON + portable POPCOUNT paths
//!
//! See `docs/architecture/layer-1-recall.md` for the math and
//! `docs/phases/phase-1-hdc-kernel.md` for the exit criterion
//! (8192-bit hamming-distance scan over 100k random signatures in
//! <5ms on M2).
//!
//! Scaffold state: type and shapes are final; `bind`, `bundle`, and
//! `hamming` are intentionally `todo!()` so the property test suite
//! starts RED.

use std::hash::{Hash, Hasher};

/// Dimensionality of every hypervector, in bits.
pub const D: usize = 8192;

/// Dimensionality of every hypervector, in bytes (1024).
pub const D_BYTES: usize = D / 8;

/// A binary hypervector. Fixed size: 8192 bits / 1024 bytes, aligned
/// to a 64-byte cache line so AVX-512 / NEON loads can use aligned
/// instructions without a runtime check.
#[derive(Clone, Copy)]
#[repr(C, align(64))]
pub struct HV(pub [u8; D_BYTES]);

impl HV {
    /// All-zero hypervector. Useful as a bundling accumulator.
    pub const fn zero() -> Self {
        HV([0u8; D_BYTES])
    }

    /// Deterministic hypervector derived from a name. Same name always
    /// produces the same `HV`; different names produce uncorrelated
    /// vectors (in expectation).
    ///
    /// Implementation: xorshift64 expansion of the std DefaultHasher of
    /// the name. Adequate for phase 1; phase 2+ may switch to blake3 or
    /// xxhash for stronger uniformity guarantees.
    pub fn from_name(name: &str) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.hash(&mut hasher);
        let mut seed = hasher.finish().max(1); // xorshift breaks on zero seed
        let mut bytes = [0u8; D_BYTES];
        for chunk in bytes.chunks_mut(8) {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let next = seed.to_le_bytes();
            chunk.copy_from_slice(&next[..chunk.len()]);
        }
        HV(bytes)
    }

    /// XOR-binding. Binding is its own inverse: `a.bind(&b).bind(&b) == a`.
    /// Binding is commutative: `a.bind(&b) == b.bind(&a)`.
    ///
    /// Used to bind a role HV with a filler HV — `SUBJ ⊗ Sarah` — so the
    /// resulting vector encodes "Sarah in the SUBJ role" while looking
    /// uncorrelated to either operand.
    pub fn bind(&self, _other: &HV) -> HV {
        todo!("phase 1: implement XOR binding")
    }

    /// Per-bit majority bundling. `bundle([a, b, c])` is the hypervector
    /// where each bit is `1` iff a majority of the input HVs have that
    /// bit set. Bundle membership: `bundle([..., a, ...]).similarity(a)`
    /// is much higher than chance (≈0.5).
    ///
    /// Tie-breaking on even input counts: bit 0 wins. Documented in the
    /// property tests so future changes don't drift silently.
    pub fn bundle(_hvs: &[HV]) -> HV {
        todo!("phase 1: implement per-bit majority bundle")
    }

    /// Hamming distance — the number of bit positions where two HVs
    /// differ. Range: `0..=D`.
    ///
    /// Implemented via POPCOUNT over XOR. AVX-512 path used when
    /// `target_feature = "avx512vpopcntdq"` is available; NEON path on
    /// `target_arch = "aarch64"`; portable fallback uses
    /// `u64::count_ones()` over 128 chunks.
    pub fn hamming(&self, _other: &HV) -> u32 {
        todo!("phase 1: implement POPCOUNT-based hamming distance")
    }

    /// Cosine-like similarity score in `[0.0, 1.0]`, derived from
    /// hamming. Implemented in terms of `hamming` so phase 1 only needs
    /// to implement the kernel once.
    pub fn similarity(&self, other: &HV) -> f32 {
        1.0 - (self.hamming(other) as f32 / D as f32)
    }

    /// Indices of every set bit, in ascending order. Used by the
    /// inverted-index update path in phase 2.
    pub fn active_dims(&self) -> impl Iterator<Item = u32> + '_ {
        self.0.iter().enumerate().flat_map(|(byte_idx, &b)| {
            (0..8u32)
                .filter(move |bit| (b >> bit) & 1 == 1)
                .map(move |bit| byte_idx as u32 * 8 + bit)
        })
    }
}

impl PartialEq for HV {
    fn eq(&self, other: &Self) -> bool {
        self.0[..] == other.0[..]
    }
}

impl Eq for HV {}

impl std::fmt::Debug for HV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Show the first 32 bits as hex; the full 1024 bytes are too
        // noisy for any practical debug log.
        write!(f, "HV(")?;
        for byte in &self.0[..4] {
            write!(f, "{:02x}", byte)?;
        }
        write!(f, "..)")
    }
}
