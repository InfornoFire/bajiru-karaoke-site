pub mod error;
pub mod models;
pub mod queries;

pub use sqlx::MySqlPool;

pub type Result<T> = std::result::Result<T, error::DbError>;

pub async fn connect(database_url: &str) -> Result<MySqlPool> {
    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
