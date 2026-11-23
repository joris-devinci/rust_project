//! CPU workload for the flamegraph: the keeper's scan over many accounts.
//! Run it with `cargo flamegraph --root --example keeper_scan`.

use std::{error::Error, hint::black_box};

use rust_decimal::Decimal;
use rust_project::{
    domain::{Amount, Asset, CollateralFactor, Price, StaticFeed},
    protocol::Protocol,
};

const ACCOUNTS: u64 = 100_000;
const SCANS: usize = 200;

fn main() -> Result<(), Box<dyn Error>> {
    let protocol = protocol_with_loans(ACCOUNTS, &feed(2_000))?;
    let crashed = feed(1_200);
    let mut found = 0;
    for _ in 0..SCANS {
        found = black_box(protocol.liquidatable_accounts(black_box(&crashed))?).len();
    }
    println!("{found} of {ACCOUNTS} accounts are liquidatable");
    Ok(())
}

/// ETH at `eth_usd` dollars, BTC at $50,000.
fn feed(eth_usd: i64) -> StaticFeed {
    StaticFeed::new([
        (Asset::Eth, Price::new(Decimal::from(eth_usd))),
        (Asset::Btc, Price::new(Decimal::from(50_000))),
    ])
}

/// Each account deposits 1 ETH and borrows between 0.001 and 0.030 BTC.
fn protocol_with_loans(count: u64, feed: &StaticFeed) -> Result<Protocol, Box<dyn Error>> {
    let factor = CollateralFactor::new(Decimal::new(8, 1)).ok_or("0.8 is a valid factor")?;
    let mut protocol = Protocol::new(factor);
    for i in 0..count {
        let id = protocol.open_account()?;
        protocol.deposit(id, Asset::Eth, Amount::new(Decimal::ONE))?;
        let thousandths = i64::try_from(1 + i % 30)?;
        protocol.borrow(
            id,
            Asset::Btc,
            Amount::new(Decimal::new(thousandths, 3)),
            feed,
        )?;
    }
    Ok(protocol)
}
