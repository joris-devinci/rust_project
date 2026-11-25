import Big from "big.js";

import type { Account, Asset, Balances } from "./api.ts";

/** Part of the remaining power that Max borrows: prices may move before the request lands. */
const MAX_BORROW_SHARE = "0.999";

/** USD value of a balance, or null when the oracle has no price for the asset. */
export function valueUsd(account: Account, asset: Asset, amount: string): Big | null {
  const price = account.prices.find((quote) => quote.asset === asset);
  return price ? Big(amount).times(price.price_usd) : null;
}

/** The API refuses to describe an account whose debt has no price, so none is missing here. */
export function debtUsd(account: Account): Big {
  return entries(account.position.debt).reduce(
    (total, [asset, amount]) => total.plus(valueUsd(account, asset, amount) ?? 0),
    Big(0),
  );
}

/** Borrowing power left before liquidation; negative once the limit is crossed. */
export function marginUsd(account: Account): Big {
  return Big(account.collateral.borrowing_power_usd).minus(debtUsd(account));
}

/** Share of the borrowing power used. Rounding never crosses the limit the API decided on. */
export function utilization(account: Account): string {
  if (account.health === "NoDebt") return "0%";
  const power = Big(account.collateral.borrowing_power_usd);
  if (power.lte(0)) return "Over 100%";
  const rounding = account.health === "Liquidatable" ? Big.roundUp : Big.roundDown;
  return `${debtUsd(account).times(100).div(power).toFixed(2, rounding)}%`;
}

/** Largest amount of the asset that can be borrowed, rounded down. */
export function maxBorrow(account: Account, asset: Asset): string {
  const price = valueUsd(account, asset, "1");
  const margin = marginUsd(account);
  if (!price || margin.lte(0)) return "0";
  return margin
    .times(MAX_BORROW_SHARE)
    .div(price)
    .round(asset === "Usd" ? 2 : 6, Big.roundDown)
    .toFixed();
}

/** Nothing but the keeper can remove collateral: the protocol has no withdrawal. */
export function wasLiquidated(before: Account | null, after: Account): boolean {
  return (
    before?.id === after.id &&
    entries(before.position.collateral).length > 0 &&
    entries(after.position.collateral).length === 0
  );
}

export function entries(balances: Balances): [Asset, string][] {
  return Object.entries(balances) as [Asset, string][];
}
