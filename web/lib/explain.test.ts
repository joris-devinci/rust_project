import assert from "node:assert/strict";
import { test } from "node:test";

import { type Account, ApiError } from "./api.ts";
import { explain } from "./explain.ts";

const account: Account = {
  id: 0,
  health: "Safe",
  borrow_utilization_percent: "50",
  position: { collateral: { Eth: "1.5" }, debt: { Usd: "1800" } },
  collateral: { total_value_usd: "4500", borrowing_power_usd: "3600" },
  prices: [{ asset: "Usd", price_usd: "1" }],
};

const refusedBorrow = new ApiError(422, "borrow would make the position liquidatable");

test("a refused borrow tells the limit apart from what Max fills in", () => {
  assert.equal(
    explain(refusedBorrow, account, "Usd"),
    "Too much: $1,800.00 of borrowing power is left. Max fills in 1,798.2 USD, to leave room for price moves.",
  );
});

test("a borrow without collateral asks for a deposit", () => {
  const empty: Account = {
    ...account,
    position: { collateral: {}, debt: {} },
    collateral: { total_value_usd: "0", borrowing_power_usd: "0" },
  };
  assert.equal(explain(refusedBorrow, empty, "Usd"), "Deposit collateral before borrowing.");
});

test("a borrow at the limit asks for a repayment or collateral", () => {
  const full: Account = { ...account, position: { collateral: { Eth: "1.5" }, debt: { Usd: "3600" } } };
  assert.equal(explain(refusedBorrow, full, "Usd"), "No borrowing power is left: repay or deposit more collateral.");
});

test("a refused repayment says how much is owed", () => {
  const error = new ApiError(422, "repay exceeds the Usd debt");
  assert.equal(explain(error, account, "Usd"), "Too much: you owe 1,800 USD.");
});

test("other failures keep their message", () => {
  assert.equal(explain(new ApiError(0, "The web server is unreachable."), account, "Usd"), "The web server is unreachable.");
});
