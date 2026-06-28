use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LyricsResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateLyricsRequest {
    pub content: String,
}
