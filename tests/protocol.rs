use rust_decimal::Decimal;
use rust_project::{
    domain::{Amount, Asset, CollateralFactor, Price, StaticFeed},
    protocol::{Health, LendingError, Position, Protocol},
};

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

fn protocol_with_eth() -> Result<(Protocol, rust_project::protocol::AccountId), LendingError> {
    let mut protocol = Protocol::new(factor());
    let id = protocol.open_account()?;
    protocol.deposit(id, Asset::Eth, Amount::new(decimal("1.5")))?;
    Ok((protocol, id))
}

#[test]
fn position_deposit_repay_and_validation_rules() -> Result<(), LendingError> {
    let mut position = Position::default();
    position.deposit(Asset::Eth, Amount::new(decimal("1")))?;
    position.deposit(Asset::Eth, Amount::new(decimal("0.5")))?;
    assert_eq!(
        position.health_report(&feed("2000"), factor())?.status,
        Health::NoDebt
    );

    assert!(matches!(
        position.deposit(Asset::Eth, Amount::new(decimal("-1"))),
        Err(LendingError::NegativeAmount)
    ));
    assert!(
        position
            .repay(Asset::Btc, Amount::new(decimal("-1")))
            .is_err()
    );
    Ok(())
}

#[test]
fn protocol_borrow_health_and_liquidation_rules() -> Result<(), LendingError> {
    let (mut protocol, id) = protocol_with_eth()?;
    protocol.borrow(id, Asset::Btc, Amount::new(decimal("0.03")), &feed("2000"))?;
    assert_eq!(protocol.health(id, &feed("2000"))?, Health::Safe);

    assert!(matches!(
        protocol.borrow(
            id,
            Asset::Btc,
            Amount::new(decimal("0.048001")),
            &feed("2000")
        ),
        Err(LendingError::WouldBeUnhealthy)
    ));

    let crashed = feed("1200");
    assert_eq!(protocol.liquidatable_accounts(&crashed)?, vec![id]);
    let liquidated = protocol.liquidate(id, &crashed)?;
    assert!(liquidated.collateral_value(&crashed).is_ok());
    Ok(())
}

#[test]
fn unknown_account_is_rejected() {
    let mut protocol = Protocol::new(factor());
    assert!(matches!(
        protocol.deposit(
            rust_project::protocol::AccountId(42),
            Asset::Eth,
            Amount::new(decimal("1"))
        ),
        Err(LendingError::UnknownAccount(_))
    ));
}
