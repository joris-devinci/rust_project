use std::collections::HashMap;

use serde::Serialize;

use crate::domain::{Amount, Asset, CollateralFactor, PriceFeed, UsdValue};

use super::{
    error::LendingError,
    health::{self, Health, HealthReport},
};

/// An account's decimal collateral and debt quantities, grouped by asset.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Position {
    collateral: HashMap<Asset, Amount>,
    debt: HashMap<Asset, Amount>,
}

impl Position {
    /// Adds a non-negative quantity of collateral.
    pub fn deposit(&mut self, asset: Asset, amount: Amount) -> Result<(), LendingError> {
        if amount.is_negative() {
            return Err(LendingError::NegativeAmount);
        }
        let balance = self.collateral.entry(asset).or_insert(Amount::ZERO);
        *balance = balance.checked_add(amount).ok_or(LendingError::Overflow)?;
        Ok(())
    }

    /// Reduces debt; repaying more than the balance is rejected.
    pub fn repay(&mut self, asset: Asset, amount: Amount) -> Result<(), LendingError> {
        if amount.is_negative() {
            return Err(LendingError::NegativeAmount);
        }
        let debt = self.debt.get(&asset).copied().unwrap_or(Amount::ZERO);
        if amount > debt {
            return Err(LendingError::RepayExceedsDebt(asset));
        }
        let remaining = debt.checked_sub(amount).ok_or(LendingError::Overflow)?;
        if remaining == Amount::ZERO {
            self.debt.remove(&asset);
        } else {
            self.debt.insert(asset, remaining);
        }
        Ok(())
    }

    /// Adds debt without checking health: only `Protocol::borrow`, which checks it, calls this.
    pub(super) fn add_debt(&mut self, asset: Asset, amount: Amount) -> Result<(), LendingError> {
        if amount.is_negative() {
            return Err(LendingError::NegativeAmount);
        }
        if amount == Amount::ZERO {
            return Ok(());
        }
        let balance = self.debt.entry(asset).or_insert(Amount::ZERO);
        *balance = balance.checked_add(amount).ok_or(LendingError::Overflow)?;
        Ok(())
    }

    pub fn health(
        &self,
        feed: &dyn PriceFeed,
        collateral_factor: CollateralFactor,
    ) -> Result<Health, LendingError> {
        Ok(self.health_report(feed, collateral_factor)?.status)
    }

    pub fn health_report(
        &self,
        feed: &dyn PriceFeed,
        collateral_factor: CollateralFactor,
    ) -> Result<HealthReport, LendingError> {
        health::evaluate(&self.collateral, &self.debt, feed, collateral_factor)
    }

    /// Values all collateral at the supplied feed's current USD prices.
    pub fn collateral_value(&self, feed: &dyn PriceFeed) -> Result<UsdValue, LendingError> {
        health::total_usd(&self.collateral, feed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn decimal(value: &str) -> Decimal {
        value.parse().expect("valid decimal")
    }

    #[test]
    fn deposit_accumulates_collateral() -> Result<(), LendingError> {
        let mut position = Position::default();
        position.deposit(Asset::Eth, Amount::new(decimal("1")))?;
        position.deposit(Asset::Eth, Amount::new(decimal("0.5")))?;
        assert_eq!(
            position.collateral.get(&Asset::Eth),
            Some(&Amount::new(decimal("1.5")))
        );
        Ok(())
    }

    #[test]
    fn deposit_overflow_leaves_balance_unchanged() -> Result<(), LendingError> {
        let mut position = Position::default();
        position.deposit(Asset::Eth, Amount::new(Decimal::MAX))?;
        assert!(matches!(
            position.deposit(Asset::Eth, Amount::new(Decimal::ONE)),
            Err(LendingError::Overflow)
        ));
        assert_eq!(
            position.collateral.get(&Asset::Eth),
            Some(&Amount::new(Decimal::MAX))
        );
        Ok(())
    }

    #[test]
    fn partial_repay_reduces_debt() -> Result<(), LendingError> {
        let mut position = Position::default();
        position
            .debt
            .insert(Asset::Btc, Amount::new(decimal("0.03")));
        position.repay(Asset::Btc, Amount::new(decimal("0.01")))?;
        assert_eq!(
            position.debt.get(&Asset::Btc),
            Some(&Amount::new(decimal("0.02")))
        );
        Ok(())
    }

    #[test]
    fn full_repay_clears_debt() -> Result<(), LendingError> {
        let mut position = Position::default();
        position
            .debt
            .insert(Asset::Btc, Amount::new(decimal("0.03")));
        position.repay(Asset::Btc, Amount::new(decimal("0.03")))?;
        assert!(position.debt.is_empty());
        Ok(())
    }

    #[test]
    fn repay_more_than_debt_is_rejected() {
        let mut position = Position::default();
        position
            .debt
            .insert(Asset::Btc, Amount::new(decimal("0.001")));
        assert!(matches!(
            position.repay(Asset::Btc, Amount::new(decimal("0.002"))),
            Err(LendingError::RepayExceedsDebt(Asset::Btc))
        ));
        assert_eq!(
            position.debt.get(&Asset::Btc),
            Some(&Amount::new(decimal("0.001")))
        );
    }

    #[test]
    fn negative_amounts_are_rejected() {
        let mut position = Position::default();
        assert!(matches!(
            position.deposit(Asset::Eth, Amount::new(decimal("-1"))),
            Err(LendingError::NegativeAmount)
        ));
        assert!(matches!(
            position.repay(Asset::Btc, Amount::new(decimal("-1"))),
            Err(LendingError::NegativeAmount)
        ));
    }
}
