use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ArtistInfo {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TagInfo {
    pub id: u32,
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImageInfo {
    pub id: u32,
    pub public_url: String,
    pub credits: Option<String>,
}

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
