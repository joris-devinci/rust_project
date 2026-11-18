use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Asset {
    Eth,
    Btc,
    Solana,
    Usdc,
}

impl Asset {
    pub const ALL: [Self; 4] = [Self::Eth, Self::Btc, Self::Solana, Self::Usdc];
}
