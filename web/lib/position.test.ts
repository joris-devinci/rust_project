import assert from "node:assert/strict";
import { test } from "node:test";

import type { Account, Balances } from "./api.ts";
import { debtUsd, marginUsd, maxBorrow, utilization, valueUsd, wasLiquidated } from "./position.ts";

// 1.5 ETH at $3,000 x 80% gives $3,600 of borrowing power
function account(debt: Balances, collateral: Balances = { Eth: "1.5" }): Account {
  return {
    id: 0,
    health: "Safe",
    borrow_utilization_percent: null,
    position: { collateral, debt },
    collateral: { total_value_usd: "4500", borrowing_power_usd: "3600" },
    prices: [
      { asset: "Btc", price_usd: "60000" },
      { asset: "Eth", price_usd: "3000" },
      { asset: "Usd", price_usd: "1" },
    ],
  };
}

test("a balance is worth its quantity times the price", () => {
  assert.equal(valueUsd(account({}), "Eth", "1.5")?.toFixed(), "4500");
});

test("a balance has no value without a price", () => {
  assert.equal(valueUsd(account({}), "Solana", "1"), null);
});

test("debt adds up across assets without rounding", () => {
  assert.equal(debtUsd(account({ Usd: "0.1", Btc: "0.0000033" })).toFixed(), "0.298");
});

test("margin is the borrowing power left", () => {
  assert.equal(marginUsd(account({ Usd: "3312" })).toFixed(), "288");
});

test("max borrow keeps a 0.1% margin for price moves", () => {
  assert.equal(maxBorrow(account({}), "Usd"), "3596.4");
});

test("max borrow rounds down to the asset's precision", () => {
  // $3,596.40 / $60,000 = 0.05994; $3,596.40 / $3,000 = 1.1988
  assert.equal(maxBorrow(account({}), "Btc"), "0.05994");
  // ($3,600 - $3,333.33) x 0.999 = $266.40333
  assert.equal(maxBorrow(account({ Usd: "3333.33" }), "Usd"), "266.4");
});

test("max borrow is zero without any power left", () => {
  assert.equal(maxBorrow(account({ Usd: "3700" }), "Usd"), "0");
});

test("max borrow is zero without a price", () => {
  assert.equal(maxBorrow(account({}), "Solana"), "0");
});

test("collateral that disappears is a liquidation", () => {
  assert.equal(wasLiquidated(account({ Usd: "3600" }), account({}, {})), true);
});

test("a full repayment is not a liquidation", () => {
  assert.equal(wasLiquidated(account({ Usd: "3600" }), account({})), false);
});

test("an account that was already empty is not liquidated", () => {
  assert.equal(wasLiquidated(account({}, {}), account({}, {})), false);
});

test("a first observation is not a liquidation", () => {
  assert.equal(wasLiquidated(null, account({}, {})), false);
});

test("another account's position is not compared", () => {
  const other = { ...account({}, {}), id: 1 };
  assert.equal(wasLiquidated(account({ Usd: "3600" }), other), false);
});

test("utilization rounds down below the limit", () => {
  // 3,599.9 / 3,600 = 99.9972%
  assert.equal(utilization({ ...account({ Usd: "3599.9" }), health: "Warning" }), "99.99%");
});

test("utilization rounds up past the limit", () => {
  assert.equal(utilization({ ...account({ Usd: "3600.01" }), health: "Liquidatable" }), "100.01%");
});

test("utilization is zero without debt", () => {
  assert.equal(utilization({ ...account({}), health: "NoDebt" }), "0%");
});

test("utilization is unbounded without borrowing power", () => {
  const worthless = { ...account({ Usd: "1" }), health: "Liquidatable" as const };
  worthless.collateral = { total_value_usd: "0", borrowing_power_usd: "0" };
  assert.equal(utilization(worthless), "Over 100%");
});
