use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub twitch_id: Option<u64>,
    pub discord_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub twitch_id: Option<u64>,
    pub discord_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    pub username: String,
    pub twitch_id: Option<u64>,
    pub discord_id: Option<u64>,
}
