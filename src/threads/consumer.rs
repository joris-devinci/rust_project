pub async fn consumer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv()?;
    let queue = crate::threads::queue::QUEUE.clone();

    println!("Consumer making connections with DB");
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    println!("Connection with DB established");

    loop {
        let ticker = {
            let mut queue = queue.lock().unwrap();
            queue.pop()
        };

        let Some(ticker) = ticker else {
            println!("There is no Ticker inside the option");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        };

        println!("Queue len = {}", queue.lock().unwrap().len());

        sqlx::query(
            "INSERT INTO ticker (asset, venue, price, date)
            VALUES ($1, $2, $3::numeric, $4)"
        )
        .bind(ticker.asset.to_string())
        .bind(ticker.venue.to_string())
        .bind(ticker.price.0)
        .bind(ticker.date)
        .execute(&pool)
        .await?;

        // tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}