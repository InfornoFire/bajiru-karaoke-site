use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Lyrics {
    pub id: i32,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewLyrics {
    pub content: String,
}
