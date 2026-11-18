use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{Amount, Asset, OracleError, PriceFeed},
    protocol::{AccountId, Health, Position},
};

use super::{AppState, error::ApiError};

pub fn router(state: AppState) -> Router {
    Router::new()
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

#[derive(Serialize)]
struct AccountView {
    id: AccountId,
    health: Health,
    position: Position,
    prices: Vec<AssetPriceView>,
}

#[derive(Serialize)]
struct AssetPriceView {
    asset: Asset,
    price_usd: String,
}

/// Body of deposit, borrow and repay.
#[derive(Deserialize)]
struct Movement {
    asset: Asset,
    amount: Amount,
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
    let health = protocol.health(id, feed.as_ref())?;
    let position = protocol.position(id)?.clone();
    let prices = [Asset::Btc, Asset::Eth, Asset::Solana]
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
        position,
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
