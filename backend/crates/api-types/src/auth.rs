use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RegisterRequest {
    /// Alphanumeric with `_` and `.` allowed, max 64 chars, case-insensitive unique
    pub username: String,
    /// Minimum 8 characters
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MeResponse {
    pub id: u32,
    pub username: String,
    pub capabilities: Vec<String>,
}
