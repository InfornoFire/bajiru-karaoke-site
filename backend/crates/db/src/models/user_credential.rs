use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct UserCredential {
    pub user_id: u32,
    pub password_hash: String,
}
