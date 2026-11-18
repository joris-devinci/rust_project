//! Aggregation of the prices quoted by the venues the scraper reads.

mod median;

pub use median::{median, median_by_asset};
