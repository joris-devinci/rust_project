use serde::{Deserialize, Serialize};

const PRICE_DECIMALS: usize = 8;
const PRICE_SCALE: u128 = 100_000_000;
const BPS_SCALE: u16 = 10_000;

/// USD price of one unit of an asset, with 8 decimals: $2,000.50 is `200_050_000_000`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Price(u128);

/// Quantity of an asset, with 6 decimals for every asset: 1.5 ETH is `1_500_000`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Amount(u128);

/// Value in USD, with 6 decimals like `Amount`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UsdValue(u128);

/// Ratio in basis points, from 0 to 10,000: 80% is `8_000`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bps(u16);

impl Price {
    pub const fn new(raw: u128) -> Self {
        Self(raw)
    }

    /// Parses a decimal such as `"2000.5"`, without going through `f64`.
    /// Decimals beyond the 8th are truncated. Returns `None` if the text is not a decimal.
    pub fn from_decimal(text: &str) -> Option<Self> {
        let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
        let is_digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
        if whole.is_empty() || !is_digits(whole) || !is_digits(fraction) {
            return None;
        }
        let padded: String = fraction
            .chars()
            .chain(std::iter::repeat('0'))
            .take(PRICE_DECIMALS)
            .collect();
        let whole: u128 = whole.parse().ok()?;
        let fraction: u128 = padded.parse().ok()?;
        whole
            .checked_mul(PRICE_SCALE)?
            .checked_add(fraction)
            .map(Self)
    }

    /// Average of two prices, rounding down, without overflow.
    pub fn midpoint(self, other: Self) -> Self {
        Self(self.0.midpoint(other.0))
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

    /// Converts to USD, rounding up. Used for debt, so that dust is never valued at zero.
    pub fn to_usd_rounded_up(self, price: Price) -> Option<UsdValue> {
        self.0
            .checked_mul(price.0)
            .map(|value| UsdValue(value.div_ceil(PRICE_SCALE)))
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }
}

impl UsdValue {
    pub const ZERO: Self = Self(0);

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    /// Applies a ratio, rounding down. Returns `None` on overflow.
    pub fn apply(self, ratio: Bps) -> Option<Self> {
        self.0
            .checked_mul(u128::from(ratio.0))
            .map(|value| Self(value / u128::from(BPS_SCALE)))
    }
}

impl Bps {
    /// Returns `None` above 100%.
    pub const fn new(raw: u16) -> Option<Self> {
        if raw <= BPS_SCALE {
            Some(Self(raw))
        } else {
            None
        }
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

    #[test]
    fn dust_rounds_down_to_zero_but_up_to_one_unit() {
        let dust = Amount::new(1); // 0.000001 of an asset
        let price = Price::new(50_000_000); // $0.50
        assert_eq!(dust.to_usd(price), Some(UsdValue(0)));
        assert_eq!(dust.to_usd_rounded_up(price), Some(UsdValue(1)));
    }

    #[test]
    fn applying_80_percent_to_3000_usd_gives_2400_usd() {
        let ratio = Bps::new(8_000).expect("80% is a valid ratio");
        assert_eq!(
            UsdValue(3_000_000_000).apply(ratio),
            Some(UsdValue(2_400_000_000))
        );
    }

    #[test]
    fn decimal_prices_are_parsed_exactly() {
        assert_eq!(
            Price::from_decimal("2000.5"),
            Some(Price::new(200_050_000_000))
        );
        assert_eq!(
            Price::from_decimal("93250"),
            Some(Price::new(9_325_000_000_000))
        );
        assert_eq!(Price::from_decimal("0.00000001"), Some(Price::new(1)));
    }

    #[test]
    fn decimals_beyond_the_eighth_are_truncated() {
        assert_eq!(
            Price::from_decimal("1.123456789"),
            Some(Price::new(112_345_678))
        );
    }

    #[test]
    fn non_decimal_text_is_rejected() {
        for text in ["", ".5", "-1", "1e3", "12.3.4", "NaN", " 1"] {
            assert_eq!(Price::from_decimal(text), None, "{text:?}");
        }
    }

    #[test]
    fn midpoint_rounds_down() {
        assert_eq!(Price::new(3).midpoint(Price::new(6)), Price::new(4));
        assert_eq!(
            Price::new(u128::MAX).midpoint(Price::new(u128::MAX)),
            Price::new(u128::MAX)
        );
    }

    #[test]
    fn bps_above_100_percent_is_rejected() {
        assert_eq!(Bps::new(10_001), None);
    }
}
