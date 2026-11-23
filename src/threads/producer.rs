use crate::models::{Asset, Venue, ticker::Ticker, ticker_dto::TickerDTO};
use std::{collections::HashMap, sync::LazyLock, time::Duration};

const REQUESTS_PER_SECOND_LIMIT_COINBASE: f64 = 10.0;
const REQUESTS_PER_SECOND_LIMIT_BINANCE: f64 = 3.0;

static ENDPOINTS: LazyLock<HashMap<u8, (Asset, Venue, String)>> = LazyLock::new(|| {
    let api_key = std::env::var("COINGECKO_API_KEY")
            .expect("COINGECKO_API_KEY must be set");

    HashMap::from([
        (0, (Asset::Btc, Venue::Coinbase, "https://api.exchange.coinbase.com/products/BTC-USD/ticker".to_string())),
        (1, (Asset::Eth, Venue::Coinbase, "https://api.exchange.coinbase.com/products/ETH-USD/ticker".to_string())),
        (2, (Asset::Solana, Venue::Coinbase, "https://api.exchange.coinbase.com/products/SOL-USD/ticker".to_string())),
        (3, (Asset::Btc, Venue::CoinGecko, format!("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_vol=true&x_cg_demo_api_key={api_key}"))),
        (4, (Asset::Eth, Venue::CoinGecko, format!("https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd&include_24hr_vol=true&x_cg_demo_api_key={api_key}"))),
        (5, (Asset::Solana, Venue::CoinGecko, format!("https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd&include_24hr_vol=true&x_cg_demo_api_key={api_key}"))),
        (6, (Asset::Btc, Venue::Binance, "https://data-api.binance.vision/api/v3/ticker/24hr?symbol=BTCUSDT".to_string())),
        (7, (Asset::Eth, Venue::Binance, "https://data-api.binance.vision/api/v3/ticker/24hr?symbol=ETHUSDT".to_string())),
        (8, (Asset::Solana, Venue::Binance, "https://data-api.binance.vision/api/v3/ticker/24hr?symbol=SOLUSDT".to_string())),
    ])
});

pub async fn producer(tid: u8) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let queue = crate::threads::queue::QUEUE.clone();
    let (asset, venue, url) = ENDPOINTS.get(&tid).expect("Unknown endpoint");

    let request_interval = Duration::from_secs_f64(match venue {
        Venue::Coinbase => 3.0 / REQUESTS_PER_SECOND_LIMIT_COINBASE,
        Venue::Binance => 3.0 / REQUESTS_PER_SECOND_LIMIT_BINANCE,
        Venue::CoinGecko => 120.0,
    });

    let client = reqwest::Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()?;

    loop {
        let start_time = tokio::time::Instant::now();

        let ticker_dto = get_ticket_dto(*venue, url, &client).await?;

        let ticker = Ticker::new(*asset, *venue, ticker_dto)?;

        queue.lock().unwrap().push(ticker);

        // println!("TID {} added a ticker", tid);

        let end_time = tokio::time::Instant::now();
        let elapsed = end_time.duration_since(start_time);

        if elapsed < request_interval {
            tokio::time::sleep(request_interval - elapsed).await;
        }
    }
}

pub async fn get_ticket_dto(
    venue: Venue,
    url: &str,
    client: &reqwest::Client,
) -> Result<TickerDTO, Box<dyn std::error::Error + Send + Sync>> {
    let response: serde_json::Value = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let quote = match venue {
        Venue::Coinbase => &response,
        Venue::CoinGecko => response
            .as_object()
            .and_then(|coins| coins.values().next())
            .ok_or("Empty CoinGecko response")?,
        Venue::Binance => &response,
    };

    let (price, volume) = match venue {
        Venue::Coinbase => ("price", "volume"),
        Venue::CoinGecko => ("usd", "usd_24h_vol"),
        Venue::Binance => ("lastPrice", "volume"),
    };

    Ok(TickerDTO {
        price: quote[price].to_string().trim_matches('"').to_string(),
        volume: quote[volume].to_string().trim_matches('"').to_string(),
    })
}
