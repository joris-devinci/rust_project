use rust_decimal::Decimal;
use rust_project::{
    domain::{Asset, Price},
    oracle::{median, median_by_asset},
};

fn decimal(value: &str) -> Decimal {
    value.parse().expect("valid decimal")
}

fn prices(values: &[&str]) -> Vec<Price> {
    values
        .iter()
        .map(|&value| Price::new(decimal(value)))
        .collect()
}

#[test]
fn median_selection_rules() {
    assert_eq!(median(&mut []), None);
    assert_eq!(
        median(&mut prices(&["30", "10", "20"])),
        Some(Price::new(decimal("20")))
    );
    assert_eq!(
        median(&mut prices(&["40", "10", "30", "20"])),
        Some(Price::new(decimal("25")))
    );
    let mut ticks = prices(&["2000", "2001", "1999", "2000", "1"]);
    assert_eq!(median(&mut ticks), Some(Price::new(decimal("2000"))));
}

#[test]
fn median_is_calculated_per_asset() {
    let medians = median_by_asset([
        (Asset::Eth, Price::new(decimal("2000"))),
        (Asset::Btc, Price::new(decimal("50000"))),
        (Asset::Eth, Price::new(decimal("2002"))),
        (Asset::Eth, Price::new(decimal("1"))),
    ]);
    assert_eq!(medians.get(&Asset::Eth), Some(&Price::new(decimal("2000"))));
    assert_eq!(
        medians.get(&Asset::Btc),
        Some(&Price::new(decimal("50000")))
    );
    assert_eq!(medians.get(&Asset::Solana), None);
}
