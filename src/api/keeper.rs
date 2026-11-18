use std::time::Duration;

use crate::{
    domain::PriceFeed,
    protocol::{AccountId, LendingError, Position},
};

use super::{AppState, error::ApiError};

/// Every `period`, liquidates the unhealthy positions. The scan is CPU work over every
/// account, so it runs on tokio's blocking thread pool instead of an async worker thread.
pub async fn run_keeper(state: AppState, period: Duration) {
    let mut interval = tokio::time::interval(period);
    loop {
        interval.tick().await;
        let feed = match state.get_current_prices().await {
            Ok(feed) => feed,
            Err(error) => {
                eprintln!("keeper: prices unavailable: {error}");
                continue;
            }
        };
        let scan_state = state.clone();
        match tokio::task::spawn_blocking(move || liquidate_unhealthy(&scan_state, feed.as_ref()))
            .await
        {
            Ok(Ok(liquidations)) => {
                for (id, seized) in liquidations {
                    println!("keeper: liquidated {id:?}, seized {seized:?}");
                }
            }
            Ok(Err(error)) => eprintln!("keeper: scan failed: {error}"),
            Err(error) => eprintln!("keeper: scan panicked: {error}"),
        }
    }
}

/// Scans under the read lock, so requests keep being served, then takes the write lock
/// only to liquidate the accounts it found.
fn liquidate_unhealthy(
    state: &AppState,
    feed: &dyn PriceFeed,
) -> Result<Vec<(AccountId, Position)>, ApiError> {
    let unhealthy = state.read()?.liquidatable_accounts(feed)?;
    if unhealthy.is_empty() {
        return Ok(Vec::new());
    }
    let mut protocol = state.write()?;
    let mut liquidations = Vec::new();
    for id in unhealthy {
        match protocol.liquidate(id, feed) {
            Ok(seized) => liquidations.push((id, seized)),
            // Repaid or topped up between the scan and the write lock
            Err(LendingError::NotLiquidatable(_)) => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(liquidations)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::domain::{Amount, Asset, CollateralFactor, Price, StaticFeed};
    use crate::protocol::{Health, Protocol};
    use rust_decimal::Decimal;

    fn decimal(value: &str) -> Decimal {
        value.parse().expect("valid decimal")
    }

    fn collateral_factor() -> CollateralFactor {
        CollateralFactor::new(decimal("0.8")).expect("0.8 is valid")
    }

    fn feed(eth_price: &str) -> StaticFeed {
        StaticFeed::new([
            (Asset::Eth, Price::new(decimal(eth_price))),
            (Asset::Btc, Price::new(decimal("50000"))),
        ])
    }

    // One account borrowing 0.03 BTC ($1,500) against 1.5 ETH, opened while ETH is at $2,000
    fn state_after_eth_moves_to(eth_price: &str) -> Result<(AppState, AccountId), ApiError> {
        let mut protocol = Protocol::new(collateral_factor());
        let id = protocol.open_account()?;
        protocol.deposit(id, Asset::Eth, Amount::new(decimal("1.5")))?;
        protocol.borrow(id, Asset::Btc, Amount::new(decimal("0.03")), &feed("2000"))?;
        Ok((AppState::new(protocol, Arc::new(feed(eth_price))), id))
    }

    #[test]
    fn keeper_leaves_healthy_positions_alone() -> Result<(), ApiError> {
        let feed = feed("2000");
        let (state, id) = state_after_eth_moves_to("2000")?;
        assert!(liquidate_unhealthy(&state, &feed)?.is_empty());
        assert_eq!(state.read()?.health(id, &feed)?, Health::Safe);
        Ok(())
    }

    #[test]
    fn keeper_liquidates_after_a_crash() -> Result<(), ApiError> {
        let feed = feed("1200");
        let (state, id) = state_after_eth_moves_to("1200")?;
        let liquidations = liquidate_unhealthy(&state, &feed)?;
        assert_eq!(liquidations.len(), 1);
        assert_eq!(liquidations[0].0, id);
        assert_eq!(state.read()?.health(id, &feed)?, Health::NoDebt);
        Ok(())
    }
}
