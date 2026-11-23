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
pub fn liquidate_unhealthy(
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
