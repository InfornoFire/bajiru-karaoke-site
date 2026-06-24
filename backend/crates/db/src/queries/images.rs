use crate::error::DbError;
use crate::models::image::{Image, NewImage, UpdateImage};
use sqlx::MySqlPool;

type Result<T> = std::result::Result<T, DbError>;

pub async fn get_by_id(pool: &MySqlPool, id: i32) -> Result<Option<Image>> {
    sqlx::query_as::<_, Image>(
        "SELECT id, public_url, internal_path, credits FROM images WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::from)
}

pub async fn create(pool: &MySqlPool, new: &NewImage) -> Result<Image> {
    let id =
        sqlx::query("INSERT INTO images (public_url, internal_path, credits) VALUES (?, ?, ?)")
            .bind(&new.public_url)
            .bind(&new.internal_path)
            .bind(&new.credits)
            .execute(pool)
            .await
            .map_err(DbError::from)?
            .last_insert_id();
    get_by_id(pool, id as i32).await?.ok_or(DbError::NotFound)
}

pub async fn update(pool: &MySqlPool, id: i32, upd: &UpdateImage) -> Result<Option<Image>> {
    let affected = sqlx::query(
        "UPDATE images SET public_url = ?, internal_path = ?, credits = ? WHERE id = ?",
    )
    .bind(&upd.public_url)
    .bind(&upd.internal_path)
    .bind(&upd.credits)
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
    sqlx::query("DELETE FROM images WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}
