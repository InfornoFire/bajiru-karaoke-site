use crate::error::DbError;
use crate::models::song::{NewSong, Song, UpdateSong};
use sqlx::MySqlPool;

type Result<T> = std::result::Result<T, DbError>;

pub async fn get_by_id(pool: &MySqlPool, id: i32) -> Result<Option<Song>> {
    sqlx::query_as::<_, Song>(
        "SELECT id, title, created_by, lyrics_id, date_added FROM songs WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::from)
}

pub async fn list(pool: &MySqlPool) -> Result<Vec<Song>> {
    sqlx::query_as::<_, Song>(
        "SELECT id, title, created_by, lyrics_id, date_added FROM songs ORDER BY date_added DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(DbError::from)
}

pub async fn create(pool: &MySqlPool, new: &NewSong) -> Result<Song> {
    let id = sqlx::query("INSERT INTO songs (title, created_by, lyrics_id) VALUES (?, ?, ?)")
        .bind(&new.title)
        .bind(new.created_by)
        .bind(new.lyrics_id)
        .execute(pool)
        .await
        .map_err(DbError::from)?
        .last_insert_id();
    get_by_id(pool, id as i32).await?.ok_or(DbError::NotFound)
}

pub async fn update(pool: &MySqlPool, id: i32, upd: &UpdateSong) -> Result<Option<Song>> {
    let affected = sqlx::query("UPDATE songs SET title = ?, lyrics_id = ? WHERE id = ?")
        .bind(&upd.title)
        .bind(upd.lyrics_id)
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
    sqlx::query("DELETE FROM songs WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

pub async fn get_original_artist_ids(pool: &MySqlPool, song_id: i32) -> Result<Vec<i32>> {
    sqlx::query_scalar::<_, i32>("SELECT artist_id FROM song_original_artists WHERE song_id = ?")
        .bind(song_id)
        .fetch_all(pool)
        .await
        .map_err(DbError::from)
}

pub async fn set_original_artists(
    pool: &MySqlPool,
    song_id: i32,
    artist_ids: &[i32],
) -> Result<()> {
    let mut tx = pool.begin().await.map_err(DbError::from)?;
    sqlx::query("DELETE FROM song_original_artists WHERE song_id = ?")
        .bind(song_id)
        .execute(&mut *tx)
        .await
        .map_err(DbError::from)?;
    for &artist_id in artist_ids {
        sqlx::query("INSERT IGNORE INTO song_original_artists (song_id, artist_id) VALUES (?, ?)")
            .bind(song_id)
            .bind(artist_id)
            .execute(&mut *tx)
            .await
            .map_err(DbError::from)?;
    }
    tx.commit().await.map_err(DbError::from)
}

pub async fn get_tag_ids(pool: &MySqlPool, song_id: i32) -> Result<Vec<i32>> {
    sqlx::query_scalar::<_, i32>("SELECT tag_id FROM song_tags WHERE song_id = ?")
        .bind(song_id)
        .fetch_all(pool)
        .await
        .map_err(DbError::from)
}

pub async fn set_tags(pool: &MySqlPool, song_id: i32, tag_ids: &[i32]) -> Result<()> {
    let mut tx = pool.begin().await.map_err(DbError::from)?;
    sqlx::query("DELETE FROM song_tags WHERE song_id = ?")
        .bind(song_id)
        .execute(&mut *tx)
        .await
        .map_err(DbError::from)?;
    for &tag_id in tag_ids {
        sqlx::query("INSERT IGNORE INTO song_tags (song_id, tag_id) VALUES (?, ?)")
            .bind(song_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await
            .map_err(DbError::from)?;
    }
    tx.commit().await.map_err(DbError::from)
}

pub async fn get_image_ids(pool: &MySqlPool, song_id: i32) -> Result<Vec<i32>> {
    sqlx::query_scalar::<_, i32>("SELECT image_id FROM song_images WHERE song_id = ?")
        .bind(song_id)
        .fetch_all(pool)
        .await
        .map_err(DbError::from)
}

pub async fn set_images(pool: &MySqlPool, song_id: i32, image_ids: &[i32]) -> Result<()> {
    let mut tx = pool.begin().await.map_err(DbError::from)?;
    sqlx::query("DELETE FROM song_images WHERE song_id = ?")
        .bind(song_id)
        .execute(&mut *tx)
        .await
        .map_err(DbError::from)?;
    for &image_id in image_ids {
        sqlx::query("INSERT IGNORE INTO song_images (song_id, image_id) VALUES (?, ?)")
            .bind(song_id)
            .bind(image_id)
            .execute(&mut *tx)
            .await
            .map_err(DbError::from)?;
    }
    tx.commit().await.map_err(DbError::from)
}
