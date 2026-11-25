import type { Account, Prices } from "./api.ts";
import { wasLiquidated } from "./position.ts";

/** Time between two observations. */
export const PERIOD_MS = 2000;

/** Half an hour of history. */
export const MAX_POINTS = 900;

export type Point = { time: number; percent: number };

/** What this browser has observed since the page opened. */
export type Session = {
  account: Account | null;
  /** Oracle prices, shown before any account is open. */
  prices: Prices | null;
  points: Point[];
  /** Last account and levels observed before the collateral disappeared. */
  liquidation: { before: Account; points: Point[]; time: number } | null;
  /** Why the last refresh failed; the values on screen are then stale. */
  error: string | null;
  /** The server no longer knows the account: it restarted. */
  lost: boolean;
};

export type Action =
  | { type: "observed"; account: Account; time: number }
  | { type: "priced"; prices: Prices }
  | { type: "failed"; message: string }
  | { type: "lost" }
  | { type: "dismissed" };

export const initialSession: Session = {
  account: null,
  prices: null,
  points: [],
  liquidation: null,
  error: null,
  lost: false,
};

export function reduce(session: Session, action: Action): Session {
  switch (action.type) {
    case "observed": {
      const before = session.account;
      const { account, time } = action;
      const history = before?.id === account.id ? session.points : [];
      const point = { time, percent: Number(account.borrow_utilization_percent ?? 0) };
      return {
        ...session,
        account,
        points: [...history, point].slice(-MAX_POINTS),
        liquidation:
          before && wasLiquidated(before, account)
            ? { before, points: history, time }
            : session.liquidation,
        error: null,
        lost: false,
      };
    }
    case "priced":
      return { ...session, prices: action.prices, error: null };
    case "failed":
      return { ...session, error: action.message };
    case "lost":
      return { ...initialSession, prices: session.prices, liquidation: session.liquidation, lost: true };
    case "dismissed":
      return { ...session, liquidation: null };
  }
}
