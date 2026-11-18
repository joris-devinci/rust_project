//! Median with a partial selection, O(n), against a full sort, O(n log n).

use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use rust_decimal::Decimal;
use rust_project::{domain::Price, oracle::median};

fn median_vs_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("median");
    for count in [3_i64, 100, 10_000] {
        // Prices in a scrambled order, so that neither version gets sorted input
        let prices: Vec<Price> = (0..count)
            .map(|i| Price::new(Decimal::new(i * 7_919 % 10_007, 2)))
            .collect();
        // median reorders its input: each run gets a fresh copy, made outside the timing
        group.bench_function(BenchmarkId::new("select_nth", count), |b| {
            b.iter_batched(
                || prices.clone(),
                |mut copy| median(black_box(&mut copy)),
                BatchSize::SmallInput,
            )
        });
        group.bench_function(BenchmarkId::new("full_sort", count), |b| {
            b.iter_batched(
                || prices.clone(),
                |mut copy| {
                    black_box(&mut copy).sort_unstable();
                    copy[copy.len() / 2]
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

criterion_group!(benches, median_vs_sort);
criterion_main!(benches);
