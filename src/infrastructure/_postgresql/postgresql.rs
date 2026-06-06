use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

// CHANGED: The return type is now a Result, which matches Ok(pool) and allows the ? operator.
pub async fn connect_database_url(postgresql_url: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(postgresql_url)
        .await?; // CHANGED: Replaced .expect("msg") with ? to bubble up connection errors

    // Verify connection by running a simple query
    let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await?;

    println!("Successfully connected to Neon! Row result: {}", row.0);

    // Return the pool wrapped in Ok()
    Ok(pool)
}
