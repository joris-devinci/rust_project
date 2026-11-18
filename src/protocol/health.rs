use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::{Amount, Asset, Bps, Price, PriceFeed, UsdValue};

use super::error::LendingError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Health {
    NoDebt,
    Safe,
    Liquidatable,
}

/// Liquidatable when the debt exceeds the collateral weighted by the liquidation threshold.
/// Collateral is rounded down and debt rounded up, so rounding always favours the protocol.
pub(super) fn evaluate(
    collateral: &HashMap<Asset, Amount>,
    debt: &HashMap<Asset, Amount>,
    feed: &dyn PriceFeed,
    liquidation_threshold: Bps,
) -> Result<Health, LendingError> {
    if debt.is_empty() {
        return Ok(Health::NoDebt);
    }
    let borrowing_power = total_usd(collateral, feed, Amount::to_usd)?
        .apply(liquidation_threshold)
        .ok_or(LendingError::Overflow)?;
    if total_usd(debt, feed, Amount::to_usd_rounded_up)? > borrowing_power {
        Ok(Health::Liquidatable)
    } else {
        Ok(Health::Safe)
    }
}

fn total_usd(
    balances: &HashMap<Asset, Amount>,
    feed: &dyn PriceFeed,
    convert: fn(Amount, Price) -> Option<UsdValue>,
) -> Result<UsdValue, LendingError> {
    balances
        .iter()
        .try_fold(UsdValue::ZERO, |total, (&asset, &amount)| {
            let value = convert(amount, feed.price(asset)?).ok_or(LendingError::Overflow)?;
            total.checked_add(value).ok_or(LendingError::Overflow)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{OracleError, StaticFeed};

    #[test]
    fn collateral_sums_across_assets() -> Result<(), LendingError> {
        let feed = StaticFeed::new([
            (Asset::Eth, Price::new(200_000_000_000)),   // $2,000
            (Asset::Btc, Price::new(5_000_000_000_000)), // $50,000
        ]);
        // 1 ETH + 0.1 BTC = $7,000, x 80% = $5,600 of borrowing power
        let collateral = HashMap::from([
            (Asset::Eth, Amount::new(1_000_000)),
            (Asset::Btc, Amount::new(100_000)),
        ]);
        let at_limit = evaluate(&collateral, &btc_debt(112_000), &feed, eighty_percent())?;
        let above = evaluate(&collateral, &btc_debt(112_001), &feed, eighty_percent())?;
        assert_eq!(at_limit, Health::Safe);
        assert_eq!(above, Health::Liquidatable);
        Ok(())
    }

    #[test]
    fn dust_debt_without_collateral_is_liquidatable() -> Result<(), LendingError> {
        let feed = StaticFeed::new([(Asset::Solana, Price::new(50_000_000))]); // $0.50
        let dust = HashMap::from([(Asset::Solana, Amount::new(1))]);
        let health = evaluate(&HashMap::new(), &dust, &feed, eighty_percent())?;
        assert_eq!(health, Health::Liquidatable);
        Ok(())
    }

    fn feed(eth_price: u128) -> StaticFeed {
        StaticFeed::new([
            (Asset::Eth, Price::new(eth_price)),
            (Asset::Btc, Price::new(5_000_000_000_000)),
        ])
    }

    fn eighty_percent() -> Bps {
        Bps::new(8_000).expect("80% is a valid ratio")
    }

    // 1.5 ETH of collateral
    fn collateral() -> HashMap<Asset, Amount> {
        HashMap::from([(Asset::Eth, Amount::new(1_500_000))])
    }

    fn btc_debt(raw: u128) -> HashMap<Asset, Amount> {
        HashMap::from([(Asset::Btc, Amount::new(raw))])
    }

    #[test]
    fn no_debt_is_never_liquidatable() -> Result<(), LendingError> {
        let health = evaluate(&collateral(), &HashMap::new(), &feed(1), eighty_percent())?;
        assert_eq!(health, Health::NoDebt);
        Ok(())
    }

    #[test]
    fn eth_at_2000_covers_1500_dollars_of_btc_debt() -> Result<(), LendingError> {
        // 0.03 BTC is $1,500; borrowing power is $3,000 x 80% = $2,400.
        let health = evaluate(
            &collateral(),
            &btc_debt(30_000),
            &feed(200_000_000_000),
            eighty_percent(),
        )?;
        assert_eq!(health, Health::Safe);
        Ok(())
    }

    #[test]
    fn eth_crash_to_1200_makes_it_liquidatable() -> Result<(), LendingError> {
        // borrowing power: $1,800 x 80% = $1,440 < $1,500 of BTC debt.
        let health = evaluate(
            &collateral(),
            &btc_debt(30_000),
            &feed(120_000_000_000),
            eighty_percent(),
        )?;
        assert_eq!(health, Health::Liquidatable);
        Ok(())
    }

    #[test]
    fn debt_equal_to_borrowing_power_is_still_safe() -> Result<(), LendingError> {
        let health = evaluate(
            &collateral(),
            &btc_debt(48_000),
            &feed(200_000_000_000),
            eighty_percent(),
        )?;
        assert_eq!(health, Health::Safe);
        Ok(())
    }

    #[test]
    fn missing_price_is_an_oracle_error() {
        let btc_only = StaticFeed::new([(Asset::Btc, Price::new(5_000_000_000_000))]);
        let result = evaluate(
            &collateral(),
            &btc_debt(30_000),
            &btc_only,
            eighty_percent(),
        );
        assert!(matches!(
            result,
            Err(LendingError::Oracle(OracleError::NoPrice(Asset::Eth)))
        ));
    }
}
