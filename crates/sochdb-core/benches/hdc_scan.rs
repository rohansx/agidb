//! Phase 1 exit criterion bench.
//!
//! Target: 8192-bit hamming-distance scan over 100k random signatures
//! completes in **under 5ms** on the benchmark laptop (Apple M2 / Intel
//! i7-12700H, 16 GB RAM, NVMe SSD).
//!
//! Run with:
//!   cargo bench -p sochdb-core --bench hdc_scan
//!
//! Failure modes the bench should catch:
//! - portable fallback used when AVX-512 is available
//! - per-HV allocations leaking into the hot path
//! - aliasing-prevented vectorization (verify with `cargo asm`)
//!
//! Until phase 1 lands a real `hamming`, this bench will panic on the
//! first iteration — that's expected; the bench exists to lock in the
//! exit criterion once implementation begins.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use sochdb_core::hdc::{HV, D_BYTES};

const SCAN_SIZE: usize = 100_000;

fn make_random_corpus(n: usize, seed: u64) -> Vec<HV> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut corpus = Vec::with_capacity(n);
    for _ in 0..n {
        let mut bytes = [0u8; D_BYTES];
        rng.fill_bytes(&mut bytes);
        corpus.push(HV(bytes));
    }
    corpus
}

fn bench_hamming_scan_100k(c: &mut Criterion) {
    let corpus = make_random_corpus(SCAN_SIZE, 0x50CC_DB00_u64);
    let mut query_bytes = [0u8; D_BYTES];
    StdRng::seed_from_u64(0xC0FF_EE00_u64).fill_bytes(&mut query_bytes);
    let query = HV(query_bytes);

    let mut group = c.benchmark_group("hdc_scan");
    group.throughput(Throughput::Elements(SCAN_SIZE as u64));
    group.sample_size(20);
    group.bench_function("hamming_100k", |b| {
        b.iter(|| {
            let mut min = u32::MAX;
            for hv in corpus.iter() {
                let d = black_box(query.hamming(black_box(hv)));
                if d < min {
                    min = d;
                }
            }
            black_box(min)
        })
    });
    group.finish();
}

criterion_group!(benches, bench_hamming_scan_100k);
criterion_main!(benches);
