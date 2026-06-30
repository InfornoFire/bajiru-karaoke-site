use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MeResponse {
    pub id: u32,
    pub username: String,
    pub capabilities: Vec<String>,
}
