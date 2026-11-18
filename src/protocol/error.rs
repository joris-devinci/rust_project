use thiserror::Error;

use crate::domain::{Asset, OracleError};

use super::engine::AccountId;

#[derive(Debug, Error)]
pub enum LendingError {
    #[error("arithmetic overflow")]
    Overflow,
    #[error("amount cannot be negative")]
    NegativeAmount,
    #[error("repay exceeds the {0:?} debt")]
    RepayExceedsDebt(Asset),
    #[error("unknown account {0:?}")]
    UnknownAccount(AccountId),
    #[error("borrow would make the position liquidatable")]
    WouldBeUnhealthy,
    #[error("account {0:?} is healthy and cannot be liquidated")]
    NotLiquidatable(AccountId),
    #[error(transparent)]
    Oracle(#[from] OracleError),
}
