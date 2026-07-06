//! Performance video model.

use serde::{Deserialize, Serialize};

/// A video file attached to a performance.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct PerformanceVideo {
    pub id: u32,
    pub performance_id: u32,
    /// Publicly served URL for clients.
    pub public_url: String,
    /// Absolute filesystem path used for actual file.
    pub internal_path: Option<String>,
}

/// Input for creating a new performance video record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPerformanceVideo {
    pub performance_id: u32,
    pub public_url: String,
    pub internal_path: Option<String>,
}
