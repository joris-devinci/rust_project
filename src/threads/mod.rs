pub mod consumer;
pub mod producer;

pub mod queue {
    use crate::models::ticker::Ticker;
    use std::sync::{Arc, LazyLock, Mutex};

    pub static QUEUE: LazyLock<Arc<Mutex<Vec<Ticker>>>> =
        LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));
}
