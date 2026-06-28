use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Performance {
    pub id: u32,
    pub created_by: Option<u32>,
    pub title: Option<String>,
    pub lyrics_id: Option<u32>,
    pub play_count: i32,
    pub duration: Option<u32>,
    pub performance_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPerformance {
    pub created_by: Option<u32>,
    pub title: Option<String>,
    pub lyrics_id: Option<u32>,
    pub duration: Option<u32>,
    pub performance_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePerformance {
    pub title: Option<String>,
    pub lyrics_id: Option<u32>,
    pub duration: Option<u32>,
    pub performance_date: DateTime<Utc>,
}
