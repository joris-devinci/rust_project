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
    pub fn new(asset: &str, venue: &str, ticker_dto: TickerDTO) -> Result<Self, std::io::Error> {
        
        let date = Utc::now();

        let asset = match asset.to_lowercase().as_str() {
            "eth" => Asset::Eth,
            "btc" => Asset::Btc,
            "sol" => Asset::Solana,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unsupported asset: {asset}"),
                ));
            }
        };

        let venue = match venue.to_lowercase().as_str() {
            "coinbase" => Venue::Coinbase,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unsupported venue: {venue}"),
                ));
            }
        };

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
