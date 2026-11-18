mod threads;

use rust_project::{
    api::{AppState, router},
    domain::Bps,
    models,
    protocol::Protocol,
};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

fn main() {
    dotenvy::dotenv().expect("could not load .env");
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in the environment");
    let runtime = tokio::runtime::Runtime::new().unwrap();

    runtime.block_on(async {
        // API
        let threshold = Bps::new(8_000).expect("80% is a valid ratio");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("could not connect API to PostgreSQL");
        let state = AppState::with_database(Protocol::new(threshold), pool);
        tokio::spawn(rust_project::api::run_keeper(
            state.clone(),
            std::time::Duration::from_secs(5),
        ));

        let listener = TcpListener::bind("127.0.0.1:3000")
            .await
            .expect("could not bind API address");

        tokio::spawn(async move {
            if let Err(error) = axum::serve(listener, router(state)).await {
                eprintln!("API server failed: {error}");
            }
        });
        println!("API listening on http://127.0.0.1:3000");

        // Producer & Consumer
        let mut handles = Vec::new();
        let producer_count = 3;

        for tid in 0..producer_count {
            handles.push(tokio::spawn(threads::producer::producer(
                tid,
                producer_count,
            )));
        }

        handles.push(tokio::spawn(threads::consumer::consumer()));

        for handle in handles {
            if let Err(error) = handle.await.unwrap() {
                eprintln!("Request failed: {error}");
            }
        }
    });
}
