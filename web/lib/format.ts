import type Big from "big.js";

import type { Asset, Health } from "./api.ts";

export const ASSETS: Asset[] = ["Eth", "Btc", "Solana", "Usd"];

export const SYMBOLS: Record<Asset, string> = { Eth: "ETH", Btc: "BTC", Solana: "SOL", Usd: "USD" };

export const HEALTH_LABELS: Record<Health, string> = {
  NoDebt: "No debt",
  Safe: "Safe",
  Unsafe: "Unsafe",
  Warning: "Warning",
  Liquidatable: "Liquidatable",
};

const usd = new Intl.NumberFormat("en-US", { style: "currency", currency: "USD" });
const clock = new Intl.DateTimeFormat("en-GB", { timeStyle: "medium" });

/** For display only: calculations stay on exact decimals. */
export function formatUsd(value: Big | string): string {
  return usd.format(Number(value));
}

export function formatTime(time: number): string {
  return clock.format(time);
}

/** Groups thousands on the text itself: a number would drop decimals. */
export function formatQuantity(amount: string): string {
  const [whole, decimals] = amount.split(".");
  const grouped = whole.replace(/\B(?=(\d{3})+$)/g, ",");
  return decimals === undefined ? grouped : `${grouped}.${decimals}`;
}
