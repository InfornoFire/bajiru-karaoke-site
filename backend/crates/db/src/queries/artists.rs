use crate::error::DbError;
use crate::models::artist::{Artist, NewArtist, UpdateArtist};
use sqlx::MySqlPool;

type Result<T> = std::result::Result<T, DbError>;

pub async fn get_by_id(pool: &MySqlPool, id: i32) -> Result<Option<Artist>> {
    sqlx::query_as::<_, Artist>("SELECT id, name, description FROM artists WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(DbError::from)
}

pub async fn list(pool: &MySqlPool) -> Result<Vec<Artist>> {
    sqlx::query_as::<_, Artist>("SELECT id, name, description FROM artists ORDER BY name")
        .fetch_all(pool)
        .await
        .map_err(DbError::from)
}

pub async fn create(pool: &MySqlPool, new: &NewArtist) -> Result<Artist> {
    let id = sqlx::query("INSERT INTO artists (name, description) VALUES (?, ?)")
        .bind(&new.name)
        .bind(&new.description)
        .execute(pool)
        .await
        .map_err(DbError::from)?
        .last_insert_id();
    get_by_id(pool, id as i32).await?.ok_or(DbError::NotFound)
}

pub async fn update(pool: &MySqlPool, id: i32, upd: &UpdateArtist) -> Result<Option<Artist>> {
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

pub async fn delete(pool: &MySqlPool, id: i32) -> Result<bool> {
    sqlx::query("DELETE FROM artists WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}
