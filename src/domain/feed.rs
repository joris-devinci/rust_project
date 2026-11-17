use std::collections::HashMap;

use super::{asset::Asset, error::OracleError, money::Price};

pub trait PriceFeed: Send + Sync {
    fn price(&self, asset: Asset) -> Result<Price, OracleError>;
}

/// Fixed prices, used in tests and until the oracle is ready.
pub struct StaticFeed {
    prices: HashMap<Asset, Price>,
}

impl StaticFeed {
    pub fn new(prices: impl IntoIterator<Item = (Asset, Price)>) -> Self {
        Self {
            prices: prices.into_iter().collect(),
        }
    }
}

impl PriceFeed for StaticFeed {
    fn price(&self, asset: Asset) -> Result<Price, OracleError> {
        self.prices
            .get(&asset)
            .copied()
            .ok_or(OracleError::NoPrice(asset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_feed_returns_configured_price() {
        let feed = StaticFeed::new([(Asset::Usdc, Price::new(100_000_000))]);
        assert_eq!(feed.price(Asset::Usdc).ok(), Some(Price::new(100_000_000)));
    }

    #[test]
    fn static_feed_rejects_unknown_asset() {
        let feed = StaticFeed::new([(Asset::Usdc, Price::new(100_000_000))]);
        assert!(matches!(
            feed.price(Asset::Eth),
            Err(OracleError::NoPrice(Asset::Eth))
        ));
    }
}
