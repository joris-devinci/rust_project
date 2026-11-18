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
    lower.iter().max().map(|&below| below.midpoint(upper))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn prices(raw: &[u128]) -> Vec<Price> {
        raw.iter().map(|&value| Price::new(value)).collect()
    }

    #[test]
    fn no_prices_have_no_median() {
        assert_eq!(median(&mut []), None);
    }

    #[test]
    fn odd_count_takes_the_middle_price() {
        assert_eq!(median(&mut prices(&[30, 10, 20])), Some(Price::new(20)));
    }

    #[test]
    fn even_count_takes_the_midpoint_of_the_two_middle_prices() {
        assert_eq!(median(&mut prices(&[40, 10, 30, 20])), Some(Price::new(25)));
    }

    #[test]
    fn one_outlier_does_not_move_the_median() {
        let mut ticks = prices(&[2_000, 2_001, 1_999, 2_000, 1]);
        assert_eq!(median(&mut ticks), Some(Price::new(2_000)));
    }

    #[test]
    fn each_asset_gets_the_median_of_its_venues() {
        let medians = median_by_asset([
            (Asset::Eth, Price::new(2_000)),
            (Asset::Btc, Price::new(50_000)),
            (Asset::Eth, Price::new(2_002)),
            (Asset::Eth, Price::new(1)), // one venue sending a wrong price
        ]);
        assert_eq!(medians.get(&Asset::Eth), Some(&Price::new(2_000)));
        assert_eq!(medians.get(&Asset::Btc), Some(&Price::new(50_000)));
        assert_eq!(medians.get(&Asset::Solana), None);
    }
}
