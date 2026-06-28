use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::{ArtistInfo, MediaInfo};
use crate::songs::SongSummary;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePerformanceRequest {
    pub title: Option<String>,
    pub performance_date: DateTime<Utc>,
    pub duration: Option<u32>,
    pub song_ids: Vec<u32>,
    pub singer_ids: Vec<u32>,
    pub lyrics: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePerformanceRequest {
    pub title: Option<String>,
    pub performance_date: DateTime<Utc>,
    pub duration: Option<u32>,
    pub song_ids: Vec<u32>,
    pub singer_ids: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PerformanceSummary {
    pub id: u32,
    pub title: Option<String>,
    pub play_count: i32,
    pub duration: Option<u32>,
    pub performance_date: DateTime<Utc>,
    pub singers: Vec<ArtistInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PerformanceResponse {
    pub id: u32,
    pub title: Option<String>,
    pub play_count: i32,
    pub duration: Option<u32>,
    pub performance_date: DateTime<Utc>,
    pub songs: Vec<SongSummary>,
    pub singers: Vec<ArtistInfo>,
    pub audio: Vec<MediaInfo>,
    pub video: Vec<MediaInfo>,
}
