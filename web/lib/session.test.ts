import assert from "node:assert/strict";
import { test } from "node:test";

import type { Account } from "./api.ts";
import { initialSession, MAX_POINTS, reduce, type Session } from "./session.ts";

function account(percent: string | null, collateral: Account["position"]["collateral"] = { Eth: "1" }): Account {
  return {
    id: 3,
    health: percent ? "Warning" : "NoDebt",
    borrow_utilization_percent: percent,
    position: { collateral, debt: percent ? { Usd: "2000" } : {} },
    collateral: { total_value_usd: "3000", borrowing_power_usd: "2400" },
    prices: [{ asset: "Eth", price_usd: "3000" }],
  };
}

function observed(session: Session, seen: Account, time: number): Session {
  return reduce(session, { type: "observed", account: seen, time });
}

test("each observation adds a point", () => {
  const session = observed(observed(initialSession, account("90"), 1), account("91.5"), 2);
  assert.deepEqual(session.points, [
    { time: 1, percent: 90 },
    { time: 2, percent: 91.5 },
  ]);
});

test("an account without debt is drawn at zero", () => {
  assert.deepEqual(observed(initialSession, account(null), 1).points, [{ time: 1, percent: 0 }]);
});

test("old points are dropped", () => {
  let session = initialSession;
  for (let time = 0; time <= MAX_POINTS; time++) session = observed(session, account("90"), time);
  assert.equal(session.points.length, MAX_POINTS);
  assert.equal(session.points[0].time, 1);
});

test("a liquidation freezes the last observed account", () => {
  const before = account("99.9");
  const session = observed(observed(initialSession, before, 1), account(null, {}), 2);
  assert.deepEqual(session.liquidation, { before, points: [{ time: 1, percent: 99.9 }], time: 2 });
});

test("the liquidation stays until it is dismissed", () => {
  const liquidated = observed(observed(initialSession, account("99.9"), 1), account(null, {}), 2);
  const later = observed(liquidated, account(null, {}), 3);
  assert.equal(later.liquidation?.time, 2);
  assert.equal(reduce(later, { type: "dismissed" }).liquidation, null);
});

test("a failure keeps the last account and marks it stale", () => {
  const session = reduce(observed(initialSession, account("90"), 1), { type: "failed", message: "no price" });
  assert.equal(session.account?.id, 3);
  assert.equal(session.error, "no price");
});

test("the next observation clears the failure", () => {
  const failed = reduce(initialSession, { type: "failed", message: "no price" });
  assert.equal(observed(failed, account("90"), 1).error, null);
});

test("a lost account is not a liquidation", () => {
  const session = reduce(observed(initialSession, account("99"), 1), { type: "lost" });
  assert.equal(session.account, null);
  assert.equal(session.liquidation, null);
  assert.equal(session.lost, true);
  assert.deepEqual(session.points, []);
});

test("another account starts a new history", () => {
  const session = observed(observed(initialSession, account("90"), 1), { ...account("10"), id: 4 }, 2);
  assert.deepEqual(session.points, [{ time: 2, percent: 10 }]);
});

test("a lost account keeps an undismissed liquidation", () => {
  const liquidated = observed(observed(initialSession, account("99.9"), 1), account(null, {}), 2);
  assert.equal(reduce(liquidated, { type: "lost" }).liquidation?.time, 2);
});
