use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::models::Asset;

use super::{error::OracleError, money::Price};

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
        if asset == Asset::Usd {
            return Ok(Price::new(Decimal::ONE));
        }

        self.prices
            .get(&asset)
            .copied()
            .ok_or(OracleError::NoPrice(asset))
    }
}
