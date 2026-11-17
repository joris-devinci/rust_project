use std::collections::HashMap;

use serde::Serialize;

use crate::domain::{Amount, Asset, Bps, PriceFeed};

use super::{
    error::LendingError,
    health::{self, Health},
};

/// Collateral and debt of one account, per asset.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Position {
    collateral: HashMap<Asset, Amount>,
    debt: HashMap<Asset, Amount>,
}

impl Position {
    /// Adds collateral. Public because it can only improve the health factor.
    pub fn deposit(&mut self, asset: Asset, amount: Amount) -> Result<(), LendingError> {
        let balance = self.collateral.entry(asset).or_insert(Amount::new(0));
        *balance = balance.checked_add(amount).ok_or(LendingError::Overflow)?;
        Ok(())
    }

    /// Reduces debt. Public because it can only improve the health factor.
    pub fn repay(&mut self, asset: Asset, amount: Amount) -> Result<(), LendingError> {
        let debt = self.debt.get(&asset).copied().unwrap_or(Amount::new(0));
        let remaining = debt
            .checked_sub(amount)
            .ok_or(LendingError::RepayExceedsDebt(asset))?;
        if remaining == Amount::new(0) {
            self.debt.remove(&asset);
        } else {
            self.debt.insert(asset, remaining);
        }
        Ok(())
    }

    /// Adds debt without checking health: only `Protocol::borrow`, which checks it, calls this.
    pub(super) fn add_debt(&mut self, asset: Asset, amount: Amount) -> Result<(), LendingError> {
        if amount == Amount::new(0) {
            return Ok(());
        }
        let balance = self.debt.entry(asset).or_insert(Amount::new(0));
        *balance = balance.checked_add(amount).ok_or(LendingError::Overflow)?;
        Ok(())
    }

    pub fn health(
        &self,
        feed: &dyn PriceFeed,
        liquidation_threshold: Bps,
    ) -> Result<Health, LendingError> {
        health::evaluate(&self.collateral, &self.debt, feed, liquidation_threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deposit_accumulates_collateral() -> Result<(), LendingError> {
        let mut position = Position::default();
        position.deposit(Asset::Eth, Amount::new(1_000_000))?;
        position.deposit(Asset::Eth, Amount::new(500_000))?;
        assert_eq!(
            position.collateral.get(&Asset::Eth),
            Some(&Amount::new(1_500_000))
        );
        Ok(())
    }

    #[test]
    fn deposit_overflow_leaves_balance_unchanged() -> Result<(), LendingError> {
        let mut position = Position::default();
        position.deposit(Asset::Eth, Amount::new(u128::MAX))?;
        assert!(matches!(
            position.deposit(Asset::Eth, Amount::new(1)),
            Err(LendingError::Overflow)
        ));
        assert_eq!(
            position.collateral.get(&Asset::Eth),
            Some(&Amount::new(u128::MAX))
        );
        Ok(())
    }

    #[test]
    fn partial_repay_reduces_debt() -> Result<(), LendingError> {
        let mut position = Position::default();
        position
            .debt
            .insert(Asset::Usdc, Amount::new(1_500_000_000));
        position.repay(Asset::Usdc, Amount::new(500_000_000))?;
        assert_eq!(
            position.debt.get(&Asset::Usdc),
            Some(&Amount::new(1_000_000_000))
        );
        Ok(())
    }

    #[test]
    fn full_repay_clears_debt() -> Result<(), LendingError> {
        let mut position = Position::default();
        position
            .debt
            .insert(Asset::Usdc, Amount::new(1_500_000_000));
        position.repay(Asset::Usdc, Amount::new(1_500_000_000))?;
        assert!(position.debt.is_empty());
        Ok(())
    }

    #[test]
    fn repay_more_than_debt_is_rejected() {
        let mut position = Position::default();
        position.debt.insert(Asset::Usdc, Amount::new(100_000_000));
        assert!(matches!(
            position.repay(Asset::Usdc, Amount::new(150_000_000)),
            Err(LendingError::RepayExceedsDebt(Asset::Usdc))
        ));
        assert_eq!(
            position.debt.get(&Asset::Usdc),
            Some(&Amount::new(100_000_000))
        );
    }
}
