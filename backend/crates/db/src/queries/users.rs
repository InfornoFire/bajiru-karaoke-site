//! Query functions for the `users` table.

use crate::error::DbError;
use crate::models::user::{NewUser, UpdateUser, User};
use sqlx::MySqlPool;

type Result<T> = std::result::Result<T, DbError>;

/// Fetches a user by primary key.
pub async fn get_by_id(pool: &MySqlPool, id: u32) -> Result<Option<User>> {
    sqlx::query_as::<_, User>("SELECT id, username, twitch_id, discord_id FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(DbError::from)
}

/// Fetches a user by Twitch provider ID.
pub async fn get_by_twitch_id(pool: &MySqlPool, twitch_id: u64) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, twitch_id, discord_id FROM users WHERE twitch_id = ?",
    )
    .bind(twitch_id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::from)
}

/// Fetches a user by Discord provider ID.
pub async fn get_by_discord_id(pool: &MySqlPool, discord_id: u64) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, twitch_id, discord_id FROM users WHERE discord_id = ?",
    )
    .bind(discord_id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::from)
}

/// Fetches a user by username (case insensitive via DB collation).
pub async fn get_by_username(pool: &MySqlPool, username: &str) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, twitch_id, discord_id FROM users WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(DbError::from)
}

/// Returns all users ordered by ID.
pub async fn list(pool: &MySqlPool) -> Result<Vec<User>> {
    sqlx::query_as::<_, User>("SELECT id, username, twitch_id, discord_id FROM users ORDER BY id")
        .fetch_all(pool)
        .await
        .map_err(DbError::from)
}

/// Inserts a new user and returns the created row.
///
/// # Errors
///
/// Returns [`DbError::Conflict`] if the username is already taken.
pub async fn create(pool: &MySqlPool, new: &NewUser) -> Result<User> {
    let result =
        sqlx::query("INSERT INTO users (username, twitch_id, discord_id) VALUES (?, ?, ?)")
            .bind(&new.username)
            .bind(new.twitch_id)
            .bind(new.discord_id)
            .execute(pool)
            .await;

    let id = match result {
        Ok(r) => r.last_insert_id(),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => return Err(DbError::Conflict),
        Err(e) => return Err(DbError::Sqlx(e)),
    };

    get_by_id(pool, id as u32).await?.ok_or(DbError::NotFound)
}

/// Inserts a user keyed on `twitch_id`, updating `username` on conflict.
///
/// Used on every successful Twitch OAuth login so the username stays in sync
/// with the user's current Twitch display name.
pub async fn upsert_by_twitch(pool: &MySqlPool, new: &NewUser) -> Result<User> {
    sqlx::query(
        "INSERT INTO users (username, twitch_id, discord_id) VALUES (?, ?, ?) \
         ON DUPLICATE KEY UPDATE username = VALUES(username)",
    )
    .bind(&new.username)
    .bind(new.twitch_id)
    .bind(new.discord_id)
    .execute(pool)
    .await
    .map_err(DbError::from)?;
    get_by_twitch_id(pool, new.twitch_id.ok_or(DbError::NotFound)?)
        .await?
        .ok_or(DbError::NotFound)
}

/// Inserts a user keyed on `discord_id`, updating `username` on conflict.
///
/// Used on every successful Discord OAuth login so the username stays in sync
/// with the user's current Discord username.
pub async fn upsert_by_discord(pool: &MySqlPool, new: &NewUser) -> Result<User> {
    sqlx::query(
        "INSERT INTO users (username, twitch_id, discord_id) VALUES (?, ?, ?) \
         ON DUPLICATE KEY UPDATE username = VALUES(username)",
    )
    .bind(&new.username)
    .bind(new.twitch_id)
    .bind(new.discord_id)
    .execute(pool)
    .await
    .map_err(DbError::from)?;
    get_by_discord_id(pool, new.discord_id.ok_or(DbError::NotFound)?)
        .await?
        .ok_or(DbError::NotFound)
}

/// Replaces all mutable fields on a user. Returns `None` if the ID does not exist.
pub async fn update(pool: &MySqlPool, id: u32, upd: &UpdateUser) -> Result<Option<User>> {
    let affected =
        sqlx::query("UPDATE users SET username = ?, twitch_id = ?, discord_id = ? WHERE id = ?")
            .bind(&upd.username)
            .bind(upd.twitch_id)
            .bind(upd.discord_id)
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

/// Deletes a user by ID. Returns `true` if a row was deleted.
pub async fn delete(pool: &MySqlPool, id: u32) -> Result<bool> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

/// Returns the IDs of all songs the user has favorited.
pub async fn get_favorite_song_ids(pool: &MySqlPool, user_id: u32) -> Result<Vec<u32>> {
    sqlx::query_scalar::<_, u32>("SELECT song_id FROM user_favorite_songs WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(DbError::from)
}

/// Adds a song to the user's favorites. Silently ignores duplicates.
pub async fn add_favorite_song(pool: &MySqlPool, user_id: u32, song_id: u32) -> Result<()> {
    sqlx::query("INSERT IGNORE INTO user_favorite_songs (user_id, song_id) VALUES (?, ?)")
        .bind(user_id)
        .bind(song_id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Removes a song from the user's favorites.
pub async fn remove_favorite_song(pool: &MySqlPool, user_id: u32, song_id: u32) -> Result<()> {
    sqlx::query("DELETE FROM user_favorite_songs WHERE user_id = ? AND song_id = ?")
        .bind(user_id)
        .bind(song_id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}
