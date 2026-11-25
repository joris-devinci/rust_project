"use client";

import { type FormEvent, useState } from "react";

import type { Account, Asset, Movement } from "@/lib/api";
import { explain } from "@/lib/explain";
import { ASSETS, formatUsd, SYMBOLS } from "@/lib/format";
import { maxBorrow, valueUsd } from "@/lib/position";

const LABELS: Record<Movement, string> = { deposit: "Deposit", borrow: "Borrow", repay: "Repay" };
const MOVEMENTS: Movement[] = ["deposit", "borrow", "repay"];
const AMOUNT = /^\d+(\.\d+)?$/;

type Props = {
  account: Account;
  refresh: () => Promise<Account | null>;
  move: (movement: Movement, asset: Asset, amount: string) => Promise<void>;
};

export function ActionForm(props: Props) {
  const [movement, setMovement] = useState<Movement>("deposit");
  return (
    <section aria-label="Actions" className="rounded-lg bg-panel p-5 sm:p-6">
      <div role="group" aria-label="Action" className="mb-5 grid grid-cols-3 gap-1 rounded-md bg-stone p-1">
        {MOVEMENTS.map((option) => (
          <button
            key={option}
            type="button"
            aria-pressed={option === movement}
            onClick={() => setMovement(option)}
            className="rounded px-3 py-2 font-semibold transition-colors hover:bg-stone-dark aria-pressed:bg-river aria-pressed:text-white"
          >
            {LABELS[option]}
          </button>
        ))}
      </div>
      <MovementForm key={movement} movement={movement} {...props} />
    </section>
  );
}

function MovementForm({ movement, account, refresh, move }: Props & { movement: Movement }) {
  const debts = ASSETS.filter((asset) => account.position.debt[asset]);
  const choices = movement === "repay" ? debts : ASSETS;
  const [chosen, setChosen] = useState<Asset>(movement === "borrow" ? "Usd" : "Eth");
  const [amount, setAmount] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // A repaid or liquidated debt leaves the list
  const asset = choices.includes(chosen) ? chosen : choices[0];
  if (asset === undefined) return <p className="text-ink-soft">There is no debt to repay.</p>;

  const quantity = amount.trim().replace(",", ".");
  const isValid = AMOUNT.test(quantity) && /[1-9]/.test(quantity);
  const value = isValid ? valueUsd(account, asset, quantity) : null;

  async function fillMax() {
    if (movement === "repay") return setAmount(account.position.debt[asset] ?? "");
    // The price may have moved since the last refresh
    const fresh = await refresh();
    if (fresh) setAmount(maxBorrow(fresh, asset));
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (!isValid) return setError("Enter an amount above zero.");
    setPending(true);
    setError(null);
    try {
      await move(movement, asset, quantity);
      setAmount("");
    } catch (failure) {
      setError(explain(failure, account, asset));
    } finally {
      setPending(false);
    }
  }

  const field = "w-full rounded-md border border-ink-soft bg-white px-3 py-2.5 placeholder:text-ink-soft";
  return (
    <form onSubmit={submit} noValidate>
      <label className="mb-4 block">
        <span className="mb-1 block font-semibold">Asset</span>
        <select className={`select ${field}`} value={asset} onChange={(event) => setChosen(event.target.value as Asset)}>
          {choices.map((choice) => (
            <option key={choice} value={choice}>
              {SYMBOLS[choice]}
            </option>
          ))}
        </select>
      </label>
      <label htmlFor="amount" className="mb-1 block font-semibold">
        Amount
      </label>
      <div className="flex gap-2">
        <input
          id="amount"
          className={field}
          inputMode="decimal"
          autoComplete="off"
          placeholder="0.0"
          value={amount}
          aria-invalid={error !== null}
          aria-describedby="amount-note"
          onChange={(event) => setAmount(event.target.value)}
        />
        {movement !== "deposit" && (
          <button
            type="button"
            onClick={fillMax}
            className="rounded-md bg-stone px-4 font-semibold transition-colors hover:bg-stone-dark"
          >
            Max
          </button>
        )}
      </div>
      <p id="amount-note" className="mt-1.5 mb-4 min-h-6 text-ink-soft">
        {value && `About ${formatUsd(value)}`}
      </p>
      {error && (
        <p role="alert" className="mb-4 rounded-md bg-amber-pale px-3 py-2.5">
          {error}
        </p>
      )}
      <button
        type="submit"
        disabled={pending}
        className="w-full rounded-md bg-river px-4 py-3 font-semibold text-white transition-colors hover:bg-river-deep disabled:opacity-60"
      >
        {pending ? "Sending" : `${LABELS[movement]} ${SYMBOLS[asset]}`}
      </button>
    </form>
  );
}
