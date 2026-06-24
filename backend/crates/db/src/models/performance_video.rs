use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct PerformanceVideo {
    pub id: i32,
    pub performance_id: i32,
    pub public_url: String,
    pub internal_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPerformanceVideo {
    pub performance_id: i32,
    pub public_url: String,
    pub internal_path: Option<String>,
}
