mod models;
use models::{ticker::Ticker, ticker_dto::TickerDTO};

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    if let Err(error) = runtime.block_on(test()) {
        eprintln!("Error: {}", error);
    }
}




async fn test() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.exchange.coinbase.com/products/BTC-USD/ticker";

    let ticker_dto: TickerDTO = reqwest::Client::new()
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let ticker = Ticker::new("BTC", "coinbase", ticker_dto)?;

    println!(
        "{:?}-{:?} last price: {}",
        ticker.asset, ticker.venue, ticker.price.0
    );
    println!(
        "{:?}-{:?} 24h volume: {}",
        ticker.asset, ticker.venue, ticker.volume.0
    );

    Ok(())
}

