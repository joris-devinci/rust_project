use crate::models::{ticker::Ticker, ticker_dto::TickerDTO, Asset, Venue};
use std::{
    collections::HashMap,
    sync::{LazyLock},
};

static ENDPOINTS: LazyLock<HashMap<u8, (Asset, Venue, String)>> = LazyLock::new(|| {
    HashMap::from([
        (0, (Asset::Btc, Venue::Coinbase, "https://api.exchange.coinbase.com/products/BTC-USD/ticker".to_string())),
        (1, (Asset::Eth, Venue::Coinbase, "https://api.exchange.coinbase.com/products/ETH-USD/ticker".to_string())),
        (2, (Asset::Solana, Venue::Coinbase, "https://api.exchange.coinbase.com/products/SOL-USD/ticker".to_string())),
    ])
});

pub async fn producer(tid: u8) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let queue = crate::threads::queue::QUEUE.clone();
    
    let (asset, venue, url) = ENDPOINTS.get(&tid).expect("Unknown endpoint");
    let mut cnt = 0;
    loop {
        let ticker_dto: TickerDTO = reqwest::Client::new()
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let ticker = Ticker::new(*asset, *venue, ticker_dto)?;

        queue.lock().unwrap().push(ticker);

        cnt += 1;
        println!("TID {} added a {} ticker", tid, cnt);
        // tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}