import type { Prices } from "@/lib/api";
import { ASSETS, formatUsd, SYMBOLS } from "@/lib/format";

export function PriceStrip({ prices, stale }: { prices: Prices; stale: boolean }) {
  return (
    <section aria-labelledby="prices" className="rounded-lg bg-panel p-5 sm:p-6">
      <h2 id="prices" className="text-lg font-semibold">
        Oracle prices
      </h2>
      <p className="mb-3 text-ink-soft">{stale ? "Last known values, not refreshed" : "Median of Coinbase, Binance and CoinGecko"}</p>
      <dl>
        {ASSETS.map((asset) => {
          const price = prices[asset];
          return (
            <div key={asset} className="flex justify-between gap-4 border-t border-stone-dark py-2.5">
              <dt className="font-semibold">{SYMBOLS[asset]}</dt>
              <dd>{price === undefined ? "No recent price" : formatUsd(price)}</dd>
            </div>
          );
        })}
      </dl>
    </section>
  );
}
