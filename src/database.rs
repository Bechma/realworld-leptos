#[cfg(feature = "ssr")]
static DB: once_cell::sync::OnceCell<sqlx::PgPool> = once_cell::sync::OnceCell::new();

#[cfg(feature = "ssr")]
async fn create_pool() -> sqlx::PgPool {
    use sqlx::Executor;

    let database_url = &std::env::var("DATABASE_URL").expect("no database url specify");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await
        .expect("could not connect to database_url");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("migrations failed");

    pool.execute("CREATE EXTENSION IF NOT EXISTS pgcrypto")
        .await
        .expect("pgcrypto not available");
    pool
}

#[cfg(feature = "ssr")]
pub async fn init_db() -> Result<(), sqlx::Pool<sqlx::Postgres>> {
    DB.set(create_pool().await)
}

#[cfg(feature = "ssr")]
pub fn get_db<'a>() -> &'a sqlx::PgPool {
    DB.get().expect("database unitialized")
}
