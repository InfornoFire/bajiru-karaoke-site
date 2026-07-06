//! Lyrics subresource request and response types.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response body for lyrics GET endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LyricsResponse {
    pub content: String,
}

/// Request body for lyrics PUT endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateLyricsRequest {
    pub content: String,
}
