"use client";

import { useState } from "react";

import { ActionForm } from "@/components/ActionForm";
import { Gauge } from "@/components/Gauge";
import { LiquidationReport } from "@/components/LiquidationReport";
import { PositionTable } from "@/components/PositionTable";
import { PriceStrip } from "@/components/PriceStrip";
import { UtilizationPanel } from "@/components/UtilizationPanel";
import type { Account, Prices } from "@/lib/api";
import { FULL_SCALE } from "@/lib/chart";
import { messageOf, useAccount } from "@/lib/useAccount";

export default function Home() {
  const { session, refresh, open, move, dismiss } = useAccount();
  const { account, points, liquidation } = session;
  const prices = account ? pricesOf(account) : session.prices;
  const [opening, setOpening] = useState(false);
  const [openError, setOpenError] = useState<string | null>(null);

  async function openAccount() {
    setOpening(true);
    setOpenError(null);
    try {
      await open();
    } catch (failure) {
      setOpenError(messageOf(failure));
    } finally {
      setOpening(false);
    }
  }

  const error = openError ?? session.error;
  const stale = session.error !== null;
  return (
    <>
      <header className="on-river bg-river text-white">
        <div className="mx-auto flex max-w-7xl flex-wrap items-center gap-x-8 gap-y-2 px-4 py-4 sm:px-8">
          <h1 className="text-xl font-bold">Lending simulation</h1>
          {account && <span>Account {account.id}</span>}
          <span className="sm:ml-auto">Simulation, no real funds</span>
          {account && (
            <button
              type="button"
              onClick={openAccount}
              disabled={opening}
              className="rounded-md bg-white/15 px-3 py-1.5 font-semibold transition-colors hover:bg-white/25 disabled:opacity-60"
            >
              Open another account
            </button>
          )}
        </div>
      </header>
      <main className="mx-auto max-w-7xl px-4 py-6 sm:px-8 sm:py-8">
        {session.lost && (
          <p role="status" className="mb-6 rounded-lg bg-amber-pale px-5 py-4">
            This account no longer exists: the server restarted and accounts live in its memory. Open a new one.
          </p>
        )}
        {error && (
          <p role="alert" className="mb-6 rounded-lg bg-amber-pale px-5 py-4">
            {error} {account && session.error && "The values below are the last ones received."}
          </p>
        )}
        {liquidation && <LiquidationReport liquidation={liquidation} onDismiss={dismiss} />}
        {/* On a phone the two columns merge, and the form comes right after the level */}
        <div className="grid gap-6 lg:grid-cols-[minmax(0,2fr)_minmax(0,1fr)] lg:gap-8">
          <div className="contents lg:grid lg:content-start lg:gap-6">
            {account ? (
              <>
                <div className="order-1 min-w-0">
                  <UtilizationPanel account={account} points={points} liquidation={liquidation} stale={stale} />
                </div>
                <div className="order-3 min-w-0">
                  <PositionTable account={account} />
                </div>
              </>
            ) : (
              (prices || error === null) && <Welcome ready={prices !== null && !opening} onOpen={openAccount} />
            )}
          </div>
          <div className="contents lg:grid lg:content-start lg:gap-6">
            {account && (
              <div className="order-2 min-w-0">
                <ActionForm key={account.id} account={account} refresh={refresh} move={move} />
              </div>
            )}
            {prices && (
              <div className="order-4 min-w-0">
                <PriceStrip prices={prices} stale={stale} />
              </div>
            )}
          </div>
        </div>
      </main>
    </>
  );
}

function Welcome({ ready, onOpen }: { ready: boolean; onOpen: () => void }) {
  return (
    <section className="on-river overflow-hidden rounded-lg bg-river text-white">
      <div className="p-5 sm:p-7">
        <h2 className="mb-4 text-3xl font-bold text-balance sm:text-4xl">
          How close can a loan get before it is liquidated?
        </h2>
        <p className="max-w-prose text-lg">
          Deposit collateral, borrow up to 80% of its value, and let the market move it. Past 100% of your borrowing
          power, the keeper seizes the collateral.
        </p>
        <button
          type="button"
          onClick={onOpen}
          disabled={!ready}
          className="mt-6 rounded-md bg-white px-5 py-3 font-semibold text-river transition-colors hover:bg-stone disabled:opacity-60"
        >
          {ready ? "Open an account" : "Connecting"}
        </button>
      </div>
      <Gauge scale={FULL_SCALE} />
    </section>
  );
}

function pricesOf(account: Account): Prices {
  return Object.fromEntries(account.prices.map(({ asset, price_usd }) => [asset, price_usd]));
}
