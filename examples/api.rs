//! Demo of the lending API with fixed prices, until the oracle is wired in.
//! Run it with `cargo run --example api`.

use std::{error::Error, sync::Arc, time::Duration};

use rust_project::{
    api::{AppState, router, run_keeper},
    domain::{Asset, Bps, Price, StaticFeed},
    protocol::Protocol,
};
use tokio::{net::TcpListener, runtime::Runtime};

const ADDRESS: &str = "127.0.0.1:3001";
const KEEPER_PERIOD: Duration = Duration::from_secs(5);

fn main() -> Result<(), Box<dyn Error>> {
    let feed = StaticFeed::new([
        (Asset::Eth, Price::new(200_000_000_000)),   // $2,000
        (Asset::Btc, Price::new(5_000_000_000_000)), // $50,000
        (Asset::Solana, Price::new(15_000_000_000)), // $150
        (Asset::Usdc, Price::new(100_000_000)),      // $1
    ]);
    let threshold = Bps::new(8_000).ok_or("80% is a valid ratio")?;
    let state = AppState::new(Protocol::new(threshold), Arc::new(feed));
    Runtime::new()?.block_on(serve(state))?;
    Ok(())
}

async fn serve(state: AppState) -> std::io::Result<()> {
    tokio::spawn(run_keeper(state.clone(), KEEPER_PERIOD));
    let listener = TcpListener::bind(ADDRESS).await?;
    println!("lending API listening on http://{ADDRESS}");
    axum::serve(listener, router(state)).await
}
