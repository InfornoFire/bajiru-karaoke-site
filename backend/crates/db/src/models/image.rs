use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Image {
    pub id: i32,
    pub public_url: String,
    pub internal_path: Option<String>,
    pub credits: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewImage {
    pub public_url: String,
    pub internal_path: Option<String>,
    pub credits: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateImage {
    pub public_url: String,
    pub internal_path: Option<String>,
    pub credits: Option<String>,
}
