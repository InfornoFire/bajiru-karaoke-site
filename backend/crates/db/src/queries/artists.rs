//! Query functions for the `artists` table.

use sqlx::{Executor, MySql, MySqlConnection};

use crate::error::DbError;
use crate::models::artist::{Artist, NewArtist, UpdateArtist};

type Result<T> = std::result::Result<T, DbError>;

/// Fetches an artist by ID.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: u32,
) -> Result<Option<Artist>> {
    sqlx::query_as::<_, Artist>("SELECT id, name, description FROM artists WHERE id = ?")
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(DbError::from)
}

/// Returns all artists ordered by name.
pub async fn list(executor: impl Executor<'_, Database = MySql>) -> Result<Vec<Artist>> {
    sqlx::query_as::<_, Artist>("SELECT id, name, description FROM artists ORDER BY name")
        .fetch_all(executor)
        .await
        .map_err(DbError::from)
}

/// Inserts a new artist and returns the created row.
pub async fn create(conn: &mut MySqlConnection, new: &NewArtist) -> Result<Artist> {
    let id = sqlx::query("INSERT INTO artists (name, description) VALUES (?, ?)")
        .bind(&new.name)
        .bind(&new.description)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?
        .last_insert_id();
    get_by_id(&mut *conn, id as u32)
        .await?
        .ok_or(DbError::NotFound)
}

/// Updates an artist's mutable fields. Returns `None` if the ID does not exist.
pub async fn update(
    conn: &mut MySqlConnection,
    id: u32,
    upd: &UpdateArtist,
) -> Result<Option<Artist>> {
    let affected = sqlx::query("UPDATE artists SET name = ?, description = ? WHERE id = ?")
        .bind(&upd.name)
        .bind(&upd.description)
        .bind(id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?
        .rows_affected();
    if affected == 0 {
        return Ok(None);
    }
    get_by_id(&mut *conn, id).await
}

/// Deletes an artist by ID. Returns `true` if a row was deleted.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, id: u32) -> Result<bool> {
    sqlx::query("DELETE FROM artists WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}
