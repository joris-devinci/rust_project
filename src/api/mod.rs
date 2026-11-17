mod error;
mod keeper;
mod routes;

use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{domain::PriceFeed, protocol::Protocol};

use error::ApiError;

pub use keeper::run_keeper;
pub use routes::router;

/// Shared by every request: the protocol behind a lock, and the price source.
#[derive(Clone)]
pub struct AppState {
    protocol: Arc<RwLock<Protocol>>,
    feed: Arc<dyn PriceFeed>,
}

impl AppState {
    pub fn new(protocol: Protocol, feed: Arc<dyn PriceFeed>) -> Self {
        Self {
            protocol: Arc::new(RwLock::new(protocol)),
            feed,
        }
    }

    fn read(&self) -> Result<RwLockReadGuard<'_, Protocol>, ApiError> {
        self.protocol.read().map_err(|_| ApiError::LockPoisoned)
    }

    fn write(&self) -> Result<RwLockWriteGuard<'_, Protocol>, ApiError> {
        self.protocol.write().map_err(|_| ApiError::LockPoisoned)
    }
}
