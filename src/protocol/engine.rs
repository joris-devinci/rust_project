use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::{Amount, Asset, CollateralFactor, PriceFeed, UsdValue};

use super::{
    error::LendingError,
    health::{Health, HealthReport},
    position::Position,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(u64);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Price, StaticFeed};
    use rust_decimal::Decimal;

    fn decimal(value: &str) -> Decimal {
        value.parse().expect("valid decimal")
    }

    fn collateral_factor() -> CollateralFactor {
        CollateralFactor::new(decimal("0.8")).expect("0.8 is valid")
    }

    fn feed() -> StaticFeed {
        StaticFeed::new([
            (Asset::Eth, Price::new(decimal("2000"))),
            (Asset::Btc, Price::new(decimal("50000"))),
        ])
    }

    // Protocol with an account holding 1.5 ETH of collateral, at an 80% threshold
    fn protocol_with_eth() -> Result<(Protocol, AccountId), LendingError> {
        let mut protocol = Protocol::new(collateral_factor());
        let id = protocol.open_account()?;
        protocol.deposit(id, Asset::Eth, Amount::new(decimal("1.5")))?;
        Ok((protocol, id))
    }

    #[test]
    fn accounts_get_distinct_ids() -> Result<(), LendingError> {
        let mut protocol = Protocol::new(collateral_factor());
        assert_ne!(protocol.open_account()?, protocol.open_account()?);
        Ok(())
    }

    #[test]
    fn borrow_within_limit_succeeds() -> Result<(), LendingError> {
        let (mut protocol, id) = protocol_with_eth()?;
        protocol.borrow(id, Asset::Btc, Amount::new(decimal("0.03")), &feed())?;
        assert_eq!(protocol.health(id, &feed())?, Health::Safe);
        Ok(())
    }

    #[test]
    fn borrow_usd_uses_the_dollar_price() -> Result<(), LendingError> {
        let (mut protocol, id) = protocol_with_eth()?;
        protocol.borrow(id, Asset::Usd, Amount::new(decimal("2400")), &feed())?;
        assert_eq!(protocol.health(id, &feed())?, Health::Warning);

        assert!(matches!(
            protocol.borrow(id, Asset::Usd, Amount::new(decimal("0.01")), &feed()),
            Err(LendingError::WouldBeUnhealthy)
        ));
        assert_eq!(protocol.health(id, &feed())?, Health::Warning);
        Ok(())
    }

    #[test]
    fn borrow_up_to_the_exact_limit_succeeds() -> Result<(), LendingError> {
        let (mut protocol, id) = protocol_with_eth()?;
        protocol.borrow(id, Asset::Btc, Amount::new(decimal("0.048")), &feed())?;
        assert_eq!(protocol.health(id, &feed())?, Health::Warning);
        Ok(())
    }

    #[test]
    fn borrow_beyond_limit_is_rejected_and_changes_nothing() -> Result<(), LendingError> {
        let (mut protocol, id) = protocol_with_eth()?;
        let result = protocol.borrow(id, Asset::Btc, Amount::new(decimal("0.048001")), &feed());
        assert!(matches!(result, Err(LendingError::WouldBeUnhealthy)));
        assert_eq!(protocol.health(id, &feed())?, Health::NoDebt);
        Ok(())
    }

    // ETH crashes from $2,000 to $1,200: 1.5 ETH now backs only $1,440 of debt
    fn crashed_feed() -> StaticFeed {
        StaticFeed::new([
            (Asset::Eth, Price::new(decimal("1200"))),
            (Asset::Btc, Price::new(decimal("50000"))),
        ])
    }

    // Account with 1.5 ETH of collateral and 0.03 BTC ($1,500) of debt, safe at $2,000
    fn protocol_with_loan() -> Result<(Protocol, AccountId), LendingError> {
        let (mut protocol, id) = protocol_with_eth()?;
        protocol.borrow(id, Asset::Btc, Amount::new(decimal("0.03")), &feed())?;
        Ok((protocol, id))
    }

    #[test]
    fn only_unhealthy_accounts_are_listed() -> Result<(), LendingError> {
        let (mut protocol, borrower) = protocol_with_loan()?;
        let depositor = protocol.open_account()?;
        protocol.deposit(depositor, Asset::Eth, Amount::new(decimal("1")))?;
        assert!(protocol.liquidatable_accounts(&feed())?.is_empty());
        assert_eq!(
            protocol.liquidatable_accounts(&crashed_feed())?,
            vec![borrower]
        );
        Ok(())
    }

    #[test]
    fn healthy_account_cannot_be_liquidated() -> Result<(), LendingError> {
        let (mut protocol, id) = protocol_with_loan()?;
        let result = protocol.liquidate(id, &feed());
        assert!(matches!(result, Err(LendingError::NotLiquidatable(_))));
        assert_eq!(protocol.health(id, &feed())?, Health::Safe);
        Ok(())
    }

    #[test]
    fn liquidation_seizes_collateral_and_clears_debt() -> Result<(), LendingError> {
        let (mut protocol, id) = protocol_with_loan()?;
        let before = protocol.position(id)?.clone();
        let seized = protocol.liquidate(id, &crashed_feed())?;
        assert_eq!(seized, before);
        assert_eq!(protocol.position(id)?, &Position::default());
        Ok(())
    }

    #[test]
    fn unknown_account_is_rejected() {
        let mut protocol = Protocol::new(collateral_factor());
        let result = protocol.deposit(AccountId(42), Asset::Eth, Amount::new(decimal("1")));
        assert!(matches!(
            result,
            Err(LendingError::UnknownAccount(AccountId(42)))
        ));
    }
}
