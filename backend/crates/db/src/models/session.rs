//! Session model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A server side session row, keyed by the SHA256 hash of the session token.
///
/// A request is authenticated by rehashing its cookie value and
/// looking up the matching row.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: u32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
