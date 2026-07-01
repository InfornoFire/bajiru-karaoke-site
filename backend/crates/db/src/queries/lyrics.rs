//! Query functions for the `lyrics` table.
//!
//! Lyrics rows are shared: multiple songs or performances can point to the same
//! `lyrics_id`.

use crate::error::DbError;
use crate::models::lyrics::{Lyrics, NewLyrics};
use sqlx::MySqlPool;

type Result<T> = std::result::Result<T, DbError>;

/// Fetches a lyrics row by ID.
pub async fn get_by_id(pool: &MySqlPool, id: u32) -> Result<Option<Lyrics>> {
    sqlx::query_as::<_, Lyrics>("SELECT id, content FROM lyrics WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(DbError::from)
}

/// Inserts a new lyrics row and returns it.
pub async fn create(pool: &MySqlPool, new: &NewLyrics) -> Result<Lyrics> {
    let id = sqlx::query("INSERT INTO lyrics (content) VALUES (?)")
        .bind(&new.content)
        .execute(pool)
        .await
        .map_err(DbError::from)?
        .last_insert_id();
    get_by_id(pool, id as u32).await?.ok_or(DbError::NotFound)
}

/// Updates the content of an existing lyrics row in place.
///
/// Because lyrics rows are shared, this change is visible to every song and
/// performance that references this ID.
pub async fn update(pool: &MySqlPool, id: u32, content: &str) -> Result<()> {
    sqlx::query("UPDATE lyrics SET content = ? WHERE id = ?")
        .bind(content)
        .bind(id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Deletes a lyrics row by ID. Returns `true` if a row was deleted.
pub async fn delete(pool: &MySqlPool, id: u32) -> Result<bool> {
    sqlx::query("DELETE FROM lyrics WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

/// Returns the total number of songs and performances that reference this lyrics record.
///
/// Check this before deleting: only delete the row when the count reaches zero.
pub async fn reference_count(pool: &MySqlPool, id: u32) -> Result<u64> {
    let songs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM songs WHERE lyrics_id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(DbError::from)?;
    let perfs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM performances WHERE lyrics_id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(DbError::from)?;
    Ok((songs + perfs) as u64)
}
