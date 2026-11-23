//! Time of the keeper's scan, which holds the protocol's read lock while it runs.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rust_decimal::Decimal;
use rust_project::{
    domain::{Amount, Asset, CollateralFactor, Price, StaticFeed},
    protocol::Protocol,
};

/// ETH at `eth_usd` dollars, BTC at $50,000.
fn feed(eth_usd: i64) -> StaticFeed {
    StaticFeed::new([
        (Asset::Eth, Price::new(Decimal::from(eth_usd))),
        (Asset::Btc, Price::new(Decimal::from(50_000))),
    ])
}

/// Each account deposits 1 ETH and borrows between 0.001 and 0.030 BTC, safe at $2,000.
fn protocol_with_loans(count: u64) -> Protocol {
    let factor = CollateralFactor::new(Decimal::new(8, 1)).expect("0.8 is a valid factor");
    let mut protocol = Protocol::new(factor);
    let feed = feed(2_000);
    for i in 0..count {
        let id = protocol.open_account().expect("ids do not overflow");
        protocol
            .deposit(id, Asset::Eth, Amount::new(Decimal::ONE))
            .expect("deposit fits");
        let thousandths = i64::try_from(1 + i % 30).expect("below 31");
        protocol
            .borrow(
                id,
                Asset::Btc,
                Amount::new(Decimal::new(thousandths, 3)),
                &feed,
            )
            .expect("loan is healthy with ETH at $2,000");
    }
    protocol
}

fn scan(c: &mut Criterion) {
    // ETH crashes to $1,200: about a third of the accounts become liquidatable
    let crashed = feed(1_200);
    let mut group = c.benchmark_group("liquidatable_accounts");
    for count in [1_000, 10_000, 100_000] {
        // Built outside of iter: only the scan is timed
        let protocol = protocol_with_loans(count);
        group.bench_function(BenchmarkId::from_parameter(count), |b| {
            b.iter(|| protocol.liquidatable_accounts(black_box(&crashed)))
        });
    }
    group.finish();
}

criterion_group!(benches, scan);
criterion_main!(benches);
