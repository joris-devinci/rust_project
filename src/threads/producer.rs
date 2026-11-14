use crate::models::{ticker::Ticker, ticker_dto::TickerDTO, Asset, Venue};
use std::{
    collections::HashMap,
    sync::LazyLock,
    time::Duration,
};

const REQUESTS_PER_SECOND_LIMIT: f64 = 10.0;

static ENDPOINTS: LazyLock<HashMap<u8, (Asset, Venue, String)>> = LazyLock::new(|| {
    HashMap::from([
        (0, (Asset::Btc, Venue::Coinbase, "https://api.exchange.coinbase.com/products/BTC-USD/ticker".to_string())),
        (1, (Asset::Eth, Venue::Coinbase, "https://api.exchange.coinbase.com/products/ETH-USD/ticker".to_string())),
        (2, (Asset::Solana, Venue::Coinbase, "https://api.exchange.coinbase.com/products/SOL-USD/ticker".to_string())),
    ])
});

pub async fn producer(
    tid: u8,
    producer_count: u8,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let queue = crate::threads::queue::QUEUE.clone();
    let (asset, venue, url) = ENDPOINTS.get(&tid).expect("Unknown endpoint");
    
    let request_interval = Duration::from_secs_f64(f64::from(producer_count) / REQUESTS_PER_SECOND_LIMIT);
    
    let client = reqwest::Client::builder()
        .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
        .build()?;

    loop {
        let start_time = tokio::time::Instant::now();

        let ticker_dto: TickerDTO = client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let ticker = Ticker::new(*asset, *venue, ticker_dto)?;

        queue.lock().unwrap().push(ticker);

        println!("TID {} added a ticker", tid);

        let end_time = tokio::time::Instant::now();
        let elapsed = end_time.duration_since(start_time);
        if elapsed < request_interval {
            tokio::time::sleep(request_interval - elapsed).await;
        }
    }
}