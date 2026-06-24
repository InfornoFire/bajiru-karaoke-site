use crate::error::DbError;
use crate::models::user::{NewUser, UpdateUser, User};
use sqlx::MySqlPool;

type Result<T> = std::result::Result<T, DbError>;

pub async fn get_by_id(pool: &MySqlPool, id: i32) -> Result<Option<User>> {
    sqlx::query_as::<_, User>("SELECT id, username, twitch_id, discord_id FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(DbError::from)
}

pub async fn get_by_twitch_id(pool: &MySqlPool, twitch_id: i64) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, twitch_id, discord_id FROM users WHERE twitch_id = ?",
    )
    .bind(twitch_id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::from)
}

pub async fn get_by_discord_id(pool: &MySqlPool, discord_id: i64) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, twitch_id, discord_id FROM users WHERE discord_id = ?",
    )
    .bind(discord_id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::from)
}

pub async fn get_by_username(pool: &MySqlPool, username: &str) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, twitch_id, discord_id FROM users WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(DbError::from)
}

pub async fn list(pool: &MySqlPool) -> Result<Vec<User>> {
    sqlx::query_as::<_, User>("SELECT id, username, twitch_id, discord_id FROM users ORDER BY id")
        .fetch_all(pool)
        .await
        .map_err(DbError::from)
}

pub async fn create(pool: &MySqlPool, new: &NewUser) -> Result<User> {
    let id = sqlx::query("INSERT INTO users (username, twitch_id, discord_id) VALUES (?, ?, ?)")
        .bind(&new.username)
        .bind(new.twitch_id)
        .bind(new.discord_id)
        .execute(pool)
        .await
        .map_err(DbError::from)?
        .last_insert_id();
    get_by_id(pool, id as i32).await?.ok_or(DbError::NotFound)
}

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

pub async fn update(pool: &MySqlPool, id: i32, upd: &UpdateUser) -> Result<Option<User>> {
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

pub async fn delete(pool: &MySqlPool, id: i32) -> Result<bool> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

pub async fn get_favorite_song_ids(pool: &MySqlPool, user_id: i32) -> Result<Vec<i32>> {
    sqlx::query_scalar::<_, i32>("SELECT song_id FROM user_favorite_songs WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(DbError::from)
}

pub async fn add_favorite_song(pool: &MySqlPool, user_id: i32, song_id: i32) -> Result<()> {
    sqlx::query("INSERT IGNORE INTO user_favorite_songs (user_id, song_id) VALUES (?, ?)")
        .bind(user_id)
        .bind(song_id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

pub async fn remove_favorite_song(pool: &MySqlPool, user_id: i32, song_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM user_favorite_songs WHERE user_id = ? AND song_id = ?")
        .bind(user_id)
        .bind(song_id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}
