//! Query functions for the `performances` table and its related join tables.
//!
//! M2M relations (songs, singers) are managed via full replace helpers
//! (`set_songs`, `set_singers`) that delete existing rows and reinsert.

use std::collections::HashMap;

use sqlx::{Executor, MySql, MySqlConnection};

use crate::error::DbError;
use crate::models::artist::Artist;
use crate::models::performance::{NewPerformance, Performance, UpdatePerformance};
use crate::models::song::Song;

type Result<T> = std::result::Result<T, DbError>;

/// Fetches a performance by ID.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: u32,
) -> Result<Option<Performance>> {
    sqlx::query_as::<_, Performance>(
        "SELECT id, created_by, title, lyrics_id, play_count, duration, performance_date \
         FROM performances WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Returns all performances ordered by performance date descending.
pub async fn list(executor: impl Executor<'_, Database = MySql>) -> Result<Vec<Performance>> {
    sqlx::query_as::<_, Performance>(
        "SELECT id, created_by, title, lyrics_id, play_count, duration, performance_date \
         FROM performances ORDER BY performance_date DESC",
    )
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Inserts a new performance and returns the created row.
pub async fn create(conn: &mut MySqlConnection, new: &NewPerformance) -> Result<Performance> {
    let id = sqlx::query(
        "INSERT INTO performances \
         (created_by, title, lyrics_id, duration, performance_date) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(new.created_by)
    .bind(&new.title)
    .bind(new.lyrics_id)
    .bind(new.duration)
    .bind(new.performance_date)
    .execute(&mut *conn)
    .await
    .map_err(DbError::from)?
    .last_insert_id();
    get_by_id(&mut *conn, id as u32)
        .await?
        .ok_or(DbError::NotFound)
}

/// Updates a performance's mutable scalar fields. Returns `None` if the ID does not exist.
pub async fn update(
    conn: &mut MySqlConnection,
    id: u32,
    upd: &UpdatePerformance,
) -> Result<Option<Performance>> {
    let affected = sqlx::query(
        "UPDATE performances \
         SET title = ?, duration = ?, performance_date = ? \
         WHERE id = ?",
    )
    .bind(&upd.title)
    .bind(upd.duration)
    .bind(upd.performance_date)
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

/// Sets the `lyrics_id` foreign key on a performance, or clears it with `None`.
pub async fn update_lyrics_id(
    executor: impl Executor<'_, Database = MySql>,
    id: u32,
    lyrics_id: Option<u32>,
) -> Result<()> {
    sqlx::query("UPDATE performances SET lyrics_id = ? WHERE id = ?")
        .bind(lyrics_id)
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Returns the lyrics content from the first linked song that has lyrics.
///
/// Used as a fallback when the performance itself has no `lyrics_id` set.
/// Songs are ordered by ID to give a deterministic result when multiple are linked.
pub async fn get_fallback_song_lyrics(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: u32,
) -> Result<Option<String>> {
    sqlx::query_scalar::<_, String>(
        "SELECT l.content \
         FROM lyrics l \
         JOIN songs s ON s.lyrics_id = l.id \
         JOIN performance_songs ps ON ps.song_id = s.id \
         WHERE ps.performance_id = ? \
         ORDER BY ps.song_id \
         LIMIT 1",
    )
    .bind(performance_id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Deletes a performance by ID. Returns `true` if a row was deleted.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, id: u32) -> Result<bool> {
    sqlx::query("DELETE FROM performances WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

/// Atomically increments the play count for a performance.
pub async fn increment_play_count(
    executor: impl Executor<'_, Database = MySql>,
    id: u32,
) -> Result<()> {
    sqlx::query("UPDATE performances SET play_count = play_count + 1 WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Returns the songs linked to a performance via the `performance_songs` join table.
pub async fn get_songs(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: u32,
) -> Result<Vec<Song>> {
    sqlx::query_as::<_, Song>(
        "SELECT s.id, s.title, s.created_by, s.lyrics_id, s.date_added \
         FROM songs s \
         JOIN performance_songs ps ON ps.song_id = s.id \
         WHERE ps.performance_id = ?",
    )
    .bind(performance_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Replaces the full set of songs for a performance.
///
/// Must be called within a caller provided transaction for atomicity.
pub async fn set_songs(
    conn: &mut MySqlConnection,
    performance_id: u32,
    song_ids: &[u32],
) -> Result<()> {
    sqlx::query("DELETE FROM performance_songs WHERE performance_id = ?")
        .bind(performance_id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    for &song_id in song_ids {
        sqlx::query("INSERT INTO performance_songs (performance_id, song_id) VALUES (?, ?)")
            .bind(performance_id)
            .bind(song_id)
            .execute(&mut *conn)
            .await
            .map_err(DbError::from)?;
    }
    Ok(())
}

/// Returns the singers for a performance via the `performance_singers` join table.
pub async fn get_singers(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: u32,
) -> Result<Vec<Artist>> {
    sqlx::query_as::<_, Artist>(
        "SELECT a.id, a.name, a.description \
         FROM artists a \
         JOIN performance_singers ps ON ps.artist_id = a.id \
         WHERE ps.performance_id = ?",
    )
    .bind(performance_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Returns singers for multiple performances, keyed by performance ID.
///
/// Performances with no singers are absent from the returned map.
pub async fn get_singers_batch(
    executor: impl Executor<'_, Database = MySql>,
    performance_ids: &[u32],
) -> Result<HashMap<u32, Vec<Artist>>> {
    if performance_ids.is_empty() {
        return Ok(HashMap::new());
    }

    #[derive(sqlx::FromRow)]
    struct Row {
        performance_id: u32,
        id: u32,
        name: String,
        description: Option<String>,
    }

    let mut builder = sqlx::QueryBuilder::new(
        "SELECT ps.performance_id, a.id, a.name, a.description \
         FROM artists a \
         JOIN performance_singers ps ON ps.artist_id = a.id \
         WHERE ps.performance_id IN (",
    );
    let mut separated = builder.separated(", ");
    for performance_id in performance_ids {
        separated.push_bind(performance_id);
    }
    builder.push(")");

    let rows: Vec<Row> = builder
        .build_query_as()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)?;

    let mut by_performance: HashMap<u32, Vec<Artist>> = HashMap::new();
    for row in rows {
        by_performance
            .entry(row.performance_id)
            .or_default()
            .push(Artist {
                id: row.id,
                name: row.name,
                description: row.description,
            });
    }
    Ok(by_performance)
}

/// Replaces the full set of singers for a performance.
///
/// Must be called within a caller provided transaction for atomicity.
pub async fn set_singers(
    conn: &mut MySqlConnection,
    performance_id: u32,
    artist_ids: &[u32],
) -> Result<()> {
    sqlx::query("DELETE FROM performance_singers WHERE performance_id = ?")
        .bind(performance_id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    for &artist_id in artist_ids {
        sqlx::query("INSERT INTO performance_singers (performance_id, artist_id) VALUES (?, ?)")
            .bind(performance_id)
            .bind(artist_id)
            .execute(&mut *conn)
            .await
            .map_err(DbError::from)?;
    }
    Ok(())
}
