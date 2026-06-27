use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Song {
    pub id: u32,
    pub title: String,
    pub created_by: Option<u32>,
    pub lyrics_id: Option<u32>,
    pub date_added: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSong {
    pub title: String,
    pub created_by: Option<u32>,
    pub lyrics_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSong {
    pub title: String,
    pub lyrics_id: Option<u32>,
}
