//! Drives the lending API over real HTTP, like an external client would.

use std::sync::Arc;

use reqwest::{Client, StatusCode};
use rust_decimal::Decimal;
use rust_project::{
    api::{AppState, router},
    domain::{Amount, Asset, CollateralFactor, Price, StaticFeed},
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
    borrow_utilization_percent: Option<String>,
    collateral: CollateralValueView,
    prices: Vec<AssetPriceView>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct CollateralValueView {
    total_value_usd: String,
    borrowing_power_usd: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct AssetPriceView {
    asset: Asset,
    price_usd: String,
}

fn decimal(value: &str) -> Decimal {
    value.parse().expect("valid decimal")
}

/// Starts the API on a free local port, with ETH at $2,000 and an 80% threshold.
async fn spawn_api() -> String {
    let feed = StaticFeed::new([
        (Asset::Eth, Price::new(decimal("2000"))),
        (Asset::Btc, Price::new(decimal("50000"))),
    ]);
    let protocol =
        Protocol::new(CollateralFactor::new(decimal("0.8")).expect("0.8 is a valid factor"));
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

async fn post(client: &Client, url: String, asset: Asset, amount: &str) -> StatusCode {
    let movement = Movement {
        asset,
        amount: Amount::new(decimal(amount)),
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
    account_view(client, account_url).await.health
}

async fn account_view(client: &Client, account_url: &str) -> AccountView {
    let view: AccountView = client
        .get(account_url)
        .send()
        .await
        .expect("the API answers")
        .json()
        .await
        .expect("body is an account view");
    view
}

#[test]
fn deposit_borrow_and_repay_over_http() {
    let runtime = Runtime::new().expect("a tokio runtime starts");
    runtime.block_on(async {
        let base = spawn_api().await;
        let client = Client::new();
        let account = open_account(&client, &base).await;

        // 1.5 ETH at $2,000 x 80% gives $2,400 of borrowing power
        let deposited = post(&client, format!("{account}/deposit"), Asset::Eth, "1.5").await;
        assert_eq!(deposited, StatusCode::NO_CONTENT);
        let view = account_view(&client, &account).await;
        assert_eq!(view.health, Health::NoDebt);
        assert_eq!(view.borrow_utilization_percent, None);
        assert_eq!(
            view.collateral,
            CollateralValueView {
                total_value_usd: "3000".to_owned(),
                borrowing_power_usd: "2400".to_owned(),
            }
        );
        assert_eq!(
            view.prices,
            vec![
                AssetPriceView {
                    asset: Asset::Btc,
                    price_usd: "50000".to_owned(),
                },
                AssetPriceView {
                    asset: Asset::Eth,
                    price_usd: "2000".to_owned(),
                },
                AssetPriceView {
                    asset: Asset::Usd,
                    price_usd: "1".to_owned(),
                },
            ]
        );

        let too_much = post(&client, format!("{account}/borrow"), Asset::Btc, "0.048001").await;
        assert_eq!(too_much, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(health(&client, &account).await, Health::NoDebt);

        let borrowed = post(&client, format!("{account}/borrow"), Asset::Btc, "0.03").await;
        assert_eq!(borrowed, StatusCode::NO_CONTENT);
        assert_eq!(health(&client, &account).await, Health::Safe);

        let repaid = post(&client, format!("{account}/repay"), Asset::Btc, "0.03").await;
        assert_eq!(repaid, StatusCode::NO_CONTENT);
        assert_eq!(health(&client, &account).await, Health::NoDebt);

        let borrowed = post(&client, format!("{account}/borrow"), Asset::Usd, "1500").await;
        assert_eq!(borrowed, StatusCode::NO_CONTENT);
        let view = account_view(&client, &account).await;
        assert_eq!(view.health, Health::Safe);
        assert_eq!(view.borrow_utilization_percent.as_deref(), Some("62.5"));

        let borrowed = post(&client, format!("{account}/borrow"), Asset::Usd, "300").await;
        assert_eq!(borrowed, StatusCode::NO_CONTENT);
        let view = account_view(&client, &account).await;
        assert_eq!(view.health, Health::Unsafe);
        assert_eq!(view.borrow_utilization_percent.as_deref(), Some("75"));

        let borrowed = post(&client, format!("{account}/borrow"), Asset::Usd, "360").await;
        assert_eq!(borrowed, StatusCode::NO_CONTENT);
        let view = account_view(&client, &account).await;
        assert_eq!(view.health, Health::Warning);
        assert_eq!(view.borrow_utilization_percent.as_deref(), Some("90"));

        let borrowed = post(&client, format!("{account}/borrow"), Asset::Usd, "240").await;
        assert_eq!(borrowed, StatusCode::NO_CONTENT);
        assert_eq!(health(&client, &account).await, Health::Warning);

        let too_much = post(&client, format!("{account}/borrow"), Asset::Usd, "0.01").await;
        assert_eq!(too_much, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(health(&client, &account).await, Health::Warning);

        let repaid = post(&client, format!("{account}/repay"), Asset::Usd, "2400").await;
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
