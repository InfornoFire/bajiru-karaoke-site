//! Password credential model.

use serde::{Deserialize, Serialize};

/// An argon2 password hash stored for a user.
///
/// Absent row means the account was created via OAuth and has no password.
/// The hash is a PHC string with the salt embedded.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct UserCredential {
    pub user_id: u32,
    pub password_hash: String,
}
