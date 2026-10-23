use thiserror::Error;

use crate::domain::Asset;

#[derive(Debug, Error)]
pub enum LendingError {
    #[error("arithmetic overflow")]
    Overflow,
    #[error("repay exceeds the {0:?} debt")]
    RepayExceedsDebt(Asset),
}
