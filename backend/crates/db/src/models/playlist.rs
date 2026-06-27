use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Playlist {
    pub id: u32,
    pub title: String,
    pub description: Option<String>,
    pub kind: String,
    pub created_by: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPlaylist {
    pub title: String,
    pub description: Option<String>,
    pub kind: String,
    pub created_by: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlaylist {
    pub title: String,
    pub description: Option<String>,
    pub kind: String,
}
