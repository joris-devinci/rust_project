//! Drives the lending API over real HTTP, like an external client would.

use std::sync::Arc;

use reqwest::{Client, StatusCode};
use rust_project::{
    api::{AppState, router},
    domain::{Amount, Asset, Bps, Price, StaticFeed},
    protocol::{Health, Protocol},
};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, runtime::Runtime};

#[derive(Serialize)]
struct Movement {
    asset: Asset,
    amount: Amount,
}

#[derive(Deserialize)]
struct OpenedAccount {
    id: u64,
}

#[derive(Deserialize)]
struct AccountView {
    health: Health,
}

/// Starts the API on a free local port, with ETH at $2,000 and an 80% threshold.
async fn spawn_api() -> String {
    let feed = StaticFeed::new([
        (Asset::Eth, Price::new(200_000_000_000)),
        (Asset::Btc, Price::new(5_000_000_000_000)),
    ]);
    let protocol = Protocol::new(Bps::new(8_000).expect("80% is a valid ratio"));
    let app = router(AppState::new(protocol, Arc::new(feed)));
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a local port is free");
    let address = listener.local_addr().expect("the listener has an address");
    tokio::spawn(async move { axum::serve(listener, app).await });
    format!("http://{address}")
}

async fn open_account(client: &Client, base: &str) -> String {
    let response = client
        .post(format!("{base}/accounts"))
        .send()
        .await
        .expect("the API answers");
    assert_eq!(response.status(), StatusCode::CREATED);
    let opened: OpenedAccount = response.json().await.expect("body is an account id");
    format!("{base}/accounts/{}", opened.id)
}

async fn post(client: &Client, url: String, asset: Asset, raw_amount: u128) -> StatusCode {
    let movement = Movement {
        asset,
        amount: Amount::new(raw_amount),
    };
    client
        .post(url)
        .json(&movement)
        .send()
        .await
        .expect("the API answers")
        .status()
}

async fn health(client: &Client, account_url: &str) -> Health {
    let view: AccountView = client
        .get(account_url)
        .send()
        .await
        .expect("the API answers")
        .json()
        .await
        .expect("body is an account view");
    view.health
}

#[test]
fn deposit_borrow_and_repay_over_http() {
    let runtime = Runtime::new().expect("a tokio runtime starts");
    runtime.block_on(async {
        let base = spawn_api().await;
        let client = Client::new();
        let account = open_account(&client, &base).await;

        // 1.5 ETH at $2,000 x 80% gives $2,400 of borrowing power
        let deposited = post(&client, format!("{account}/deposit"), Asset::Eth, 1_500_000).await;
        assert_eq!(deposited, StatusCode::NO_CONTENT);
        assert_eq!(health(&client, &account).await, Health::NoDebt);

        let too_much = post(&client, format!("{account}/borrow"), Asset::Btc, 48_001).await;
        assert_eq!(too_much, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(health(&client, &account).await, Health::NoDebt);

        let borrowed = post(&client, format!("{account}/borrow"), Asset::Btc, 30_000).await;
        assert_eq!(borrowed, StatusCode::NO_CONTENT);
        assert_eq!(health(&client, &account).await, Health::Safe);

        let repaid = post(&client, format!("{account}/repay"), Asset::Btc, 30_000).await;
        assert_eq!(repaid, StatusCode::NO_CONTENT);
        assert_eq!(health(&client, &account).await, Health::NoDebt);
    });
}

#[test]
fn unknown_account_is_not_found() {
    let runtime = Runtime::new().expect("a tokio runtime starts");
    runtime.block_on(async {
        let base = spawn_api().await;
        let client = Client::new();
        let response = client
            .get(format!("{base}/accounts/42"))
            .send()
            .await
            .expect("the API answers");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    });
}
