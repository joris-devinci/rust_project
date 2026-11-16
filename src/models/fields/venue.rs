use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub enum Venue {
    Coinbase,
}

impl Display for Venue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Venue::Coinbase => write!(f, "COINBASE"),
        }
    }
}
