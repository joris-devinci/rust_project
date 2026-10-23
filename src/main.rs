
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TickerDTO {
    price: String,
    volume: String,
}

struct Ticker {
    name: String,
    price: i128,
    volume: i128
}

impl Ticker {
    fn new(name: String, ticker_dto: TickerDTO) -> Self {
        Self { name, price: (), volume: () }
    }
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    if let Err(error) = runtime.block_on(test()) {
        eprintln!("Error: {}", error);
    }
}

async fn test() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.exchange.coinbase.com/products/BTC-USD/ticker";

    let ticker: TickerDTO = reqwest::Client::new()
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    println!("BTC/USD last price: {}", ticker.price);
    println!("BTC/USD 24h volume: {}", ticker.volume);

    Ok(())
}
