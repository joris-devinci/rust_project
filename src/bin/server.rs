//! Lending API priced by the oracle: reads the scraper's tickers from Postgres.
//! Start the database and the scraper first (`docker compose up -d`, then `cargo run`),
//! then run this server with `cargo run --bin server`.

use std::{error::Error, sync::Arc, time::Duration};

use rust_project::{
    api::{AppState, router, run_keeper},
    domain::Bps,
    oracle::{OracleFeed, run_oracle},
    protocol::Protocol,
};
use sqlx::postgres::PgPoolOptions;
use tokio::{net::TcpListener, runtime::Runtime};

const ADDRESS: &str = "127.0.0.1:3001";
const ORACLE_PERIOD: Duration = Duration::from_secs(2);
const KEEPER_PERIOD: Duration = Duration::from_secs(5);

fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;
    let database_url = std::env::var("DATABASE_URL")?;
    let threshold = Bps::new(8_000).ok_or("80% is a valid ratio")?;
    Runtime::new()?.block_on(serve(&database_url, threshold))
}

async fn serve(database_url: &str, threshold: Bps) -> Result<(), Box<dyn Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(database_url)
        .await?;
    let feed = Arc::new(OracleFeed::default());
    tokio::spawn(run_oracle(pool, Arc::clone(&feed), ORACLE_PERIOD));

    let state = AppState::new(Protocol::new(threshold), feed);
    tokio::spawn(run_keeper(state.clone(), KEEPER_PERIOD));

    let listener = TcpListener::bind(ADDRESS).await?;
    println!("lending API listening on http://{ADDRESS}");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
