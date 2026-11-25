"use client";

import { useEffect, useRef } from "react";

import type { Balances } from "@/lib/api";
import { formatQuantity, formatTime, formatUsd, SYMBOLS } from "@/lib/format";
import { debtUsd, entries, utilization } from "@/lib/position";
import type { Session } from "@/lib/session";

type Props = { liquidation: NonNullable<Session["liquidation"]>; onDismiss: () => void };

/** Before and after of a liquidation, kept until it is dismissed. */
export function LiquidationReport({ liquidation: { before, time }, onDismiss }: Props) {
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    // It mounts at the top of the page, possibly far from what the visitor is doing
    const calm = matchMedia("(prefers-reduced-motion: reduce)").matches;
    heading.current?.focus({ preventScroll: true });
    window.scrollTo({ top: 0, behavior: calm ? "auto" : "smooth" });
  }, []);

  const involved = entries({ ...before.position.collateral, ...before.position.debt }).map(([asset]) => asset);
  const prices = before.prices
    .filter(({ asset }) => involved.includes(asset))
    .map(({ asset, price_usd }) => `${SYMBOLS[asset]} ${formatUsd(price_usd)}`);
  const rows = [
    {
      label: "Collateral",
      before: list(before.position.collateral),
      value: formatUsd(before.collateral.total_value_usd),
      after: "Seized",
    },
    {
      label: "Debt",
      before: list(before.position.debt),
      value: formatUsd(debtUsd(before)),
      after: "Cancelled",
    },
    {
      label: "Power used",
      before: utilization(before),
      value: null,
      after: "No debt",
    },
  ];
  return (
    <section role="alert" className="flood on-river mb-6 rounded-lg bg-limit p-5 text-white sm:p-7">
      <div className="mb-4 flex flex-wrap items-start justify-between gap-4">
        <div>
          <h2 ref={heading} tabIndex={-1} className="text-3xl font-bold">Position liquidated</h2>
          <p>
            Account {before.id}, detected at {formatTime(time)}
          </p>
        </div>
        <button
          type="button"
          onClick={onDismiss}
          className="rounded-md bg-white px-4 py-2 font-semibold text-limit transition-colors hover:bg-stone"
        >
          Dismiss
        </button>
      </div>
      <table className="w-full text-left">
        <thead>
          <tr className="border-b border-white/50">
            <th className="py-2 pr-3 font-normal">
              <span className="sr-only">Item</span>
            </th>
            <th className="py-2 pr-3 font-normal">Last observed</th>
            <th className="py-2 font-normal">After</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.label} className="border-b border-white/30 align-top">
              <th scope="row" className="py-2.5 pr-3 font-semibold">
                {row.label}
              </th>
              <td className="py-2.5 pr-3">
                {row.before}
                {row.value && <span className="block text-sm">{row.value}</span>}
              </td>
              <td className="py-2.5 font-semibold">{row.after}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <p className="mt-4 max-w-prose">
        Last observed prices: {prices.join(", ")}. The quantities are exact. The API does not expose the price that
        triggered the keeper.
      </p>
    </section>
  );
}

function list(balances: Balances): string {
  const items = entries(balances).map(([asset, amount]) => `${formatQuantity(amount)} ${SYMBOLS[asset]}`);
  return items.length > 0 ? items.join(", ") : "None";
}
