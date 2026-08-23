static DB: std::sync::OnceLock<sqlx::PgPool> = std::sync::OnceLock::new();

type InitError = Box<dyn std::error::Error + Send + Sync>;

async fn create_pool() -> Result<sqlx::PgPool, InitError> {
    let database_url = std::env::var("DATABASE_URL").map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "DATABASE_URL is required; set it to the PostgreSQL connection URL used by the application ({error})"
            ),
        )
    })?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .connect(database_url.as_str())
        .await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}

pub async fn init_db() -> Result<(), InitError> {
    let pool = create_pool().await?;
    if DB.set(pool).is_err() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "database was already initialized",
        )
        .into());
    }
    Ok(())
}

/// # Panics
///
/// Panics if called before [`init_db`] has completed successfully.
pub fn get_db() -> &'static sqlx::PgPool {
    DB.get()
        .unwrap_or_else(|| panic!("database is not initialized"))
}
