//! Database access layer: connection pooling, models, and query functions.

pub mod error;
pub mod models;
pub mod queries;

pub use sqlx::MySqlPool;

pub type Result<T> = std::result::Result<T, error::DbError>;

/// Connects to MySQL and runs any pending migrations.
///
/// # Errors
///
/// Returns [`error::DbError`] if the connection cannot be established or a
/// migration fails.
pub async fn connect(database_url: &str) -> Result<MySqlPool> {
    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
