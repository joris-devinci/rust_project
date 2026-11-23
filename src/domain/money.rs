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

    pub fn new(value: Decimal) -> Self {
        Self(value)
    }

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
