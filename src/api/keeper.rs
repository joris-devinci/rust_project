use std::time::Duration;

use crate::protocol::{AccountId, LendingError, Position};

use super::{AppState, error::ApiError};

/// Every `period`, liquidates the unhealthy positions. The scan is CPU work over every
/// account, so it runs on tokio's blocking thread pool instead of an async worker thread.
pub async fn run_keeper(state: AppState, period: Duration) {
    let mut interval = tokio::time::interval(period);
    loop {
        interval.tick().await;
        let scan_state = state.clone();
        match tokio::task::spawn_blocking(move || liquidate_unhealthy(&scan_state)).await {
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
fn liquidate_unhealthy(state: &AppState) -> Result<Vec<(AccountId, Position)>, ApiError> {
    let feed = state.feed.as_ref();
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
    use crate::domain::{Amount, Asset, Bps, Price, StaticFeed};
    use crate::protocol::{Health, Protocol};

    fn feed(eth_price: u128) -> StaticFeed {
        StaticFeed::new([
            (Asset::Eth, Price::new(eth_price)),
            (Asset::Usdc, Price::new(100_000_000)),
        ])
    }

    // One account borrowing 1,500 USDC against 1.5 ETH, opened while ETH is at $2,000
    fn state_after_eth_moves_to(eth_price: u128) -> Result<(AppState, AccountId), ApiError> {
        let mut protocol = Protocol::new(Bps::new(8_000).expect("80% is a valid ratio"));
        let id = protocol.open_account()?;
        protocol.deposit(id, Asset::Eth, Amount::new(1_500_000))?;
        protocol.borrow(
            id,
            Asset::Usdc,
            Amount::new(1_500_000_000),
            &feed(200_000_000_000),
        )?;
        Ok((AppState::new(protocol, Arc::new(feed(eth_price))), id))
    }

    #[test]
    fn keeper_leaves_healthy_positions_alone() -> Result<(), ApiError> {
        let (state, id) = state_after_eth_moves_to(200_000_000_000)?;
        assert!(liquidate_unhealthy(&state)?.is_empty());
        assert_eq!(state.read()?.health(id, state.feed.as_ref())?, Health::Safe);
        Ok(())
    }

    #[test]
    fn keeper_liquidates_after_a_crash() -> Result<(), ApiError> {
        let (state, id) = state_after_eth_moves_to(120_000_000_000)?;
        let liquidations = liquidate_unhealthy(&state)?;
        assert_eq!(liquidations.len(), 1);
        assert_eq!(liquidations[0].0, id);
        assert_eq!(
            state.read()?.health(id, state.feed.as_ref())?,
            Health::NoDebt
        );
        Ok(())
    }
}
