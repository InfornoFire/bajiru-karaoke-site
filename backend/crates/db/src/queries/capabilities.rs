//! Query functions for the `capabilities` table.

use sqlx::{Executor, MySql, MySqlConnection};

use crate::error::DbError;
use crate::models::capability::{Capability, NewCapability};

type Result<T> = std::result::Result<T, DbError>;

/// Fetches a capability by ID.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: u32,
) -> Result<Option<Capability>> {
    sqlx::query_as::<_, Capability>("SELECT id, title FROM capabilities WHERE id = ?")
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(DbError::from)
}

/// Returns all capabilities ordered by title.
pub async fn list(executor: impl Executor<'_, Database = MySql>) -> Result<Vec<Capability>> {
    sqlx::query_as::<_, Capability>("SELECT id, title FROM capabilities ORDER BY title")
        .fetch_all(executor)
        .await
        .map_err(DbError::from)
}

/// Returns the capabilities assigned to a user.
pub async fn list_for_user(
    executor: impl Executor<'_, Database = MySql>,
    user_id: u32,
) -> Result<Vec<Capability>> {
    sqlx::query_as::<_, Capability>(
        "SELECT c.id, c.title \
         FROM capabilities c \
         JOIN user_capabilities uc ON uc.capability_id = c.id \
         WHERE uc.user_id = ? \
         ORDER BY c.title",
    )
    .bind(user_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Inserts a capability if it does not already exist, then returns the row.
pub async fn create(conn: &mut MySqlConnection, new: &NewCapability) -> Result<Capability> {
    sqlx::query("INSERT IGNORE INTO capabilities (title) VALUES (?)")
        .bind(&new.title)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    sqlx::query_as::<_, Capability>("SELECT id, title FROM capabilities WHERE title = ?")
        .bind(&new.title)
        .fetch_one(&mut *conn)
        .await
        .map_err(DbError::from)
}

/// Deletes a capability by ID. Returns `true` if a row was deleted.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, id: u32) -> Result<bool> {
    sqlx::query("DELETE FROM capabilities WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

/// Grants a capability to a user. Silently ignores duplicate grants.
pub async fn add_to_user(
    executor: impl Executor<'_, Database = MySql>,
    user_id: u32,
    capability_id: u32,
) -> Result<()> {
    sqlx::query("INSERT IGNORE INTO user_capabilities (user_id, capability_id) VALUES (?, ?)")
        .bind(user_id)
        .bind(capability_id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Revokes a capability from a user.
pub async fn remove_from_user(
    executor: impl Executor<'_, Database = MySql>,
    user_id: u32,
    capability_id: u32,
) -> Result<()> {
    sqlx::query("DELETE FROM user_capabilities WHERE user_id = ? AND capability_id = ?")
        .bind(user_id)
        .bind(capability_id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Returns whether a user holds a specific capability.
pub async fn user_has(
    executor: impl Executor<'_, Database = MySql>,
    user_id: u32,
    capability_id: u32,
) -> Result<bool> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(\
             SELECT 1 FROM user_capabilities \
             WHERE user_id = ? AND capability_id = ?\
         )",
    )
    .bind(user_id)
    .bind(capability_id)
    .fetch_one(executor)
    .await
    .map_err(DbError::from)
}
