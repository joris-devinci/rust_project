use thiserror::Error;

use crate::models::Asset;

#[derive(Debug, Error)]
pub enum OracleError {
    #[error("no price for {0:?}")]
    NoPrice(Asset),
}
