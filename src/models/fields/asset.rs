use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub enum Asset {
    Eth,
    Btc,
    Solana,
}

impl Display for Asset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Asset::Eth => write!(f, "ETH"),
            Asset::Btc => write!(f, "BTC"),
            Asset::Solana => write!(f, "SOL"),
        }
    }
}
