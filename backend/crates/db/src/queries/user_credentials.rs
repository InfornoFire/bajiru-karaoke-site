use sqlx::MySqlPool;

use crate::{error::DbError, models::user_credential::UserCredential};

type Result<T> = std::result::Result<T, DbError>;

pub async fn create(pool: &MySqlPool, user_id: u32, password_hash: &str) -> Result<()> {
    sqlx::query("INSERT INTO user_credentials (user_id, password_hash) VALUES (?, ?)")
        .bind(user_id)
        .bind(password_hash)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

pub async fn get_by_user_id(pool: &MySqlPool, user_id: u32) -> Result<Option<UserCredential>> {
    sqlx::query_as::<_, UserCredential>(
        "SELECT user_id, password_hash FROM user_credentials WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::from)
}
