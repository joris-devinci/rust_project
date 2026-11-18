use std::{collections::HashMap, sync::Arc, time::Duration};

use sqlx::PgPool;

use crate::domain::{Asset, Price};

use super::{feed::OracleFeed, median::median};

/// Tickers written by the scraper over the last minute, from every venue.
/// `price::text` because sqlx is built without its decimal feature.
const RECENT_TICKERS: &str =
    "SELECT asset, price::text FROM ticker WHERE date > now() - interval '60 seconds'";

/// Every `period`, recomputes the median price of each asset from the scraper's tickers.
/// If the query fails, the feed is emptied rather than left with stale prices.
pub async fn run_oracle(pool: PgPool, feed: Arc<OracleFeed>, period: Duration) {
    let mut interval = tokio::time::interval(period);
    loop {
        interval.tick().await;
        match sqlx::query_as::<_, (String, String)>(RECENT_TICKERS)
            .fetch_all(&pool)
            .await
        {
            Ok(rows) => feed.replace(medians_by_asset(rows)),
            Err(error) => {
                eprintln!("oracle: refresh failed: {error}");
                feed.replace(HashMap::new());
            }
        }
    }
}

/// Groups the `(symbol, price)` rows by asset and takes the median of each group.
/// Rows with an unknown symbol or an unreadable price are skipped.
fn medians_by_asset(rows: impl IntoIterator<Item = (String, String)>) -> HashMap<Asset, Price> {
    let mut by_asset: HashMap<Asset, Vec<Price>> = HashMap::new();
    for (symbol, price) in rows {
        if let (Some(asset), Some(price)) =
            (asset_from_symbol(&symbol), Price::from_decimal(&price))
        {
            by_asset.entry(asset).or_default().push(price);
        }
    }
    by_asset
        .into_iter()
        .filter_map(|(asset, mut prices)| median(&mut prices).map(|price| (asset, price)))
        .collect()
}

/// Symbols written by the scraper (its `Asset` display).
fn asset_from_symbol(symbol: &str) -> Option<Asset> {
    match symbol {
        "ETH" => Some(Asset::Eth),
        "BTC" => Some(Asset::Btc),
        "SOL" => Some(Asset::Solana),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(symbol: &str, price: &str) -> (String, String) {
        (symbol.to_string(), price.to_string())
    }

    #[test]
    fn each_asset_gets_the_median_of_its_rows() {
        let medians = medians_by_asset([
            row("ETH", "2000"),
            row("BTC", "50000"),
            row("ETH", "2002"),
            row("ETH", "1990"),
        ]);
        assert_eq!(medians.get(&Asset::Eth), Some(&Price::new(200_000_000_000)));
        assert_eq!(
            medians.get(&Asset::Btc),
            Some(&Price::new(5_000_000_000_000))
        );
        assert_eq!(medians.get(&Asset::Solana), None);
    }

    #[test]
    fn unknown_symbols_and_bad_prices_are_skipped() {
        let medians = medians_by_asset([row("DOGE", "0.1"), row("SOL", "oops")]);
        assert!(medians.is_empty());
    }
}
