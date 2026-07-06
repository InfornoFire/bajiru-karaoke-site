//! Shared response fragments used across multiple resource types.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A minimal artist record embedded in song and performance responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ArtistInfo {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
}

/// A tag embedded in song responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TagInfo {
    pub id: u32,
    pub name: String,
    /// Freeform category string (e.g. `"genre"`, `"mood"`).
    pub kind: String,
}

/// An image record embedded in song responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImageInfo {
    pub id: u32,
    pub public_url: String,
    pub credits: Option<String>,
}

/// A media file (audio or video) embedded in performance responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MediaInfo {
    pub id: u32,
    pub public_url: String,
}

/// Body returned for all error responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}
