//! Query functions for the `artists` table.

use crate::error::DbError;
use crate::models::artist::{Artist, NewArtist, UpdateArtist};
use sqlx::MySqlPool;

type Result<T> = std::result::Result<T, DbError>;

/// Fetches an artist by ID.
pub async fn get_by_id(pool: &MySqlPool, id: u32) -> Result<Option<Artist>> {
    sqlx::query_as::<_, Artist>("SELECT id, name, description FROM artists WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(DbError::from)
}

/// Returns all artists ordered by name.
pub async fn list(pool: &MySqlPool) -> Result<Vec<Artist>> {
    sqlx::query_as::<_, Artist>("SELECT id, name, description FROM artists ORDER BY name")
        .fetch_all(pool)
        .await
        .map_err(DbError::from)
}

/// Inserts a new artist and returns the created row.
pub async fn create(pool: &MySqlPool, new: &NewArtist) -> Result<Artist> {
    let id = sqlx::query("INSERT INTO artists (name, description) VALUES (?, ?)")
        .bind(&new.name)
        .bind(&new.description)
        .execute(pool)
        .await
        .map_err(DbError::from)?
        .last_insert_id();
    get_by_id(pool, id as u32).await?.ok_or(DbError::NotFound)
}

/// Updates an artist's mutable fields. Returns `None` if the ID does not exist.
pub async fn update(pool: &MySqlPool, id: u32, upd: &UpdateArtist) -> Result<Option<Artist>> {
    let affected = sqlx::query("UPDATE artists SET name = ?, description = ? WHERE id = ?")
        .bind(&upd.name)
        .bind(&upd.description)
        .bind(id)
        .execute(pool)
        .await
        .map_err(DbError::from)?
        .rows_affected();
    if affected == 0 {
        return Ok(None);
    }
    get_by_id(pool, id).await
}

/// Deletes an artist by ID. Returns `true` if a row was deleted.
pub async fn delete(pool: &MySqlPool, id: u32) -> Result<bool> {
    sqlx::query("DELETE FROM artists WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}
