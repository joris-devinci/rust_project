use rust_decimal::Decimal;
use rust_project::{
    domain::{Amount, Asset, CollateralFactor, OracleError, Price, StaticFeed},
    protocol::{Health, LendingError, evaluate},
};
use std::collections::HashMap;

fn decimal(value: &str) -> Decimal {
    value.parse().expect("valid decimal")
}

fn factor() -> CollateralFactor {
    CollateralFactor::new(decimal("0.8")).expect("0.8 is valid")
}

fn feed(eth_price: &str) -> StaticFeed {
    StaticFeed::new([
        (Asset::Eth, Price::new(decimal(eth_price))),
        (Asset::Btc, Price::new(decimal("50000"))),
    ])
}

fn collateral() -> HashMap<Asset, Amount> {
    HashMap::from([(Asset::Eth, Amount::new(decimal("1.5")))])
}

fn btc_debt(amount: &str) -> HashMap<Asset, Amount> {
    HashMap::from([(Asset::Btc, Amount::new(decimal(amount)))])
}

#[test]
fn health_statuses_for_debt_and_borrowing_power() -> Result<(), LendingError> {
    assert_eq!(
        evaluate(&collateral(), &HashMap::new(), &feed("2000"), factor())?.status,
        Health::NoDebt
    );
    assert_eq!(
        evaluate(&collateral(), &btc_debt("0.03"), &feed("2000"), factor())?.status,
        Health::Safe
    );
    assert_eq!(
        evaluate(&collateral(), &btc_debt("0.048"), &feed("2000"), factor())?.status,
        Health::Warning
    );
    assert_eq!(
        evaluate(&collateral(), &btc_debt("0.03"), &feed("1200"), factor())?.status,
        Health::Liquidatable
    );
    Ok(())
}

#[test]
fn missing_price_and_dust_debt_rules() -> Result<(), LendingError> {
    let btc_only = StaticFeed::new([(Asset::Btc, Price::new(decimal("50000")))]);
    assert!(matches!(
        evaluate(&collateral(), &btc_debt("0.03"), &btc_only, factor()),
        Err(LendingError::Oracle(OracleError::NoPrice(Asset::Eth)))
    ));

    let sol_feed = StaticFeed::new([(Asset::Solana, Price::new(decimal("0.5")))]);
    let dust = HashMap::from([(Asset::Solana, Amount::new(decimal("0.000001")))]);
    let report = evaluate(&HashMap::new(), &dust, &sol_feed, factor())?;
    assert_eq!(report.status, Health::Liquidatable);
    Ok(())
}
