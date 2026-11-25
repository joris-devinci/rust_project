# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Next.js (App Router), TypeScript and Tailwind CSS, in `web/`. Runs locally on port 3001 and proxies to the Axum API on port 3000.

## Users

- A professor evaluating a student Rust project, watching a live demo given by one of its authors.
- Later, a recruiter looking at the same project as a portfolio piece.

## Product Purpose

Visualize an offchain simulation of a lending protocol written in Rust. A visitor opens an account, deposits collateral, borrows against it, repays, and watches real market prices move the position toward liquidation. Success: the distance to liquidation is understood at a glance, and a liquidation is never missed.

## Positioning

Real prices from three venues (Coinbase, Binance, CoinGecko), reduced to a median, drive a real keeper that liquidates unhealthy positions every five seconds. Nothing about a liquidation is staged.

## Operating Context

Local demo: PostgreSQL in Docker, the Rust server, and the web app on one machine. The demo scenario is to borrow close to the limit and let the market liquidate the position.

## Capabilities and Constraints

- Assets: ETH, BTC, SOL and USD. Collateral factor: 80%.
- Actions: open an account, deposit, borrow, repay. There is no withdrawal, interest, wallet, or account ownership.
- Health comes from the API: NoDebt, Safe (below 75% of borrowing power), Unsafe (75% and above), Warning (90% and above), Liquidatable (above 100%).
- A liquidation seizes all the collateral and cancels all the debt. The API does not announce it: the UI deduces it when the collateral disappears.
- Accounts live in memory and disappear when the server restarts.
- The API exposes current prices only. Any history is what this browser observed since the page opened.
- The UI must not change the Rust core.

## Evidence on Hand

Live data from the API. There are no customers, volumes, yields or reserves to show, and none may be invented.

## Product Principles

1. The distance to liquidation comes first.
2. Show only what the API knows, and say when a value is observed or stale.
3. Never suggest real funds, a wallet, or a blockchain.
4. Code a student can explain to a professor line by line.
5. It must not read as a generic crypto dashboard.

## Accessibility & Inclusion

Keyboard operable, AA contrast, usable at phone width. Health is never conveyed by color alone.
