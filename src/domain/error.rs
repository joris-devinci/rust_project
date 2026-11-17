use thiserror::Error;

use super::asset::Asset;

#[derive(Debug, Error)]
pub enum OracleError {
    #[error("no price for {0:?}")]
    NoPrice(Asset),
}
