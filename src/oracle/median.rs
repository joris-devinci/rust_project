use std::collections::HashMap;

use crate::domain::{Asset, Price};

/// Median of the prices, or `None` if there are none. With an even count, it is the
/// midpoint of the two middle prices. A single outlier, such as a bad tick, cannot move it.
///
/// Uses a partial selection in O(n) instead of a full sort, and reorders `prices`.
pub fn median(prices: &mut [Price]) -> Option<Price> {
    let len = prices.len();
    if len == 0 {
        return None;
    }
    let (lower, upper, _) = prices.select_nth_unstable(len / 2);
    let upper = *upper;
    if len % 2 == 1 {
        return Some(upper);
    }
    lower.iter().max().and_then(|&below| below.midpoint(upper))
}

/// Median price of each asset across its quotes, one quote per venue: a single venue
/// sending a wrong price cannot move the price used by the protocol.
pub fn median_by_asset(quotes: impl IntoIterator<Item = (Asset, Price)>) -> HashMap<Asset, Price> {
    let mut by_asset: HashMap<Asset, Vec<Price>> = HashMap::new();
    for (asset, price) in quotes {
        by_asset.entry(asset).or_default().push(price);
    }
    by_asset
        .into_iter()
        .filter_map(|(asset, mut prices)| median(&mut prices).map(|price| (asset, price)))
        .collect()
}
