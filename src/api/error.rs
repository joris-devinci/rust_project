use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

use crate::protocol::LendingError;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    Lending(#[from] LendingError),
    #[error("protocol state is unavailable: a request panicked while holding its lock")]
    LockPoisoned,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Lending(LendingError::UnknownAccount(_)) => StatusCode::NOT_FOUND,
            Self::Lending(LendingError::WouldBeUnhealthy | LendingError::RepayExceedsDebt(_)) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Self::Lending(LendingError::Overflow) => StatusCode::BAD_REQUEST,
            Self::Lending(LendingError::Oracle(_)) => StatusCode::SERVICE_UNAVAILABLE,
            Self::LockPoisoned => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = ErrorBody {
            error: self.to_string(),
        };
        (status, Json(body)).into_response()
    }
}
