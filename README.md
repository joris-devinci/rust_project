# How to start the program

## Start the database

Run the following commands to start up the DB
`docker compose up -d`

Then to enter in the DB do the following
`docker ps`
`docker exec -it {hash_from_the_container\name_from_the_container} bash`
`psql -U user -d db`


## Start the Application

Run `cargo run`

to quit just press ctrl-c


## Start the web interface

You need Docker, Rust, Node 24 or later, and pnpm (`npm install -g pnpm@10.28.1`).

Install the dependencies once:

```
cd web
pnpm install
```

Then, with Docker started, use two terminals.

Terminal 1, from the root of the repository (database, then Rust application):

```
docker compose up -d --wait
cargo run
```

Terminal 2 (web interface):

```
cd web
pnpm dev
```

Open http://localhost:3001. Prices can take a few seconds to appear.
The web server relays `/api` to the Rust API on port 3000, so both ports must be free.

Run the interface tests with `pnpm test`, from `web`.


## Run the demo

The prices are real and the funds are not. The goal is to see the keeper liquidate a position.

1. Press Open an account.
2. In Deposit, choose ETH, enter 1 and submit. The whole borrowing power is available: 80% of the collateral value.
3. Optional: in Borrow, choose USD and enter 100000. The protocol refuses it.
4. In Borrow, choose USD, press Max and submit. It borrows 99.9% of what is left, and the status becomes Warning.
5. Press Max and submit again to get even closer to the limit, so that a smaller drop of ETH is enough.
   Keep the page open. The keeper checks every 5 seconds.
6. When the keeper liquidates the position, the page shows the position before and after, and the Rust terminal prints `keeper: liquidated AccountId(...)`.

Before presenting:

- Borrow ahead of time. The wait depends on the market: it can be seconds, or more than 15 minutes.
- If ETH went up meanwhile, press Max and submit again to come back to the limit.
- Do not reload the page: the liquidation report and the chart are lost. The account is kept.
- Do not restart the Rust application: accounts live in memory.

If something is wrong:

- The interface cannot reach the API: check that `cargo run` is still running, and that nothing else uses port 3000.
- The Rust application stops at startup: check that PostgreSQL is running and that `.env` is at the root.
- Port 3001 is taken: run `pnpm exec next dev -p 3002` from `web`.


## Stop the database or delete it

Stop Database:
`docker compose down`

Stop and delete volume(content) of the database:
`docker compose down -v`