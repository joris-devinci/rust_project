use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Venue {
    Coinbase,
    CoinGecko,
    Binance
}

impl Display for Venue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Venue::Coinbase => write!(f, "COINBASE"),
            Venue::CoinGecko => write!(f, "COINGECKO"),
            Venue::Binance => write!(f, "BINANCE"),
        }
    }
}
