use rust_decimal::Decimal;
use rust_project::{
    api::parse_price,
    domain::Price,
};

#[test]
fn parses_database_prices_without_rescaling() {
    assert_eq!(
        parse_price("1234.5"),
        Some(Price::new(Decimal::new(12345, 1)))
    );
    assert_eq!(
        parse_price("1.2345678900"),
        Some(Price::new(Decimal::new(123456789, 8)))
    );
}

#[test]
fn rejects_prices_that_cannot_be_represented_safely() {
    assert_eq!(parse_price("0"), None);
    assert_eq!(parse_price("-1"), None);
    assert_eq!(parse_price("not-a-price"), None);
}
