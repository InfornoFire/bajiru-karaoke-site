use crate::error::DbError;
use crate::models::performance_audio::{NewPerformanceAudio, PerformanceAudio};
use sqlx::MySqlPool;

type Result<T> = std::result::Result<T, DbError>;

pub async fn get_by_id(pool: &MySqlPool, id: i32) -> Result<Option<PerformanceAudio>> {
    sqlx::query_as::<_, PerformanceAudio>(
        "SELECT id, performance_id, public_url, internal_path \
         FROM performance_audios WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::from)
}

pub async fn list_for_performance(
    pool: &MySqlPool,
    performance_id: i32,
) -> Result<Vec<PerformanceAudio>> {
    sqlx::query_as::<_, PerformanceAudio>(
        "SELECT id, performance_id, public_url, internal_path \
         FROM performance_audios WHERE performance_id = ?",
    )
    .bind(performance_id)
    .fetch_all(pool)
    .await
    .map_err(DbError::from)
}

pub async fn create(pool: &MySqlPool, new: &NewPerformanceAudio) -> Result<PerformanceAudio> {
    let id = sqlx::query(
        "INSERT INTO performance_audios (performance_id, public_url, internal_path) \
         VALUES (?, ?, ?)",
    )
    .bind(new.performance_id)
    .bind(&new.public_url)
    .bind(&new.internal_path)
    .execute(pool)
    .await
    .map_err(DbError::from)?
    .last_insert_id();
    get_by_id(pool, id as i32).await?.ok_or(DbError::NotFound)
}

pub async fn delete(pool: &MySqlPool, id: i32) -> Result<bool> {
    sqlx::query("DELETE FROM performance_audios WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}
