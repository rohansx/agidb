# phase 1 — the HDC kernel

**duration:** weeks — (inherited from sochdb v1)
**status:** complete (inherited from sochdb v1)
**depends on:** [phase 0](./phase-0-setup.md)

## goal

land the smallest correct HDC kernel — the math that makes layer 1 work. no storage, no extraction, no API. just the kernel and its tests.

## deliverables

- [x] `agidb-core/src/hdc.rs` with the `HV` type and:
  - `from_name(&str) -> HV` — deterministic hash-derived hypervector
  - `bind(&HV, &HV) -> HV` — XOR binding
  - `bundle(&[HV]) -> HV` — per-bit majority bundling
  - `hamming(&self, &HV) -> u32` — hamming distance
  - `similarity(&self, &HV) -> f32` — derived from hamming
  - `active_dims(&self) -> impl Iterator<Item = u32>` — indices of set bits
- [x] AVX-512 POPCOUNT path behind `target_feature = "avx512vpopcntdq"`
- [x] NEON POPCOUNT path for aarch64 / Apple silicon
- [x] portable fallback using `u64::count_ones()` over 128 chunks
- [x] property tests via `proptest`:
  - binding is its own inverse: `a.bind(&b).bind(&b) == a`
  - bundling membership: each bundled HV has high similarity to the bundle
  - bundling commutativity: `bundle([a,b,c]) == bundle([c,a,b])`
- [x] micro-benchmarks via `criterion`:
  - signature compute (`from_name`)
  - single bind, single bundle of N
  - hamming over 100k stored signatures

## exit criterion

**8192-bit hamming-distance scan over 100k random signatures completes in under 5ms on M2.**

verification: `cargo bench --bench hdc_scan` in CI on the benchmark laptop. raw log committed to `bench/results/phase-1/`.

## tasks

1. write the property tests first (TDD per [common/testing.md](https://github.com/rsx/dotfiles/blob/main/.claude/rules/common/testing.md))
2. implement portable `hamming` and `bundle` to pass the property tests
3. add AVX-512 path with `is_x86_feature_detected!` runtime gate
4. add NEON path with `cfg(target_arch = "aarch64")`
5. add criterion benches, lock the 5ms target in CI
6. document the algebra in `hdc.rs` rustdoc

## risks

| risk | mitigation |
|---|---|
| AVX-512 not available on benchmark machine | confirm M2 NEON path meets target first; AVX-512 is x86 nice-to-have |
| bundle correctness subtle for even-sized inputs | tie-breaking documented and tested; mirror Torchhd's behavior |
| popcount in unsafe code | wrap unsafe in `#[inline]` fn, fuzz with `cargo-fuzz` |

## what unblocks next

phase 2 (storage) consumes the `HV` type. it depends on the layout being final (`#[repr(C, align(64))]`, fixed 1024 bytes).

## references

- [architecture/layer-1-recall.md](../architecture/layer-1-recall.md) — the math this implements
- [spec/tech-spec.md](../spec/tech-spec.md#the-hdc-kernel) — the type signature this lands
