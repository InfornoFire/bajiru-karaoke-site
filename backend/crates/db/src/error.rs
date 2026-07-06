//! Database error type used by all query functions.

/// Errors that can be returned by database operations.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// A raw sqlx error not mapped to a higher level variant.
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// A migration step failed on startup.
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    /// A query returned no rows where at least one was expected.
    #[error("record not found")]
    NotFound,

    /// A write was rejected due to a unique constraint violation.
    #[error("unique constraint violated")]
    Conflict,
}
