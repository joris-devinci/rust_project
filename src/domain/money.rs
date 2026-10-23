use serde::{Deserialize, Serialize};

const PRICE_SCALE: u128 = 100_000_000;

/// USD price of one unit of an asset, with 8 decimals: $2,000.50 is `200_050_000_000`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Price(u128);

/// Quantity of an asset, with 6 decimals for every asset: 1.5 ETH is `1_500_000`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Amount(u128);

/// Value in USD, with 6 decimals like `Amount`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UsdValue(u128);

impl Price {
    pub const fn new(raw: u128) -> Self {
        Self(raw)
    }
}

impl Amount {
    pub const fn new(raw: u128) -> Self {
        Self(raw)
    }

    /// Converts to USD, rounding down. Returns `None` on overflow.
    pub fn to_usd(self, price: Price) -> Option<UsdValue> {
        self.0
            .checked_mul(price.0)
            .map(|value| UsdValue(value / PRICE_SCALE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eth_amount_converts_to_usd() {
        let amount = Amount::new(1_500_000); // 1.5 ETH
        let price = Price::new(200_000_000_000); // $2,000
        assert_eq!(amount.to_usd(price), Some(UsdValue(3_000_000_000)));
    }

    #[test]
    fn usdc_at_one_dollar_keeps_its_amount() {
        let amount = Amount::new(1_500_000_000); // 1,500 USDC
        let price = Price::new(100_000_000); // $1
        assert_eq!(amount.to_usd(price), Some(UsdValue(1_500_000_000)));
    }

    #[test]
    fn to_usd_returns_none_on_overflow() {
        assert_eq!(Amount::new(u128::MAX).to_usd(Price::new(2)), None);
    }
}
