use super::{
    fields::{asset::Asset, venue::Venue, volume::Volume},
    ticker_dto::TickerDTO,
};

use crate::domain::Price;
use chrono::{DateTime, Utc};

pub struct Ticker {
    pub date: DateTime<Utc>,
    pub asset: Asset,
    pub venue: Venue,
    pub price: Price,
    pub volume: Volume,
}

impl Ticker {
    pub fn new(
        asset: Asset,
        venue: Venue,
        ticker_dto: TickerDTO,
    ) -> Result<Self, rust_decimal::Error> {
        let date = Utc::now();
        let price = Price::new(ticker_dto.price.parse()?);
        let volume = Volume(ticker_dto.volume);

        Ok(Self {
            date,
            asset,
            venue,
            price,
            volume,
        })
    }
}
