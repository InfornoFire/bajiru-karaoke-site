//! Query functions for the `performance_audios` table.

use sqlx::{Executor, MySql, MySqlConnection};

use crate::error::DbError;
use crate::models::performance_audio::{NewPerformanceAudio, PerformanceAudio};

type Result<T> = std::result::Result<T, DbError>;

/// Fetches a performance audio record by ID.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: u32,
) -> Result<Option<PerformanceAudio>> {
    sqlx::query_as::<_, PerformanceAudio>(
        "SELECT id, performance_id, public_url, internal_path \
         FROM performance_audios WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Returns all audio records for a given performance.
pub async fn list_for_performance(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: u32,
) -> Result<Vec<PerformanceAudio>> {
    sqlx::query_as::<_, PerformanceAudio>(
        "SELECT id, performance_id, public_url, internal_path \
         FROM performance_audios WHERE performance_id = ?",
    )
    .bind(performance_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Inserts a new performance audio record and returns it.
pub async fn create(
    conn: &mut MySqlConnection,
    new: &NewPerformanceAudio,
) -> Result<PerformanceAudio> {
    let id = sqlx::query(
        "INSERT INTO performance_audios (performance_id, public_url, internal_path) \
         VALUES (?, ?, ?)",
    )
    .bind(new.performance_id)
    .bind(&new.public_url)
    .bind(&new.internal_path)
    .execute(&mut *conn)
    .await
    .map_err(DbError::from)?
    .last_insert_id();
    get_by_id(&mut *conn, id as u32)
        .await?
        .ok_or(DbError::NotFound)
}

/// Deletes a performance audio record by ID. Returns `true` if a row was deleted.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, id: u32) -> Result<bool> {
    sqlx::query("DELETE FROM performance_audios WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}
