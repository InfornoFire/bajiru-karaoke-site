use crate::error::DbError;
use crate::models::artist::Artist;
use crate::models::performance::{NewPerformance, Performance, UpdatePerformance};
use crate::models::song::Song;
use sqlx::MySqlPool;

type Result<T> = std::result::Result<T, DbError>;

pub async fn get_by_id(pool: &MySqlPool, id: u32) -> Result<Option<Performance>> {
    sqlx::query_as::<_, Performance>(
        "SELECT id, created_by, title, lyrics_id, play_count, duration, performance_date \
         FROM performances WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::from)
}

pub async fn list(pool: &MySqlPool) -> Result<Vec<Performance>> {
    sqlx::query_as::<_, Performance>(
        "SELECT id, created_by, title, lyrics_id, play_count, duration, performance_date \
         FROM performances ORDER BY performance_date DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(DbError::from)
}

pub async fn create(pool: &MySqlPool, new: &NewPerformance) -> Result<Performance> {
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
    .execute(pool)
    .await
    .map_err(DbError::from)?
    .last_insert_id();
    get_by_id(pool, id as u32).await?.ok_or(DbError::NotFound)
}

pub async fn update(
    pool: &MySqlPool,
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
    .execute(pool)
    .await
    .map_err(DbError::from)?
    .rows_affected();
    if affected == 0 {
        return Ok(None);
    }
    get_by_id(pool, id).await
}

pub async fn update_lyrics_id(pool: &MySqlPool, id: u32, lyrics_id: Option<u32>) -> Result<()> {
    sqlx::query("UPDATE performances SET lyrics_id = ? WHERE id = ?")
        .bind(lyrics_id)
        .bind(id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

pub async fn get_fallback_song_lyrics(
    pool: &MySqlPool,
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
    .fetch_optional(pool)
    .await
    .map_err(DbError::from)
}

pub async fn delete(pool: &MySqlPool, id: u32) -> Result<bool> {
    sqlx::query("DELETE FROM performances WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

pub async fn increment_play_count(pool: &MySqlPool, id: u32) -> Result<()> {
    sqlx::query("UPDATE performances SET play_count = play_count + 1 WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

pub async fn get_songs(pool: &MySqlPool, performance_id: u32) -> Result<Vec<Song>> {
    sqlx::query_as::<_, Song>(
        "SELECT s.id, s.title, s.created_by, s.lyrics_id, s.date_added \
         FROM songs s \
         JOIN performance_songs ps ON ps.song_id = s.id \
         WHERE ps.performance_id = ?",
    )
    .bind(performance_id)
    .fetch_all(pool)
    .await
    .map_err(DbError::from)
}

pub async fn set_songs(pool: &MySqlPool, performance_id: u32, song_ids: &[u32]) -> Result<()> {
    let mut tx = pool.begin().await.map_err(DbError::from)?;
    sqlx::query("DELETE FROM performance_songs WHERE performance_id = ?")
        .bind(performance_id)
        .execute(&mut *tx)
        .await
        .map_err(DbError::from)?;
    for &song_id in song_ids {
        sqlx::query("INSERT INTO performance_songs (performance_id, song_id) VALUES (?, ?)")
            .bind(performance_id)
            .bind(song_id)
            .execute(&mut *tx)
            .await
            .map_err(DbError::from)?;
    }
    tx.commit().await.map_err(DbError::from)
}

pub async fn get_singers(pool: &MySqlPool, performance_id: u32) -> Result<Vec<Artist>> {
    sqlx::query_as::<_, Artist>(
        "SELECT a.id, a.name, a.description \
         FROM artists a \
         JOIN performance_singers ps ON ps.artist_id = a.id \
         WHERE ps.performance_id = ?",
    )
    .bind(performance_id)
    .fetch_all(pool)
    .await
    .map_err(DbError::from)
}

pub async fn set_singers(pool: &MySqlPool, performance_id: u32, artist_ids: &[u32]) -> Result<()> {
    let mut tx = pool.begin().await.map_err(DbError::from)?;
    sqlx::query("DELETE FROM performance_singers WHERE performance_id = ?")
        .bind(performance_id)
        .execute(&mut *tx)
        .await
        .map_err(DbError::from)?;
    for &artist_id in artist_ids {
        sqlx::query("INSERT INTO performance_singers (performance_id, artist_id) VALUES (?, ?)")
            .bind(performance_id)
            .bind(artist_id)
            .execute(&mut *tx)
            .await
            .map_err(DbError::from)?;
    }
    tx.commit().await.map_err(DbError::from)
}
