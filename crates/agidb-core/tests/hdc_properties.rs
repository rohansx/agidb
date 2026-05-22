//! Property tests for the HDC algebra.
//!
//! These tests encode the algebraic invariants the kernel must satisfy.
//! The scaffold ships with `todo!()` in `bind`, `bundle`, and `hamming`,
//! so every test in this file panics at scaffold time — that's the
//! intentional RED state for phase 1 TDD.
//!
//! When implementing phase 1, the goal is to turn every panic in this
//! file into a green test without weakening any invariant.

use agidb_core::hdc::{D, D_BYTES, HV};
use proptest::prelude::*;

/// Strategy for generating a fully arbitrary `HV` — 1024 random bytes
/// with no structural assumptions. Property tests use this to cover
/// the worst-case input distribution.
fn arb_hv() -> impl Strategy<Value = HV> {
    prop::collection::vec(any::<u8>(), D_BYTES).prop_map(|v| {
        let mut arr = [0u8; D_BYTES];
        arr.copy_from_slice(&v);
        HV(arr)
    })
}

/// Strategy for a small bundle of HVs (1..=7 members — mirrors working
/// memory's ~7-item Miller-magic capacity). Bundle membership is
/// stronger at small N and degrades at large N; the tests target the
/// regime agidb actually uses.
fn arb_bundle_inputs() -> impl Strategy<Value = Vec<HV>> {
    prop::collection::vec(arb_hv(), 1..=7)
}

proptest! {
    /// Binding is self-inverse: `a ⊗ b ⊗ b == a`.
    ///
    /// This is the *unbinding* property — given a bound triple
    /// `SUBJ ⊗ Sarah`, applying `SUBJ` again recovers `Sarah`. Phase 4's
    /// recall depends on this exact identity.
    #[test]
    fn bind_is_self_inverse(a in arb_hv(), b in arb_hv()) {
        let bound = a.bind(&b);
        let recovered = bound.bind(&b);
        prop_assert_eq!(recovered, a);
    }

    /// XOR is commutative, so `bind` must be too.
    #[test]
    fn bind_is_commutative(a in arb_hv(), b in arb_hv()) {
        prop_assert_eq!(a.bind(&b), b.bind(&a));
    }

    /// Bundling a single HV with itself is the identity.
    #[test]
    fn bundle_singleton_is_identity(a in arb_hv()) {
        prop_assert_eq!(HV::bundle(&[a]), a);
    }

    /// Per-bit majority is order-independent. `bundle([a,b,c])` must
    /// equal `bundle([c,a,b])` for any permutation of the inputs.
    #[test]
    fn bundle_is_commutative(inputs in arb_bundle_inputs()) {
        let mut reversed = inputs.clone();
        reversed.reverse();
        prop_assert_eq!(HV::bundle(&inputs), HV::bundle(&reversed));
    }

    /// Bundle membership: any member of a small bundle is significantly
    /// more similar to the bundle than two unrelated HVs are to each
    /// other.
    ///
    /// Threshold rationale: with N≤7 members and 8192-bit HVs, each
    /// member's expected hamming distance to the bundle is well under
    /// D/2. We use a conservative 0.6 floor — i.e. similarity ≥ 0.6,
    /// meaning hamming ≤ 0.4·D = 3276. Two unrelated HVs hover near 0.5.
    #[test]
    fn bundle_membership(inputs in arb_bundle_inputs()) {
        let bundle = HV::bundle(&inputs);
        for member in &inputs {
            let sim = bundle.similarity(member);
            prop_assert!(
                sim >= 0.6,
                "member similarity {} below floor 0.6; this would break tier-B recall",
                sim
            );
        }
    }

    /// Hamming distance from any HV to itself is zero.
    #[test]
    fn hamming_self_is_zero(a in arb_hv()) {
        prop_assert_eq!(a.hamming(&a), 0);
    }

    /// Hamming distance is symmetric.
    #[test]
    fn hamming_is_symmetric(a in arb_hv(), b in arb_hv()) {
        prop_assert_eq!(a.hamming(&b), b.hamming(&a));
    }

    /// Hamming distance is bounded by the dimensionality.
    #[test]
    fn hamming_is_bounded(a in arb_hv(), b in arb_hv()) {
        prop_assert!(a.hamming(&b) <= D as u32);
    }

    /// `similarity` is the linear complement of normalized hamming.
    /// This is a sanity check on the derived metric — it must stay in
    /// `[0.0, 1.0]` for any inputs.
    #[test]
    fn similarity_is_in_unit_interval(a in arb_hv(), b in arb_hv()) {
        let s = a.similarity(&b);
        prop_assert!((0.0..=1.0).contains(&s), "similarity {} out of [0,1]", s);
    }
}

/// Non-proptest sanity checks that exercise the deterministic surface.

#[test]
fn from_name_is_deterministic() {
    let a = HV::from_name("Sarah");
    let b = HV::from_name("Sarah");
    assert_eq!(a, b, "from_name must be deterministic for the same input");
}

#[test]
fn from_name_distinguishes_different_inputs() {
    let a = HV::from_name("Sarah");
    let b = HV::from_name("Bawri");
    assert_ne!(a, b, "different names must produce different HVs");
}

#[test]
fn zero_hv_has_no_active_dims() {
    let z = HV::zero();
    assert_eq!(z.active_dims().count(), 0);
}

#[test]
fn active_dims_yields_ascending_indices() {
    // bit 0 of byte 0, bit 3 of byte 1, bit 7 of byte 127 → indices 0, 11, 1023
    let mut bytes = [0u8; D_BYTES];
    bytes[0] = 0b0000_0001;
    bytes[1] = 0b0000_1000;
    bytes[127] = 0b1000_0000;
    let hv = HV(bytes);
    let dims: Vec<u32> = hv.active_dims().collect();
    assert_eq!(dims, vec![0, 11, 1023]);
}
