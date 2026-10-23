mod models;
use models::{Asset, Venue, Price, Volume, ticker::Ticker, ticker_dto::TickerDTO};
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};
use sqlx::postgres::PgPoolOptions;
use chrono::Utc;

static QUEUE: Mutex<Vec<Ticker>> = Mutex::new(Vec::new());
static ENDPOINTS: LazyLock<HashMap<u8, (Asset, Venue, String)>> = LazyLock::new(|| {
    HashMap::from([
        (0, (Asset::Btc, Venue::Coinbase, "https://api.exchange.coinbase.com/products/BTC-USD/ticker".to_string())),
        (1, (Asset::Eth, Venue::Coinbase, "https://api.exchange.coinbase.com/products/ETH-USD/ticker".to_string())),
        (2, (Asset::Solana, Venue::Coinbase, "https://api.exchange.coinbase.com/products/SOL-USD/ticker".to_string())),
    ])
});

fn main() {

    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    let tmp: Ticker = Ticker { date: Utc::now(), asset: Asset::Btc, venue: Venue::Coinbase, price: Price(1.to_string()), volume: Volume(1.to_string()) };
    QUEUE.lock().unwrap().push(tmp);

    runtime.block_on(async {
        let mut handles = Vec::new();

        for i in 0..3 {
            handles.push(tokio::spawn(producer(i)));
        }

        handles.push(tokio::spawn(consumer()));

        for handle in handles {
            if let Err(error) = handle.await.unwrap() {
                eprintln!("Request failed: {error}");
            }
        }
    });
}

async fn consumer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;

    loop {
        let (ticker, queue_len) = {
            let mut queue = QUEUE.lock().unwrap();
            (queue.pop().expect("Queue is empty"), queue.len())
        };
        println!("Queue len = {}", queue_len);

        sqlx::query(
            "INSERT INTO ticker (asset, venue, price, date)
            VALUES ($1, $2, $3::numeric, $4)"
        )
        .bind(ticker.asset.to_string())
        .bind(ticker.venue.to_string())
        .bind(ticker.price.0)
        .bind(ticker.date)
        .execute(&pool)
        .await?;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

async fn producer(tid: u8) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (asset, venue, url) = ENDPOINTS.get(&tid).expect("Unknown endpoint");
    loop {
        let ticker_dto: TickerDTO = reqwest::Client::new()
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let ticker = Ticker::new(*asset, *venue, ticker_dto)?;

        {
            let mut queue = QUEUE.lock().unwrap();
            queue.push(ticker);
        } 

        println!("TID {} added a ticker", tid);
        // println!("Queue len = {}", QUEUE.lock().unwrap().len());
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}
