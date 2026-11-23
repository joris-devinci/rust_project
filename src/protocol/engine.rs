use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::{Amount, Asset, CollateralFactor, PriceFeed, UsdValue};

use super::{
    error::LendingError,
    health::{Health, HealthReport},
    position::Position,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(pub u64);

/// In-memory account ledger and lending rules.
///
/// Collateral and debt are stored as decimal asset quantities; `collateral_factor` is
/// the maximum fraction of collateral value that debt may use.
#[derive(Debug)]
pub struct Protocol {
    positions: HashMap<AccountId, Position>,
    next_id: u64,
    collateral_factor: CollateralFactor,
}

/// Current collateral value and the amount that may be borrowed against it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollateralSummary {
    pub total_value_usd: UsdValue,
    pub borrowing_power_usd: UsdValue,
}

impl Protocol {
    /// Creates a protocol with the fraction of collateral value allowed to back debt.
    pub fn new(collateral_factor: CollateralFactor) -> Self {
        Self {
            positions: HashMap::new(),
            next_id: 0,
            collateral_factor,
        }
    }

    /// Opens an empty account and returns its identifier.
    pub fn open_account(&mut self) -> Result<AccountId, LendingError> {
        let id = AccountId(self.next_id);
        self.next_id = self.next_id.checked_add(1).ok_or(LendingError::Overflow)?;
        self.positions.insert(id, Position::default());
        Ok(id)
    }

    pub fn position(&self, id: AccountId) -> Result<&Position, LendingError> {
        self.positions
            .get(&id)
            .ok_or(LendingError::UnknownAccount(id))
    }

    /// Compares the account's USD debt with its price-weighted borrowing power.
    pub fn health(&self, id: AccountId, feed: &dyn PriceFeed) -> Result<Health, LendingError> {
        self.position(id)?.health(feed, self.collateral_factor)
    }

    /// Returns the health status and debt utilization percentage for an account.
    pub fn health_report(
        &self,
        id: AccountId,
        feed: &dyn PriceFeed,
    ) -> Result<HealthReport, LendingError> {
        self.position(id)?
            .health_report(feed, self.collateral_factor)
    }

    /// Values an account's collateral and applies the configured collateral factor.
    pub fn collateral_summary(
        &self,
        id: AccountId,
        feed: &dyn PriceFeed,
    ) -> Result<CollateralSummary, LendingError> {
        let total_value_usd = self.position(id)?.collateral_value(feed)?;
        let borrowing_power_usd = total_value_usd
            .apply(self.collateral_factor)
            .ok_or(LendingError::Overflow)?;
        Ok(CollateralSummary {
            total_value_usd,
            borrowing_power_usd,
        })
    }

    /// Adds collateral in ordinary asset units (for example, `1.5` ETH).
    pub fn deposit(
        &mut self,
        id: AccountId,
        asset: Asset,
        amount: Amount,
    ) -> Result<(), LendingError> {
        self.position_mut(id)?.deposit(asset, amount)
    }

    /// Reduces debt in ordinary asset units.
    pub fn repay(
        &mut self,
        id: AccountId,
        asset: Asset,
        amount: Amount,
    ) -> Result<(), LendingError> {
        self.position_mut(id)?.repay(asset, amount)
    }

    /// Applies the borrow to a copy first, and keeps it only if the position stays healthy.
    pub fn borrow(
        &mut self,
        id: AccountId,
        asset: Asset,
        amount: Amount,
        feed: &dyn PriceFeed,
    ) -> Result<(), LendingError> {
        let collateral_factor = self.collateral_factor;
        let position = self.position_mut(id)?;
        let mut candidate = position.clone();
        candidate.add_debt(asset, amount)?;
        if candidate.health(feed, collateral_factor)? == Health::Liquidatable {
            return Err(LendingError::WouldBeUnhealthy);
        }
        *position = candidate;
        Ok(())
    }

    /// Accounts whose position is liquidatable at the current prices.
    pub fn liquidatable_accounts(
        &self,
        feed: &dyn PriceFeed,
    ) -> Result<Vec<AccountId>, LendingError> {
        let mut accounts = Vec::new();
        for (&id, position) in &self.positions {
            if position.health(feed, self.collateral_factor)? == Health::Liquidatable {
                accounts.push(id);
            }
        }
        Ok(accounts)
    }

    /// Closes a liquidatable position: its debt is cancelled and its collateral is seized.
    /// Returns the seized position, and leaves the account open with nothing in it.
    pub fn liquidate(
        &mut self,
        id: AccountId,
        feed: &dyn PriceFeed,
    ) -> Result<Position, LendingError> {
        let collateral_factor = self.collateral_factor;
        let position = self.position_mut(id)?;
        if position.health(feed, collateral_factor)? != Health::Liquidatable {
            return Err(LendingError::NotLiquidatable(id));
        }
        Ok(std::mem::take(position))
    }

    fn position_mut(&mut self, id: AccountId) -> Result<&mut Position, LendingError> {
        self.positions
            .get_mut(&id)
            .ok_or(LendingError::UnknownAccount(id))
    }
}
