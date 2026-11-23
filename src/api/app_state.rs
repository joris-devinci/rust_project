use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use rust_decimal::Decimal;
use sqlx::{PgPool, Row};

use crate::{
    domain::{Asset, Price, PriceFeed, StaticFeed},
    oracle::median_by_asset,
    protocol::Protocol,
};

use super::error::ApiError;

#[derive(Clone)]
enum PriceSource {
    Static(Arc<dyn PriceFeed>),
    Database(PgPool),
}

/// Shared by every request: the protocol behind a lock and its price source.
#[derive(Clone)]
pub struct AppState {
    protocol: Arc<RwLock<Protocol>>,
    prices: PriceSource,
}

impl AppState {
    /// Creates state with a fixed feed, primarily for tests.
    pub fn new(protocol: Protocol, feed: Arc<dyn PriceFeed>) -> Self {
        Self {
            protocol: Arc::new(RwLock::new(protocol)),
            prices: PriceSource::Static(feed),
        }
    }

    /// Creates state that reads asset prices from the scraper's PostgreSQL tickers.
    pub fn with_database(protocol: Protocol, pool: PgPool) -> Self {
        Self {
            protocol: Arc::new(RwLock::new(protocol)),
            prices: PriceSource::Database(pool),
        }
    }

    /// Gets each venue's latest price, then uses the median across venues.
    pub(super) async fn get_current_prices(&self) -> Result<Arc<dyn PriceFeed>, ApiError> {
        match &self.prices {
            PriceSource::Static(feed) => Ok(Arc::clone(feed)),
            PriceSource::Database(pool) => {
                let rows = sqlx::query(
                    r#"
                    SELECT DISTINCT ON (asset, venue) asset, price::text AS price
                    FROM ticker
                    WHERE "date" > now() - CASE
                        WHEN venue = 'COINGECKO' THEN interval '90 seconds'
                        ELSE interval '30 seconds'
                    END
                    ORDER BY asset, venue, "date" DESC, id DESC
                    "#,
                )
                .fetch_all(pool)
                .await?;

                let prices = rows
                    .into_iter()
                    .map(|row| {
                        let asset_name: String = row.try_get("asset")?;
                        let price_text: String = row.try_get("price")?;
                        let asset = parse_asset(&asset_name)?;
                        let price = parse_price(&price_text).ok_or_else(|| {
                            ApiError::InvalidPriceData(format!(
                                "invalid price for {asset_name}: {price_text}"
                            ))
                        })?;
                        Ok((asset, price))
                    })
                    .collect::<Result<Vec<_>, ApiError>>()?;

                Ok(Arc::new(StaticFeed::new(median_by_asset(prices))))
            }
        }
    }

    pub fn read(&self) -> Result<RwLockReadGuard<'_, Protocol>, ApiError> {
        self.protocol.read().map_err(|_| ApiError::LockPoisoned)
    }

    pub fn write(&self) -> Result<RwLockWriteGuard<'_, Protocol>, ApiError> {
        self.protocol.write().map_err(|_| ApiError::LockPoisoned)
    }
}

pub fn parse_asset(value: &str) -> Result<Asset, ApiError> {
    match value {
        "BTC" => Ok(Asset::Btc),
        "ETH" => Ok(Asset::Eth),
        "SOL" => Ok(Asset::Solana),
        "USD" => Ok(Asset::Usd),
        _ => Err(ApiError::InvalidPriceData(format!(
            "unsupported ticker asset: {value}"
        ))),
    }
}

pub fn parse_price(value: &str) -> Option<Price> {
    let value = value.parse::<Decimal>().ok()?;
    (value > Decimal::ZERO).then_some(Price::new(value))
}
