"use client";

import { useCallback, useEffect, useReducer, useRef } from "react";

import * as api from "./api.ts";
import { initialSession, PERIOD_MS, reduce } from "./session.ts";

const STORAGE_KEY = "lending-account";

/** Follows one account: polls it, and keeps what was observed since the page opened. */
export function useAccount() {
  const [session, dispatch] = useReducer(reduce, initialSession);
  const accountId = useRef<number | null>(null);
  const lastRequest = useRef(0);

  /** Returns the fresh account, or null if there is none or it could not be read. */
  const refresh = useCallback(async (): Promise<api.Account | null> => {
    // Only the latest request counts: an older answer could look like a liquidation
    const request = ++lastRequest.current;
    const isStale = () => request !== lastRequest.current;
    const id = accountId.current;
    try {
      if (id === null) {
        const prices = await api.getPrices();
        if (!isStale()) dispatch({ type: "priced", prices });
        return null;
      }
      const account = await api.getAccount(id);
      if (isStale()) return null;
      dispatch({ type: "observed", account, time: Date.now() });
      return account;
    } catch (error) {
      if (isStale()) return null;
      if (error instanceof api.ApiError && error.status === 404) {
        accountId.current = null;
        remember(null);
        dispatch({ type: "lost" });
      } else {
        dispatch({ type: "failed", message: messageOf(error) });
      }
      return null;
    }
  }, []);

  useEffect(() => {
    accountId.current = recall();
    let timer: ReturnType<typeof setTimeout>;
    let stopped = false;
    async function poll() {
      await refresh();
      if (!stopped) timer = setTimeout(poll, PERIOD_MS);
    }
    poll();
    return () => {
      stopped = true;
      clearTimeout(timer);
    };
  }, [refresh]);

  const open = useCallback(async () => {
    const id = await api.openAccount();
    accountId.current = id;
    remember(id);
    await refresh();
  }, [refresh]);

  const move = useCallback(
    async (movement: api.Movement, asset: api.Asset, amount: string) => {
      const id = accountId.current;
      if (id === null) return;
      await api.move(id, movement, asset, amount);
      await refresh();
    },
    [refresh],
  );

  const dismiss = useCallback(() => dispatch({ type: "dismissed" }), []);

  return { session, refresh, open, move, dismiss };
}

export function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : "Something went wrong.";
}

function recall(): number | null {
  try {
    const id = Number.parseInt(localStorage.getItem(STORAGE_KEY) ?? "", 10);
    return Number.isNaN(id) ? null : id;
  } catch {
    return null; // Storage is blocked: the account lasts as long as the page
  }
}

function remember(id: number | null) {
  try {
    if (id === null) localStorage.removeItem(STORAGE_KEY);
    else localStorage.setItem(STORAGE_KEY, String(id));
  } catch {
    // Same as above
  }
}
