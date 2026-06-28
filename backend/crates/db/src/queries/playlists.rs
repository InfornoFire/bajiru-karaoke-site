use crate::error::DbError;
use crate::models::playlist::{NewPlaylist, Playlist, UpdatePlaylist};
use sqlx::MySqlPool;

type Result<T> = std::result::Result<T, DbError>;

pub async fn get_by_id(pool: &MySqlPool, id: u32) -> Result<Option<Playlist>> {
    sqlx::query_as::<_, Playlist>(
        "SELECT id, title, description, kind, created_by FROM playlists WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::from)
}

pub async fn list(pool: &MySqlPool) -> Result<Vec<Playlist>> {
    sqlx::query_as::<_, Playlist>(
        "SELECT id, title, description, kind, created_by FROM playlists ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(DbError::from)
}

pub async fn list_by_user(pool: &MySqlPool, user_id: u32) -> Result<Vec<Playlist>> {
    sqlx::query_as::<_, Playlist>(
        "SELECT id, title, description, kind, created_by FROM playlists \
         WHERE created_by = ? ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(DbError::from)
}

pub async fn create(pool: &MySqlPool, new: &NewPlaylist) -> Result<Playlist> {
    let id = sqlx::query(
        "INSERT INTO playlists (title, description, kind, created_by) VALUES (?, ?, ?, ?)",
    )
    .bind(&new.title)
    .bind(&new.description)
    .bind(&new.kind)
    .bind(new.created_by)
    .execute(pool)
    .await
    .map_err(DbError::from)?
    .last_insert_id();
    get_by_id(pool, id as u32).await?.ok_or(DbError::NotFound)
}

pub async fn update(pool: &MySqlPool, id: u32, upd: &UpdatePlaylist) -> Result<Option<Playlist>> {
    let affected =
        sqlx::query("UPDATE playlists SET title = ?, description = ?, kind = ? WHERE id = ?")
            .bind(&upd.title)
            .bind(&upd.description)
            .bind(&upd.kind)
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

pub async fn delete(pool: &MySqlPool, id: u32) -> Result<bool> {
    sqlx::query("DELETE FROM playlists WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

pub async fn get_performance_ids(pool: &MySqlPool, playlist_id: u32) -> Result<Vec<u32>> {
    sqlx::query_scalar::<_, u32>(
        "SELECT performance_id FROM playlist_performances \
         WHERE playlist_id = ? ORDER BY sort_order",
    )
    .bind(playlist_id)
    .fetch_all(pool)
    .await
    .map_err(DbError::from)
}

pub async fn set_performances(
    pool: &MySqlPool,
    playlist_id: u32,
    performance_ids: &[u32],
) -> Result<()> {
    let mut tx = pool.begin().await.map_err(DbError::from)?;
    sqlx::query("DELETE FROM playlist_performances WHERE playlist_id = ?")
        .bind(playlist_id)
        .execute(&mut *tx)
        .await
        .map_err(DbError::from)?;
    for (pos, &performance_id) in performance_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO playlist_performances (playlist_id, performance_id, sort_order) \
             VALUES (?, ?, ?)",
        )
        .bind(playlist_id)
        .bind(performance_id)
        .bind(pos as u32)
        .execute(&mut *tx)
        .await
        .map_err(DbError::from)?;
    }
    tx.commit().await.map_err(DbError::from)
}

pub async fn add_performance(
    pool: &MySqlPool,
    playlist_id: u32,
    performance_id: u32,
) -> Result<()> {
    sqlx::query(
        "INSERT IGNORE INTO playlist_performances (playlist_id, performance_id, sort_order) \
         VALUES (?, ?, COALESCE(\
             (SELECT MAX(sort_order) + 1 FROM playlist_performances WHERE playlist_id = ?), 0))",
    )
    .bind(playlist_id)
    .bind(performance_id)
    .bind(playlist_id)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(DbError::from)
}

pub async fn remove_performance(
    pool: &MySqlPool,
    playlist_id: u32,
    performance_id: u32,
) -> Result<()> {
    sqlx::query("DELETE FROM playlist_performances WHERE playlist_id = ? AND performance_id = ?")
        .bind(playlist_id)
        .bind(performance_id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}
