export type Asset = "Eth" | "Btc" | "Solana" | "Usd";

export type Health = "NoDebt" | "Safe" | "Unsafe" | "Warning" | "Liquidatable";

export type Movement = "deposit" | "borrow" | "repay";

/** Decimal quantities stay strings: a JS number would round them. */
export type Balances = Partial<Record<Asset, string>>;

/** An asset is absent when the oracle has no recent price for it. */
export type Prices = Balances;

export type Account = {
  id: number;
  health: Health;
  borrow_utilization_percent: string | null;
  position: { collateral: Balances; debt: Balances };
  collateral: { total_value_usd: string; borrowing_power_usd: string };
  prices: { asset: Asset; price_usd: string }[];
};

export class ApiError extends Error {
  readonly status: number;

  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

/** The API answers 503 when it cannot price the account. */
const NO_PRICE = "The oracle has no usable price right now. Retrying every two seconds.";

async function request(path: string, init?: RequestInit): Promise<Response> {
  const response = await fetch(`/api${path}`, init).catch(() => {
    throw new ApiError(0, "The web server is unreachable.");
  });
  if (response.ok) return response;

  // Our API answers {"error": "..."}; Axum and the Next proxy answer plain text
  const text = await response.text();
  const message = response.status === 503 ? NO_PRICE : readError(text);
  throw new ApiError(response.status, message ?? plainError(response.status));
}

function readError(text: string): string | null {
  try {
    const body: { error?: unknown } = JSON.parse(text);
    return typeof body.error === "string" ? body.error : null;
  } catch {
    return null;
  }
}

function plainError(status: number): string {
  return status >= 500
    ? "The Rust API is unreachable. Is it running on port 3000?"
    : "The API rejected this request.";
}

function post(body?: unknown): RequestInit {
  if (body === undefined) return { method: "POST" };
  return {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  };
}

export async function openAccount(): Promise<number> {
  const opened: { id: number } = await (await request("/accounts", post())).json();
  return opened.id;
}

export async function getAccount(id: number): Promise<Account> {
  return (await request(`/accounts/${id}`)).json();
}

export async function getPrices(): Promise<Prices> {
  const quotes: { asset: Asset; price: string | null }[] = await (await request("/prices")).json();
  const prices: Prices = {};
  for (const { asset, price } of quotes) {
    if (price !== null) prices[asset] = price;
  }
  return prices;
}

export async function move(id: number, movement: Movement, asset: Asset, amount: string) {
  await request(`/accounts/${id}/${movement}`, post({ asset, amount }));
}
