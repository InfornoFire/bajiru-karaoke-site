use crate::error::DbError;
use crate::models::lyrics::{Lyrics, NewLyrics};
use sqlx::MySqlPool;

type Result<T> = std::result::Result<T, DbError>;

pub async fn get_by_id(pool: &MySqlPool, id: i32) -> Result<Option<Lyrics>> {
    sqlx::query_as::<_, Lyrics>("SELECT id, content FROM lyrics WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(DbError::from)
}

pub async fn create(pool: &MySqlPool, new: &NewLyrics) -> Result<Lyrics> {
    let id = sqlx::query("INSERT INTO lyrics (content) VALUES (?)")
        .bind(&new.content)
        .execute(pool)
        .await
        .map_err(DbError::from)?
        .last_insert_id();
    get_by_id(pool, id as i32).await?.ok_or(DbError::NotFound)
}

pub async fn delete(pool: &MySqlPool, id: i32) -> Result<bool> {
    sqlx::query("DELETE FROM lyrics WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}
