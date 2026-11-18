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
pub(super) fn evaluate(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{OracleError, Price, StaticFeed};
    use rust_decimal::Decimal;

    fn decimal(value: &str) -> Decimal {
        value.parse().expect("valid decimal")
    }

    fn factor() -> CollateralFactor {
        CollateralFactor::new(decimal("0.8")).expect("0.8 is valid")
    }

    #[test]
    fn collateral_sums_across_assets() -> Result<(), LendingError> {
        let feed = StaticFeed::new([
            (Asset::Eth, Price::new(decimal("2000"))),
            (Asset::Btc, Price::new(decimal("50000"))),
        ]);
        // 1 ETH + 0.1 BTC = $7,000, x 80% = $5,600 of borrowing power
        let collateral = HashMap::from([
            (Asset::Eth, Amount::new(decimal("1"))),
            (Asset::Btc, Amount::new(decimal("0.1"))),
        ]);
        let at_limit = evaluate(&collateral, &btc_debt("0.112"), &feed, factor())?;
        let above = evaluate(&collateral, &btc_debt("0.112001"), &feed, factor())?;
        assert_eq!(at_limit.status, Health::Warning);
        assert_eq!(at_limit.utilization_percent, Some(decimal("100")));
        assert_eq!(above.status, Health::Liquidatable);
        Ok(())
    }

    #[test]
    fn classifies_unsafe_and_warning_utilization() -> Result<(), LendingError> {
        let feed = feed("2000");
        let unsafe_position = evaluate(&collateral(), &btc_debt("0.036"), &feed, factor())?;
        let warning = evaluate(&collateral(), &btc_debt("0.0432"), &feed, factor())?;

        assert_eq!(unsafe_position.status, Health::Unsafe);
        assert_eq!(unsafe_position.utilization_percent, Some(decimal("75")));
        assert_eq!(warning.status, Health::Warning);
        assert_eq!(warning.utilization_percent, Some(decimal("90")));
        Ok(())
    }

    #[test]
    fn dust_debt_without_collateral_is_liquidatable() -> Result<(), LendingError> {
        let feed = StaticFeed::new([(Asset::Solana, Price::new(decimal("0.5")))]);
        let dust = HashMap::from([(Asset::Solana, Amount::new(decimal("0.000001")))]);
        let health = evaluate(&HashMap::new(), &dust, &feed, factor())?;
        assert_eq!(health.status, Health::Liquidatable);
        assert_eq!(health.utilization_percent, None);
        Ok(())
    }

    fn feed(eth_price: &str) -> StaticFeed {
        StaticFeed::new([
            (Asset::Eth, Price::new(decimal(eth_price))),
            (Asset::Btc, Price::new(decimal("50000"))),
        ])
    }

    // 1.5 ETH of collateral
    fn collateral() -> HashMap<Asset, Amount> {
        HashMap::from([(Asset::Eth, Amount::new(decimal("1.5")))])
    }

    fn btc_debt(amount: &str) -> HashMap<Asset, Amount> {
        HashMap::from([(Asset::Btc, Amount::new(decimal(amount)))])
    }

    #[test]
    fn no_debt_is_never_liquidatable() -> Result<(), LendingError> {
        let health = evaluate(&collateral(), &HashMap::new(), &feed("1"), factor())?;
        assert_eq!(health.status, Health::NoDebt);
        assert_eq!(health.utilization_percent, None);
        Ok(())
    }

    #[test]
    fn eth_at_2000_covers_1500_dollars_of_btc_debt() -> Result<(), LendingError> {
        // 0.03 BTC is $1,500; borrowing power is $3,000 x 80% = $2,400.
        let health = evaluate(&collateral(), &btc_debt("0.03"), &feed("2000"), factor())?;
        assert_eq!(health.status, Health::Safe);
        assert_eq!(health.utilization_percent, Some(decimal("62.5")));
        Ok(())
    }

    #[test]
    fn eth_crash_to_1200_makes_it_liquidatable() -> Result<(), LendingError> {
        // borrowing power: $1,800 x 80% = $1,440 < $1,500 of BTC debt.
        let health = evaluate(&collateral(), &btc_debt("0.03"), &feed("1200"), factor())?;
        assert_eq!(health.status, Health::Liquidatable);
        Ok(())
    }

    #[test]
    fn debt_equal_to_borrowing_power_warns_but_is_not_liquidatable() -> Result<(), LendingError> {
        let health = evaluate(&collateral(), &btc_debt("0.048"), &feed("2000"), factor())?;
        assert_eq!(health.status, Health::Warning);
        Ok(())
    }

    #[test]
    fn missing_price_is_an_oracle_error() {
        let btc_only = StaticFeed::new([(Asset::Btc, Price::new(decimal("50000")))]);
        let result = evaluate(&collateral(), &btc_debt("0.03"), &btc_only, factor());
        assert!(matches!(
            result,
            Err(LendingError::Oracle(OracleError::NoPrice(Asset::Eth)))
        ));
    }
}
