use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{Amount, Asset, Price},
    protocol::{AccountId, Health, Position},
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

#[derive(Serialize)]
struct AccountView {
    id: AccountId,
    health: Health,
    position: Position,
}

/// Body of deposit, borrow and repay.
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

async fn prices(State(state): State<AppState>) -> Json<Vec<AssetPrice>> {
    let prices = Asset::ALL
        .into_iter()
        .map(|asset| AssetPrice {
            asset,
            price: state.feed.price(asset).ok(),
        })
        .collect();
    Json(prices)
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
    let protocol = state.read()?;
    let health = protocol.health(id, state.feed.as_ref())?;
    let position = protocol.position(id)?.clone();
    Ok(Json(AccountView {
        id,
        health,
        position,
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
    state
        .write()?
        .borrow(id, movement.asset, movement.amount, state.feed.as_ref())?;
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
