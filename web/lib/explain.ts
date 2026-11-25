import { type Account, ApiError, type Asset } from "./api.ts";
import { formatQuantity, formatUsd, SYMBOLS } from "./format.ts";
import { entries, marginUsd, maxBorrow } from "./position.ts";

/** Turns a refused movement into what the visitor can do instead. */
export function explain(error: unknown, account: Account, asset: Asset): string {
  if (!(error instanceof Error)) return "Something went wrong.";
  if (!(error instanceof ApiError) || error.status !== 422) return error.message;
  if (error.message.includes("liquidatable")) return explainBorrow(account, asset);
  if (error.message.includes("exceeds")) {
    const owed = formatQuantity(account.position.debt[asset] ?? "0");
    return `Too much: you owe ${owed} ${SYMBOLS[asset]}.`;
  }
  return error.message;
}

function explainBorrow(account: Account, asset: Asset): string {
  if (entries(account.position.collateral).length === 0) return "Deposit collateral before borrowing.";
  const margin = marginUsd(account);
  if (margin.lte(0)) return "No borrowing power is left: repay or deposit more collateral.";
  const max = `${formatQuantity(maxBorrow(account, asset))} ${SYMBOLS[asset]}`;
  return `Too much: ${formatUsd(margin)} of borrowing power is left. Max fills in ${max}, to leave room for price moves.`;
}
