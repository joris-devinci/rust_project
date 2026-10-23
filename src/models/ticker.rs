use super::{
    fields::{asset::Asset, price::Price, venue::Venue, volume::Volume},
    ticker_dto::TickerDTO,
};

use chrono::{DateTime, Utc};

pub struct Ticker {
    pub date: DateTime<Utc>,
    pub asset: Asset,
    pub venue: Venue,
    pub price: Price,
    pub volume: Volume,
}

impl Ticker {
    pub fn new(asset: Asset, venue: Venue, ticker_dto: TickerDTO) -> Result<Self, std::io::Error> {
        let date = Utc::now();
        let price = Price(ticker_dto.price);
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
