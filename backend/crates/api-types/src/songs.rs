use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::{ArtistInfo, ImageInfo, TagInfo};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSongRequest {
    pub title: String,
    pub artist_ids: Vec<u32>,
    pub tag_ids: Vec<u32>,
    pub image_ids: Vec<u32>,
    pub lyrics: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSongRequest {
    pub title: String,
    pub artist_ids: Vec<u32>,
    pub tag_ids: Vec<u32>,
    pub image_ids: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SongSummary {
    pub id: u32,
    pub title: String,
    pub date_added: DateTime<Utc>,
    pub artists: Vec<ArtistInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SongResponse {
    pub id: u32,
    pub title: String,
    pub date_added: DateTime<Utc>,
    pub artists: Vec<ArtistInfo>,
    pub tags: Vec<TagInfo>,
    pub images: Vec<ImageInfo>,
}
