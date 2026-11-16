use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::{Amount, Asset, Bps, PriceFeed};

use super::{error::LendingError, health::Health, position::Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(u64);

/// Every account of the lending protocol, with the rules applied to them.
#[derive(Debug)]
pub struct Protocol {
    positions: HashMap<AccountId, Position>,
    next_id: u64,
    liquidation_threshold: Bps,
}

impl Protocol {
    pub fn new(liquidation_threshold: Bps) -> Self {
        Self {
            positions: HashMap::new(),
            next_id: 0,
            liquidation_threshold,
        }
    }

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

    pub fn health(&self, id: AccountId, feed: &dyn PriceFeed) -> Result<Health, LendingError> {
        self.position(id)?.health(feed, self.liquidation_threshold)
    }

    pub fn deposit(
        &mut self,
        id: AccountId,
        asset: Asset,
        amount: Amount,
    ) -> Result<(), LendingError> {
        self.position_mut(id)?.deposit(asset, amount)
    }

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
        let liquidation_threshold = self.liquidation_threshold;
        let position = self.position_mut(id)?;
        let mut candidate = position.clone();
        candidate.add_debt(asset, amount)?;
        if candidate.health(feed, liquidation_threshold)? == Health::Liquidatable {
            return Err(LendingError::WouldBeUnhealthy);
        }
        *position = candidate;
        Ok(())
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

    fn feed() -> StaticFeed {
        StaticFeed::new([
            (Asset::Eth, Price::new(200_000_000_000)), // $2,000
            (Asset::Usdc, Price::new(100_000_000)),    // $1
        ])
    }

    // Protocol with an account holding 1.5 ETH of collateral, at an 80% threshold
    fn protocol_with_eth() -> Result<(Protocol, AccountId), LendingError> {
        let mut protocol = Protocol::new(Bps::new(8_000).expect("80% is a valid ratio"));
        let id = protocol.open_account()?;
        protocol.deposit(id, Asset::Eth, Amount::new(1_500_000))?;
        Ok((protocol, id))
    }

    #[test]
    fn accounts_get_distinct_ids() -> Result<(), LendingError> {
        let mut protocol = Protocol::new(Bps::new(8_000).expect("80% is a valid ratio"));
        assert_ne!(protocol.open_account()?, protocol.open_account()?);
        Ok(())
    }

    #[test]
    fn borrow_within_limit_succeeds() -> Result<(), LendingError> {
        let (mut protocol, id) = protocol_with_eth()?;
        protocol.borrow(id, Asset::Usdc, Amount::new(1_500_000_000), &feed())?;
        assert_eq!(protocol.health(id, &feed())?, Health::Safe);
        Ok(())
    }

    #[test]
    fn borrow_up_to_the_exact_limit_succeeds() -> Result<(), LendingError> {
        let (mut protocol, id) = protocol_with_eth()?;
        protocol.borrow(id, Asset::Usdc, Amount::new(2_400_000_000), &feed())?;
        assert_eq!(protocol.health(id, &feed())?, Health::Safe);
        Ok(())
    }

    #[test]
    fn borrow_beyond_limit_is_rejected_and_changes_nothing() -> Result<(), LendingError> {
        let (mut protocol, id) = protocol_with_eth()?;
        let result = protocol.borrow(id, Asset::Usdc, Amount::new(2_400_000_001), &feed());
        assert!(matches!(result, Err(LendingError::WouldBeUnhealthy)));
        assert_eq!(protocol.health(id, &feed())?, Health::NoDebt);
        Ok(())
    }

    #[test]
    fn unknown_account_is_rejected() {
        let mut protocol = Protocol::new(Bps::new(8_000).expect("80% is a valid ratio"));
        let result = protocol.deposit(AccountId(42), Asset::Eth, Amount::new(1));
        assert!(matches!(
            result,
            Err(LendingError::UnknownAccount(AccountId(42)))
        ));
    }
}
