//! Query functions for the `songs` table and its related join tables.
//!
//! M2M relations (artists, tags, images) are managed via full replace
//! helpers (e.g., `set_original_artists`, `set_tags`, `set_images`) that delete
//! existing rows and reinsert within a single transaction.

use std::collections::HashMap;

use crate::error::DbError;
use crate::models::artist::Artist;
use crate::models::image::Image;
use crate::models::song::{NewSong, Song, UpdateSong};
use crate::models::tag::Tag;
use sqlx::MySqlPool;

type Result<T> = std::result::Result<T, DbError>;

/// Fetches a song by ID.
pub async fn get_by_id(pool: &MySqlPool, id: u32) -> Result<Option<Song>> {
    sqlx::query_as::<_, Song>(
        "SELECT id, title, created_by, lyrics_id, date_added FROM songs WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::from)
}

/// Returns all songs ordered by date added descending.
pub async fn list(pool: &MySqlPool) -> Result<Vec<Song>> {
    sqlx::query_as::<_, Song>(
        "SELECT id, title, created_by, lyrics_id, date_added FROM songs ORDER BY date_added DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(DbError::from)
}

/// Inserts a new song and returns the created row.
pub async fn create(pool: &MySqlPool, new: &NewSong) -> Result<Song> {
    let id = sqlx::query("INSERT INTO songs (title, created_by, lyrics_id) VALUES (?, ?, ?)")
        .bind(&new.title)
        .bind(new.created_by)
        .bind(new.lyrics_id)
        .execute(pool)
        .await
        .map_err(DbError::from)?
        .last_insert_id();
    get_by_id(pool, id as u32).await?.ok_or(DbError::NotFound)
}

/// Updates a song's mutable fields. Returns `None` if the ID does not exist.
pub async fn update(pool: &MySqlPool, id: u32, upd: &UpdateSong) -> Result<Option<Song>> {
    let affected = sqlx::query("UPDATE songs SET title = ? WHERE id = ?")
        .bind(&upd.title)
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

/// Sets the `lyrics_id` foreign key on a song, or clears it with `None`.
pub async fn update_lyrics_id(pool: &MySqlPool, id: u32, lyrics_id: Option<u32>) -> Result<()> {
    sqlx::query("UPDATE songs SET lyrics_id = ? WHERE id = ?")
        .bind(lyrics_id)
        .bind(id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Deletes a song by ID. Returns `true` if a row was deleted.
pub async fn delete(pool: &MySqlPool, id: u32) -> Result<bool> {
    sqlx::query("DELETE FROM songs WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

/// Returns the original artists for a song via the `song_original_artists` join table.
pub async fn get_original_artists(pool: &MySqlPool, song_id: u32) -> Result<Vec<Artist>> {
    sqlx::query_as::<_, Artist>(
        "SELECT a.id, a.name, a.description \
         FROM artists a \
         JOIN song_original_artists soa ON soa.artist_id = a.id \
         WHERE soa.song_id = ?",
    )
    .bind(song_id)
    .fetch_all(pool)
    .await
    .map_err(DbError::from)
}

/// Returns original artists for multiple songs, keyed by song ID.
///
/// Songs with no original artists are absent from the returned map.
pub async fn get_original_artists_batch(
    pool: &MySqlPool,
    song_ids: &[u32],
) -> Result<HashMap<u32, Vec<Artist>>> {
    if song_ids.is_empty() {
        return Ok(HashMap::new());
    }

    #[derive(sqlx::FromRow)]
    struct Row {
        song_id: u32,
        id: u32,
        name: String,
        description: Option<String>,
    }

    let mut builder = sqlx::QueryBuilder::new(
        "SELECT soa.song_id, a.id, a.name, a.description \
         FROM artists a \
         JOIN song_original_artists soa ON soa.artist_id = a.id \
         WHERE soa.song_id IN (",
    );
    let mut separated = builder.separated(", ");
    for song_id in song_ids {
        separated.push_bind(song_id);
    }
    builder.push(")");

    let rows: Vec<Row> = builder
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(DbError::from)?;

    let mut by_song: HashMap<u32, Vec<Artist>> = HashMap::new();
    for row in rows {
        by_song.entry(row.song_id).or_default().push(Artist {
            id: row.id,
            name: row.name,
            description: row.description,
        });
    }
    Ok(by_song)
}

/// Replaces the full set of original artists for a song within a transaction.
pub async fn set_original_artists(
    pool: &MySqlPool,
    song_id: u32,
    artist_ids: &[u32],
) -> Result<()> {
    let mut tx = pool.begin().await.map_err(DbError::from)?;
    sqlx::query("DELETE FROM song_original_artists WHERE song_id = ?")
        .bind(song_id)
        .execute(&mut *tx)
        .await
        .map_err(DbError::from)?;
    for &artist_id in artist_ids {
        sqlx::query("INSERT INTO song_original_artists (song_id, artist_id) VALUES (?, ?)")
            .bind(song_id)
            .bind(artist_id)
            .execute(&mut *tx)
            .await
            .map_err(DbError::from)?;
    }
    tx.commit().await.map_err(DbError::from)
}

/// Returns the tags for a song via the `song_tags` join table.
pub async fn get_tags(pool: &MySqlPool, song_id: u32) -> Result<Vec<Tag>> {
    sqlx::query_as::<_, Tag>(
        "SELECT t.id, t.name, t.kind \
         FROM tags t \
         JOIN song_tags st ON st.tag_id = t.id \
         WHERE st.song_id = ?",
    )
    .bind(song_id)
    .fetch_all(pool)
    .await
    .map_err(DbError::from)
}

/// Replaces the full set of tags for a song within a transaction.
pub async fn set_tags(pool: &MySqlPool, song_id: u32, tag_ids: &[u32]) -> Result<()> {
    let mut tx = pool.begin().await.map_err(DbError::from)?;
    sqlx::query("DELETE FROM song_tags WHERE song_id = ?")
        .bind(song_id)
        .execute(&mut *tx)
        .await
        .map_err(DbError::from)?;
    for &tag_id in tag_ids {
        sqlx::query("INSERT INTO song_tags (song_id, tag_id) VALUES (?, ?)")
            .bind(song_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await
            .map_err(DbError::from)?;
    }
    tx.commit().await.map_err(DbError::from)
}

/// Returns the images for a song via the `song_images` join table.
pub async fn get_images(pool: &MySqlPool, song_id: u32) -> Result<Vec<Image>> {
    sqlx::query_as::<_, Image>(
        "SELECT i.id, i.public_url, i.internal_path, i.credits \
         FROM images i \
         JOIN song_images si ON si.image_id = i.id \
         WHERE si.song_id = ?",
    )
    .bind(song_id)
    .fetch_all(pool)
    .await
    .map_err(DbError::from)
}

/// Replaces the full set of images for a song within a transaction.
pub async fn set_images(pool: &MySqlPool, song_id: u32, image_ids: &[u32]) -> Result<()> {
    let mut tx = pool.begin().await.map_err(DbError::from)?;
    sqlx::query("DELETE FROM song_images WHERE song_id = ?")
        .bind(song_id)
        .execute(&mut *tx)
        .await
        .map_err(DbError::from)?;
    for &image_id in image_ids {
        sqlx::query("INSERT INTO song_images (song_id, image_id) VALUES (?, ?)")
            .bind(song_id)
            .bind(image_id)
            .execute(&mut *tx)
            .await
            .map_err(DbError::from)?;
    }
    tx.commit().await.map_err(DbError::from)
}
