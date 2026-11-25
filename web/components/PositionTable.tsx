import type { Account, Asset, Balances } from "@/lib/api";
import { ASSETS, formatQuantity, formatUsd, SYMBOLS } from "@/lib/format";
import { debtUsd, valueUsd } from "@/lib/position";

export function PositionTable({ account }: { account: Account }) {
  const { collateral, debt } = account.position;
  const held = ASSETS.filter((asset) => collateral[asset] ?? debt[asset]);
  return (
    <section aria-labelledby="position" className="rounded-lg bg-panel p-5 sm:p-6">
      <h2 id="position" className="mb-3 text-lg font-semibold">
        Position
      </h2>
      {held.length === 0 ? (
        <p className="max-w-prose text-ink-soft">
          Nothing here yet. Deposit ETH, BTC or SOL to get borrowing power: their price moves, so the level will too.
        </p>
      ) : (
        <table className="w-full table-fixed text-right">
          <thead className="text-ink-soft">
            <tr className="border-b border-stone-dark">
              <th className="py-2 text-left font-normal">Asset</th>
              <th className="py-2 font-normal">Collateral</th>
              <th className="py-2 font-normal">Debt</th>
            </tr>
          </thead>
          <tbody>
            {held.map((asset) => (
              <tr key={asset} className="border-b border-stone-dark align-top">
                <th scope="row" className="py-2.5 text-left font-semibold">
                  {SYMBOLS[asset]}
                </th>
                <BalanceCell account={account} asset={asset} balances={collateral} />
                <BalanceCell account={account} asset={asset} balances={debt} />
              </tr>
            ))}
          </tbody>
          <tfoot className="font-semibold">
            <tr>
              <th scope="row" className="pt-3 text-left">
                Total
              </th>
              <td className="pt-3">{formatUsd(account.collateral.total_value_usd)}</td>
              <td className="pt-3">{formatUsd(debtUsd(account))}</td>
            </tr>
          </tfoot>
        </table>
      )}
    </section>
  );
}

function BalanceCell({ account, asset, balances }: { account: Account; asset: Asset; balances: Balances }) {
  const amount = balances[asset];
  if (amount === undefined) return <td className="py-2.5 text-ink-soft">0</td>;
  const value = valueUsd(account, asset, amount);
  return (
    <td className="py-2.5 break-all">
      {formatQuantity(amount)}
      {value && <span className="block text-sm text-ink-soft">{formatUsd(value)}</span>}
    </td>
  );
}
