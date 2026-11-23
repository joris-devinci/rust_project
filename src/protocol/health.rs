use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::{Amount, Asset, CollateralFactor, PriceFeed, UsdValue};

use super::error::LendingError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Health {
    /// There is no outstanding debt.
    NoDebt,
    /// Borrowing-power utilization is below 75%.
    Safe,
    /// Utilization is at least 75% but below 90%.
    Unsafe,
    /// Utilization is at least 90% and the position is nearing liquidation.
    Warning,
    /// Debt exceeds the available borrowing power.
    Liquidatable,
}

/// Informational health status and debt utilization of available borrowing power.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthReport {
    pub status: Health,
    pub utilization_percent: Option<Decimal>,
}

/// Borrowing power is `sum(collateral × price) × factor`; debt is valued the same way.
/// A position is liquidatable when total USD debt exceeds that borrowing power.
pub fn evaluate(
    collateral: &HashMap<Asset, Amount>,
    debt: &HashMap<Asset, Amount>,
    feed: &dyn PriceFeed,
    collateral_factor: CollateralFactor,
) -> Result<HealthReport, LendingError> {
    if debt.is_empty() {
        return Ok(HealthReport {
            status: Health::NoDebt,
            utilization_percent: None,
        });
    }
    let borrowing_power = total_usd(collateral, feed)?
        .apply(collateral_factor)
        .ok_or(LendingError::Overflow)?;
    let total_debt = total_usd(debt, feed)?;
    let utilization_percent = total_debt.utilization_percent(borrowing_power);
    let unsafe_threshold = Decimal::new(75, 0);
    let warning_threshold = Decimal::new(90, 0);
    let status = if total_debt > borrowing_power {
        Health::Liquidatable
    } else if utilization_percent.is_some_and(|percent| percent >= warning_threshold) {
        Health::Warning
    } else if utilization_percent.is_some_and(|percent| percent >= unsafe_threshold) {
        Health::Unsafe
    } else {
        Health::Safe
    };
    Ok(HealthReport {
        status,
        utilization_percent,
    })
}

pub(super) fn total_usd(
    balances: &HashMap<Asset, Amount>,
    feed: &dyn PriceFeed,
) -> Result<UsdValue, LendingError> {
    balances
        .iter()
        .try_fold(UsdValue::ZERO, |total, (&asset, &amount)| {
            let value = amount
                .to_usd(feed.price(asset)?)
                .ok_or(LendingError::Overflow)?;
            total.checked_add(value).ok_or(LendingError::Overflow)
        })
}

