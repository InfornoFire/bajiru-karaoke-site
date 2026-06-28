use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct PerformanceAudio {
    pub id: u32,
    pub performance_id: u32,
    pub public_url: String,
    pub internal_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPerformanceAudio {
    pub performance_id: u32,
    pub public_url: String,
    pub internal_path: Option<String>,
}
