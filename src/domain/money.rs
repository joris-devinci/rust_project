//! Decimal money types: amounts are asset units, prices are USD per unit, and ratios
//! are fractions from zero to one. No caller multiplies or divides by hidden scales.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Decimal USD price for one unit of an asset, for example `2000` means $2,000.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Price(Decimal);

/// Decimal quantity of an asset, for example `1.5` means one and a half units.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Amount(Decimal);

/// Decimal USD value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UsdValue(Decimal);

/// Fraction of collateral value that may back debt; `0.8` means 80%.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollateralFactor(Decimal);

impl Price {
    pub fn new(value: Decimal) -> Self {
        Self(value)
    }

    /// Returns the midpoint without adding the full values first.
    pub fn midpoint(self, other: Self) -> Option<Self> {
        let half = Decimal::new(5, 1);
        self.0
            .checked_mul(half)?
            .checked_add(other.0.checked_mul(half)?)
            .map(Self)
    }

    /// Formats the decimal without artificial trailing zeroes.
    pub fn to_decimal_string(self) -> String {
        format_decimal(self.0)
    }
}

impl Amount {
    pub const ZERO: Self = Self(Decimal::ZERO);

    pub fn new(value: Decimal) -> Self {
        Self(value)
    }

    pub fn is_negative(self) -> bool {
        self.0 < Decimal::ZERO
    }

    /// Converts an asset quantity to USD using the feed's USD-per-asset price.
    pub fn to_usd(self, price: Price) -> Option<UsdValue> {
        self.0.checked_mul(price.0).map(UsdValue)
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }
}

impl UsdValue {
    pub const ZERO: Self = Self(Decimal::ZERO);

    /// Formats the USD amount without artificial trailing zeroes.
    pub fn to_decimal_string(self) -> String {
        format_decimal(self.0)
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    /// Returns debt as a percentage of the available borrowing power.
    pub fn utilization_percent(self, borrowing_power: Self) -> Option<Decimal> {
        if borrowing_power.0 <= Decimal::ZERO {
            return None;
        }
        self.0
            .checked_div(borrowing_power.0)?
            .checked_mul(Decimal::new(100, 0))
    }

    /// Applies a collateral factor to a USD value.
    pub fn apply(self, factor: CollateralFactor) -> Option<Self> {
        self.0.checked_mul(factor.0).map(Self)
    }
}

fn format_decimal(value: Decimal) -> String {
    value.normalize().to_string()
}

impl CollateralFactor {
    /// Creates a factor from 0 through 1 inclusive; `0.8` represents 80%.
    pub fn new(value: Decimal) -> Option<Self> {
        (Decimal::ZERO..=Decimal::ONE)
            .contains(&value)
            .then_some(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decimal(value: &str) -> Decimal {
        value.parse().expect("valid decimal")
    }

    #[test]
    fn asset_quantity_times_usd_price_gives_usd_value() {
        let amount = Amount::new(decimal("1.5"));
        let price = Price::new(decimal("2000"));
        assert_eq!(amount.to_usd(price), Some(UsdValue(decimal("3000"))));
    }

    #[test]
    fn decimal_values_are_not_rescaled() {
        let amount = Amount::new(decimal("1500"));
        let one_dollar = Price::new(decimal("1"));
        assert_eq!(amount.to_usd(one_dollar), Some(UsdValue(decimal("1500"))));
    }

    #[test]
    fn decimal_arithmetic_reports_overflow() {
        let amount = Amount::new(Decimal::MAX);
        assert_eq!(amount.checked_add(Amount::new(Decimal::ONE)), None);
    }

    #[test]
    fn fractional_values_are_kept_exactly() {
        let dust = Amount::new(decimal("0.000001"));
        let price = Price::new(decimal("0.5"));
        assert_eq!(dust.to_usd(price), Some(UsdValue(decimal("0.0000005"))));
    }

    #[test]
    fn applying_eighty_percent_to_three_thousand_gives_two_thousand_four_hundred() {
        let factor = CollateralFactor::new(decimal("0.8")).expect("0.8 is valid");
        assert_eq!(
            UsdValue(decimal("3000")).apply(factor),
            Some(UsdValue(decimal("2400")))
        );
    }

    #[test]
    fn collateral_factor_must_be_between_zero_and_one() {
        assert!(CollateralFactor::new(decimal("1.01")).is_none());
        assert!(CollateralFactor::new(decimal("-0.01")).is_none());
    }

    #[test]
    fn midpoint_averages_decimal_prices() {
        assert_eq!(
            Price::new(decimal("3")).midpoint(Price::new(decimal("6"))),
            Some(Price::new(decimal("4.5")))
        );
    }
}
