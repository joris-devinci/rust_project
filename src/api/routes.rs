//! HTTP endpoints for accounts. Amounts and displayed prices use ordinary decimal units.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{Amount, Asset, OracleError, Price},
    protocol::{AccountId, CollateralSummary, Health, Position},
};

use super::{AppState, error::ApiError};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/prices", get(prices))
        .route("/accounts", post(open_account))
        .route("/accounts/{id}", get(account))
        .route("/accounts/{id}/deposit", post(deposit))
        .route("/accounts/{id}/borrow", post(borrow))
        .route("/accounts/{id}/repay", post(repay))
        .with_state(state)
}

#[derive(Serialize)]
struct OpenedAccount {
    id: AccountId,
}

/// Account snapshot; `prices` lists the current USD quote for each available asset.
#[derive(Serialize)]
struct AccountView {
    id: AccountId,
    health: Health,
    borrow_utilization_percent: Option<String>,
    position: Position,
    collateral: CollateralValueView,
    prices: Vec<AssetPriceView>,
}

#[derive(Serialize)]
struct CollateralValueView {
    total_value_usd: String,
    borrowing_power_usd: String,
}

impl From<CollateralSummary> for CollateralValueView {
    fn from(summary: CollateralSummary) -> Self {
        Self {
            total_value_usd: summary.total_value_usd.to_decimal_string(),
            borrowing_power_usd: summary.borrowing_power_usd.to_decimal_string(),
        }
    }
}

#[derive(Serialize)]
struct AssetPriceView {
    asset: Asset,
    price_usd: String,
}

/// Request body for deposit, borrow, and repay; `amount` is a decimal asset quantity.
#[derive(Deserialize)]
struct Movement {
    asset: Asset,
    amount: Amount,
}

#[derive(Serialize)]
struct AssetPrice {
    asset: Asset,
    /// `null` when the oracle has no recent price for the asset
    price: Option<Price>,
}

async fn prices(State(state): State<AppState>) -> Result<Json<Vec<AssetPrice>>, ApiError> {
    let feed = state.get_current_prices().await?;
    let prices = [Asset::Eth, Asset::Btc, Asset::Solana, Asset::Usd]
        .into_iter()
        .map(|asset| AssetPrice {
            asset,
            price: feed.price(asset).ok(),
        })
        .collect();
    Ok(Json(prices))
}

async fn open_account(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<OpenedAccount>), ApiError> {
    let id = state.write()?.open_account()?;
    Ok((StatusCode::CREATED, Json(OpenedAccount { id })))
}

async fn account(
    State(state): State<AppState>,
    Path(id): Path<AccountId>,
) -> Result<Json<AccountView>, ApiError> {
    state.read()?.position(id)?;
    let feed = state.get_current_prices().await?;
    let protocol = state.read()?;
    let health_report = protocol.health_report(id, feed.as_ref())?;
    let health = health_report.status;
    let borrow_utilization_percent = health_report
        .utilization_percent
        .map(|percent| percent.round_dp(2).normalize().to_string());
    let collateral = protocol.collateral_summary(id, feed.as_ref())?.into();
    let position = protocol.position(id)?.clone();
    let prices = [Asset::Btc, Asset::Eth, Asset::Solana, Asset::Usd]
        .into_iter()
        .filter_map(|asset| match feed.price(asset) {
            Ok(price) => Some(AssetPriceView {
                asset,
                price_usd: price.to_decimal_string(),
            }),
            Err(OracleError::NoPrice(_)) => None,
        })
        .collect();
    Ok(Json(AccountView {
        id,
        health,
        borrow_utilization_percent,
        position,
        collateral,
        prices,
    }))
}

async fn deposit(
    State(state): State<AppState>,
    Path(id): Path<AccountId>,
    Json(movement): Json<Movement>,
) -> Result<StatusCode, ApiError> {
    state
        .write()?
        .deposit(id, movement.asset, movement.amount)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn borrow(
    State(state): State<AppState>,
    Path(id): Path<AccountId>,
    Json(movement): Json<Movement>,
) -> Result<StatusCode, ApiError> {
    state.read()?.position(id)?;
    let feed = state.get_current_prices().await?;
    state
        .write()?
        .borrow(id, movement.asset, movement.amount, feed.as_ref())?;
    Ok(StatusCode::NO_CONTENT)
}

async fn repay(
    State(state): State<AppState>,
    Path(id): Path<AccountId>,
    Json(movement): Json<Movement>,
) -> Result<StatusCode, ApiError> {
    state.write()?.repay(id, movement.asset, movement.amount)?;
    Ok(StatusCode::NO_CONTENT)
}
