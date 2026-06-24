use crate::error::DbError;
use crate::models::tag::{NewTag, Tag};
use sqlx::MySqlPool;

type Result<T> = std::result::Result<T, DbError>;

pub async fn get_by_id(pool: &MySqlPool, id: i32) -> Result<Option<Tag>> {
    sqlx::query_as::<_, Tag>("SELECT id, name, kind FROM tags WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(DbError::from)
}

pub async fn list(pool: &MySqlPool) -> Result<Vec<Tag>> {
    sqlx::query_as::<_, Tag>("SELECT id, name, kind FROM tags ORDER BY name")
        .fetch_all(pool)
        .await
        .map_err(DbError::from)
}

pub async fn list_by_kind(pool: &MySqlPool, kind: &str) -> Result<Vec<Tag>> {
    sqlx::query_as::<_, Tag>("SELECT id, name, kind FROM tags WHERE kind = ? ORDER BY name")
        .bind(kind)
        .fetch_all(pool)
        .await
        .map_err(DbError::from)
}

pub async fn get_or_create(pool: &MySqlPool, new: &NewTag) -> Result<Tag> {
    sqlx::query("INSERT IGNORE INTO tags (name, kind) VALUES (?, ?)")
        .bind(&new.name)
        .bind(&new.kind)
        .execute(pool)
        .await
        .map_err(DbError::from)?;
    sqlx::query_as::<_, Tag>("SELECT id, name, kind FROM tags WHERE name = ?")
        .bind(&new.name)
        .fetch_one(pool)
        .await
        .map_err(DbError::from)
}

pub async fn delete(pool: &MySqlPool, id: i32) -> Result<bool> {
    sqlx::query("DELETE FROM tags WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}
