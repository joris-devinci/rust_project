use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TickerDTO {
    pub price: String,
    pub volume: String,
}
