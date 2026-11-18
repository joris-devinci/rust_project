use std::{
    collections::HashMap,
    sync::{PoisonError, RwLock},
};

use crate::domain::{Asset, OracleError, Price, PriceFeed};

/// USDC is a stablecoin pegged to the dollar, and the scraper does not fetch it.
const USDC_PRICE: Price = Price::new(100_000_000);

/// Latest oracle prices, written by the refresh task and read by the lending protocol.
#[derive(Debug, Default)]
pub struct OracleFeed {
    prices: RwLock<HashMap<Asset, Price>>,
}

impl OracleFeed {
    /// Replaces every price at once: an asset missing from `prices` no longer has a price,
    /// so the protocol never values a position with a stale price.
    pub fn replace(&self, prices: HashMap<Asset, Price>) {
        // The map is swapped in one assignment, so a poisoned lock never holds half-written data
        *self.prices.write().unwrap_or_else(PoisonError::into_inner) = prices;
    }
}

impl PriceFeed for OracleFeed {
    fn price(&self, asset: Asset) -> Result<Price, OracleError> {
        if asset == Asset::Usdc {
            return Ok(USDC_PRICE);
        }
        self.prices
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&asset)
            .copied()
            .ok_or(OracleError::NoPrice(asset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usdc_is_pegged_to_one_dollar() -> Result<(), OracleError> {
        assert_eq!(OracleFeed::default().price(Asset::Usdc)?, USDC_PRICE);
        Ok(())
    }

    #[test]
    fn replace_drops_assets_missing_from_the_refresh() {
        let feed = OracleFeed::default();
        feed.replace(HashMap::from([(Asset::Eth, Price::new(1))]));
        feed.replace(HashMap::from([(Asset::Btc, Price::new(2))]));
        assert!(matches!(feed.price(Asset::Btc), Ok(price) if price == Price::new(2)));
        assert!(matches!(
            feed.price(Asset::Eth),
            Err(OracleError::NoPrice(Asset::Eth))
        ));
    }
}
