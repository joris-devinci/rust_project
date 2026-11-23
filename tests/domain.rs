use rust_decimal::Decimal;
use rust_project::domain::{
    Amount, Asset, CollateralFactor, OracleError, Price, PriceFeed, StaticFeed, UsdValue,
};

fn decimal(value: &str) -> Decimal {
    value.parse().expect("valid decimal")
}

#[test]
fn decimal_money_operations() {
    let amount = Amount::new(decimal("1.5"));
    let price = Price::new(decimal("2000"));
    assert_eq!(amount.to_usd(price), Some(UsdValue::new(decimal("3000"))));
    assert_eq!(
        Amount::new(decimal("1500")).to_usd(Price::new(decimal("1"))),
        Some(UsdValue::new(decimal("1500")))
    );
    assert_eq!(
        Amount::new(Decimal::MAX).checked_add(Amount::new(Decimal::ONE)),
        None
    );

    let dust = Amount::new(decimal("0.000001"));
    assert_eq!(
        dust.to_usd(Price::new(decimal("0.5"))),
        Some(UsdValue::new(decimal("0.0000005")))
    );
}

#[test]
fn collateral_factor_and_midpoint_rules() {
    assert!(CollateralFactor::new(decimal("1.01")).is_none());
    assert!(CollateralFactor::new(decimal("-0.01")).is_none());
    assert_eq!(
        Price::new(decimal("3")).midpoint(Price::new(decimal("6"))),
        Some(Price::new(decimal("4.5")))
    );
    let factor = CollateralFactor::new(decimal("0.8")).expect("0.8 is valid");
    assert_eq!(
        UsdValue::new(decimal("3000")).apply(factor),
        Some(UsdValue::new(decimal("2400")))
    );
}

#[test]
fn static_feed_and_usd_price_rules() {
    let feed = StaticFeed::new([(Asset::Eth, Price::new(decimal("2000")))]);
    assert_eq!(
        feed.price(Asset::Eth).ok(),
        Some(Price::new(decimal("2000")))
    );
    assert!(matches!(
        StaticFeed::new([(Asset::Btc, Price::new(decimal("50000")))]).price(Asset::Eth),
        Err(OracleError::NoPrice(Asset::Eth))
    ));
    assert_eq!(
        StaticFeed::new([]).price(Asset::Usd).ok(),
        Some(Price::new(Decimal::ONE))
    );
}
